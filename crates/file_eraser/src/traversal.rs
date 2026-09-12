use crate::errors::EraserResult;
use crate::filesystem_probe::FilesystemProber;
use crate::model::*;
use crate::platform::PlatformProvider;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Crawls directory hierarchies under STRICT READ-ONLY guarantees.
///
/// Cycle Prevention & Boundary Guarantees:
/// - Path deduplication: `visited_paths` tracks every normalized path enqueued,
///   preventing redundant crawls of the exact same directory.
/// - Hard Reparse Point Boundaries: Symlinks, junction points, mount points, and unknown
///   reparse points are NEVER traversed or enqueued. They are classified as boundaries.
/// - Hard-link note: Path-based deduplication visits each distinct directory path once.
///   Hard links across files share file identity tokens without creating directory cycles
///   because Windows disallows directory hard links.
pub struct DirectoryCrawler<P: PlatformProvider> {
    prober: FilesystemProber<P>,
    platform: P,
}

impl<P: PlatformProvider + Clone> DirectoryCrawler<P> {
    pub fn new(platform: P) -> EraserResult<Self> {
        let prober = FilesystemProber::new(platform.clone())?;
        Ok(Self { prober, platform })
    }

    pub fn crawl(
        &self,
        root: &Path,
        stats: &mut ResolutionStatistics,
    ) -> EraserResult<Vec<TargetProbeData>> {
        let mut results = Vec::new();
        let mut visited_paths = HashSet::<PathBuf>::new();
        let mut queue = Vec::<PathBuf>::new();

        let initial_probe = self.prober.probe_target(root)?;
        let root_norm = initial_probe.normalized_path.clone();

        visited_paths.insert(root_norm.clone());
        self.tally_stat(&initial_probe, stats);

        let can_traverse = initial_probe.kind.is_traversable_directory()
            && matches!(initial_probe.safety, SafetyClassification::SafeToAnalyze);

        results.push(initial_probe);

        if can_traverse {
            queue.push(root_norm);
        }

        while let Some(current_dir) = queue.pop() {
            let entries = match self.platform.read_dir_entries(&current_dir) {
                Ok(e) => e,
                Err(err) => {
                    stats.inaccessible_targets_encountered += 1;
                    results.push(TargetProbeData {
                        original_path: current_dir.clone(),
                        normalized_path: current_dir.clone(),
                        kind: TargetKind::Inaccessible {
                            reason: format!("Failed to read directory entries: {}", err),
                        },
                        size_bytes: ProbedValue::Unavailable {
                            reason: format!("Directory read failed: {}", err),
                        },
                        allocated_size_bytes: ProbedValue::Unavailable {
                            reason: format!("Directory read failed: {}", err),
                        },
                        identity: ProbedValue::Unavailable {
                            reason: format!("Directory read failed: {}", err),
                        },
                        is_sparse: ProbedValue::Unavailable {
                            reason: format!("Directory read failed: {}", err),
                        },
                        is_compressed: ProbedValue::Unavailable {
                            reason: format!("Directory read failed: {}", err),
                        },
                        is_encrypted: ProbedValue::Unavailable {
                            reason: format!("Directory read failed: {}", err),
                        },
                        is_reparse_point: ProbedValue::Unavailable {
                            reason: format!("Directory read failed: {}", err),
                        },
                        reparse_tag: ProbedValue::Unavailable {
                            reason: format!("Directory read failed: {}", err),
                        },
                        alternate_data_streams: ProbedValue::Unavailable {
                            reason: format!("Directory read failed: {}", err),
                        },
                        volume_info: ProbedValue::Unavailable {
                            reason: format!("Directory read failed: {}", err),
                        },
                        safety: SafetyClassification::Inaccessible {
                            reason: err.to_string(),
                        },
                    });
                    continue;
                }
            };

            for child_path in entries {
                let child_norm = self
                    .platform
                    .normalize_path(&child_path)
                    .unwrap_or_else(|_| child_path.clone());

                if visited_paths.contains(&child_norm) {
                    continue;
                }
                visited_paths.insert(child_norm.clone());

                // Fault-tolerant child probing: An inaccessible file/directory encountered
                // during traversal must NOT abort the entire crawl.
                let probe = match self.prober.probe_target(&child_norm) {
                    Ok(p) => p,
                    Err(err) => {
                        let reason = err.to_string();
                        TargetProbeData {
                            original_path: child_path.clone(),
                            normalized_path: child_norm.clone(),
                            kind: TargetKind::Inaccessible {
                                reason: reason.clone(),
                            },
                            size_bytes: ProbedValue::Unavailable {
                                reason: format!("Probe failed: {}", reason),
                            },
                            allocated_size_bytes: ProbedValue::Unavailable {
                                reason: format!("Probe failed: {}", reason),
                            },
                            identity: ProbedValue::Unavailable {
                                reason: format!("Probe failed: {}", reason),
                            },
                            is_sparse: ProbedValue::Unavailable {
                                reason: format!("Probe failed: {}", reason),
                            },
                            is_compressed: ProbedValue::Unavailable {
                                reason: format!("Probe failed: {}", reason),
                            },
                            is_encrypted: ProbedValue::Unavailable {
                                reason: format!("Probe failed: {}", reason),
                            },
                            is_reparse_point: ProbedValue::Unavailable {
                                reason: format!("Probe failed: {}", reason),
                            },
                            reparse_tag: ProbedValue::Unavailable {
                                reason: format!("Probe failed: {}", reason),
                            },
                            alternate_data_streams: ProbedValue::Unavailable {
                                reason: format!("Probe failed: {}", reason),
                            },
                            volume_info: ProbedValue::Unavailable {
                                reason: format!("Probe failed: {}", reason),
                            },
                            safety: SafetyClassification::Inaccessible {
                                reason: reason.clone(),
                            },
                        }
                    }
                };

                self.tally_stat(&probe, stats);

                // STRICT BOUNDARY: Only standard, successfully verified directories are enqueued.
                // Reparse points (junctions/symlinks) and inaccessible nodes MUST NOT be traversed.
                if probe.kind.is_traversable_directory()
                    && matches!(probe.safety, SafetyClassification::SafeToAnalyze)
                {
                    queue.push(child_norm);
                }

                results.push(probe);
            }
        }

        Ok(results)
    }

    fn tally_stat(&self, probe: &TargetProbeData, stats: &mut ResolutionStatistics) {
        match &probe.kind {
            TargetKind::RegularFile => stats.regular_files_discovered += 1,
            TargetKind::Directory => stats.directories_discovered += 1,
            k if k.is_reparse_boundary() => stats.reparse_boundaries_encountered += 1,
            TargetKind::Inaccessible { .. } => stats.inaccessible_targets_encountered += 1,
            _ => {}
        }
        if matches!(
            probe.safety,
            SafetyClassification::ProtectedSystemPath { .. }
        ) {
            stats.protected_targets_encountered += 1;
        }
    }
}
