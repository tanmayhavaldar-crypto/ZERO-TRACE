// erasure_commands.rs
//
// Drop this file into `src-tauri/src/erasure_commands.rs`, then in
// `src-tauri/src/main.rs` (or `lib.rs`) add:
//
//     mod erasure_commands;
//
// and register the two commands in your invoke_handler, e.g.:
//
//     .invoke_handler(tauri::generate_handler![
//         erasure_commands::prepare_erasure_targets,
//         erasure_commands::run_secure_erasure,
//     ])
//
// These replace `run_mock_drive_erasure`. The frontend now calls
// `prepare_erasure_targets` and `run_secure_erasure` with a `targets` key,
// so the parameter name here (`targets`) must match exactly — this is what
// was mismatched before and caused "missing required key input".
//
// NOTE ON SSDs: overwrite-based erasure (what this does) is only a strong
// guarantee on traditional HDDs. Modern SSDs remap blocks internally (wear
// levelling / TRIM), so an overwrite may not touch the physical cells that
// still hold the old data. If real SSD support matters, look at the drive's
// native ATA/NVMe Secure Erase command instead of byte overwriting.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErasureTargetInput {
    pub path: String,
    pub is_directory: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrepareResult {
    pub total_files: u64,
    pub total_size_bytes: u64,
    pub warnings: Vec<String>,
}

#[tauri::command]
pub fn prepare_erasure_targets(targets: Vec<ErasureTargetInput>) -> Result<PrepareResult, String> {
    let mut total_files = 0u64;
    let mut total_size_bytes = 0u64;
    let mut warnings = Vec::new();

    for target in &targets {
        let path = Path::new(&target.path);
        if !path.exists() {
            warnings.push(format!("Path not found, will be skipped: {}", target.path));
            continue;
        }

        if target.is_directory {
            match walk_dir_totals(path) {
                Ok((files, size)) => {
                    total_files += files;
                    total_size_bytes += size;
                }
                Err(e) => warnings.push(format!("Could not read {}: {}", target.path, e)),
            }
        } else {
            match fs::metadata(path) {
                Ok(meta) => {
                    total_files += 1;
                    total_size_bytes += meta.len();
                }
                Err(e) => warnings.push(format!("Could not read {}: {}", target.path, e)),
            }
        }
    }

    Ok(PrepareResult {
        total_files,
        total_size_bytes,
        warnings,
    })
}

fn walk_dir_totals(dir: &Path) -> std::io::Result<(u64, u64)> {
    let mut count = 0u64;
    let mut size = 0u64;
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let (c, s) = walk_dir_totals(&path)?;
            count += c;
            size += s;
        } else {
            count += 1;
            size += entry.metadata()?.len();
        }
    }
    Ok((count, size))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EraseOutcome {
    pub path: String,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ErasureProgressEvent {
    path: String,
    message: String,
}

#[tauri::command]
pub async fn run_secure_erasure(
    app: AppHandle,
    targets: Vec<ErasureTargetInput>,
    passes: u8,
) -> Result<Vec<EraseOutcome>, String> {
    let mut outcomes = Vec::with_capacity(targets.len());

    for target in targets {
        let _ = app.emit(
            "erasure-progress",
            ErasureProgressEvent {
                path: target.path.clone(),
                message: format!("Starting {}-pass overwrite: {}", passes, target.path),
            },
        );

        let result = erase_path(&app, &target.path, target.is_directory, passes);

        outcomes.push(match result {
            Ok(_) => {
                let _ = app.emit(
                    "erasure-progress",
                    ErasureProgressEvent {
                        path: target.path.clone(),
                        message: format!("Finished: {}", target.path),
                    },
                );
                EraseOutcome {
                    path: target.path,
                    success: true,
                    message: "Securely erased.".to_string(),
                }
            }
            Err(e) => EraseOutcome {
                path: target.path,
                success: false,
                message: e,
            },
        });
    }

    Ok(outcomes)
}

fn erase_path(app: &AppHandle, path_str: &str, is_directory: bool, passes: u8) -> Result<(), String> {
    let path = Path::new(path_str);
    if !path.exists() {
        return Err("Path no longer exists.".to_string());
    }

    if is_directory {
        for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let p = entry.path();
            if p.is_dir() {
                erase_path(app, &p.to_string_lossy(), true, passes)?;
            } else {
                overwrite_file(&p, passes)?;
                let _ = app.emit(
                    "erasure-progress",
                    ErasureProgressEvent {
                        path: p.to_string_lossy().to_string(),
                        message: format!("Overwritten: {}", p.to_string_lossy()),
                    },
                );
                fs::remove_file(&p).map_err(|e| e.to_string())?;
            }
        }
        fs::remove_dir(path).map_err(|e| e.to_string())?;
    } else {
        overwrite_file(path, passes)?;
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn overwrite_file(path: &Path, passes: u8) -> Result<(), String> {
    let len = fs::metadata(path).map_err(|e| e.to_string())?.len();
    let mut file = fs::OpenOptions::new()
        .write(true)
        .open(path)
        .map_err(|e| e.to_string())?;

    let mut rng_state = seed_from_time();

    for pass in 0..passes {
        file.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
        let chunk_size = 8192usize;
        let mut written = 0u64;
        while written < len {
            let this_chunk = std::cmp::min(chunk_size as u64, len - written) as usize;
            let buffer = match pass % 3 {
                0 => vec![0x00u8; this_chunk],
                1 => vec![0xFFu8; this_chunk],
                _ => random_bytes(&mut rng_state, this_chunk),
            };
            file.write_all(&buffer).map_err(|e| e.to_string())?;
            written += this_chunk as u64;
        }
        file.flush().map_err(|e| e.to_string())?;
    }

    Ok(())
}

// Small dependency-free xorshift64 PRNG — good enough for overwrite noise,
// not for anything cryptographic.
fn seed_from_time() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x9E3779B97F4A7C15);
    if nanos == 0 {
        0x9E3779B97F4A7C15
    } else {
        nanos
    }
}

fn random_bytes(state: &mut u64, len: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(len);
    while out.len() < len {
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;
        out.extend_from_slice(&state.to_le_bytes());
    }
    out.truncate(len);
    out
}
