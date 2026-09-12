#[cfg(windows)]
use crate::errors::{EraserError, EraserResult};
#[cfg(windows)]
use crate::model::*;
#[cfg(windows)]
use crate::platform::{PlatformFileAttributes, PlatformProvider};
#[cfg(windows)]
use std::ffi::OsString;
#[cfg(windows)]
use std::os::windows::ffi::{OsStrExt, OsStringExt};
#[cfg(windows)]
use std::path::{Path, PathBuf};
#[cfg(windows)]
use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_FILE_NOT_FOUND, ERROR_NO_MORE_FILES, INVALID_HANDLE_VALUE,
};
#[cfg(windows)]
use windows_sys::Win32::Storage::FileSystem::*;

// Official Windows reparse tag constants from winnt.h / ddk
#[cfg(windows)]
const IO_REPARSE_TAG_MOUNT_POINT: u32 = 0xA000_0003;
#[cfg(windows)]
const IO_REPARSE_TAG_SYMLINK: u32 = 0xA000_000C;
#[cfg(windows)]
const IO_REPARSE_TAG_APPEXECLINK: u32 = 0x8000_001B;

// Filesystem volume capability flags from winnt.h
// Note: windows-sys 0.59 does not expose these under Storage_FileSystem.
#[cfg(windows)]
const FILE_FILE_COMPRESSION: u32 = 0x0000_0010;
#[cfg(windows)]
const FILE_NAMED_STREAMS: u32 = 0x0004_0000;
#[cfg(windows)]
const FILE_READ_ONLY_VOLUME: u32 = 0x0008_0000;
#[cfg(windows)]
const FILE_SUPPORTS_SPARSE_FILES: u32 = 0x0000_0040;
#[cfg(windows)]
const FILE_SUPPORTS_ENCRYPTION: u32 = 0x0002_0000;

#[cfg(windows)]
#[derive(Debug, Clone, Default)]
pub struct WindowsPlatformProvider;

#[cfg(windows)]
fn to_wide(path: &Path) -> Vec<u16> {
    path.as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

#[cfg(windows)]
impl PlatformProvider for WindowsPlatformProvider {
    fn normalize_path(&self, path: &Path) -> EraserResult<PathBuf> {
        let wide = to_wide(path);
        let mut buf = vec![0u16; 4096];
        let len = unsafe {
            GetFullPathNameW(
                wide.as_ptr(),
                buf.len() as u32,
                buf.as_mut_ptr(),
                std::ptr::null_mut(),
            )
        };
        if len == 0 {
            return Err(EraserError::NormalizationFailed {
                path: path.to_path_buf(),
                reason: format!("GetFullPathNameW failed: err code {}", unsafe {
                    GetLastError()
                }),
            });
        }
        let os_str = OsString::from_wide(&buf[..len as usize]);
        Ok(PathBuf::from(os_str))
    }

    fn query_target_kind(&self, path: &Path) -> EraserResult<TargetKind> {
        let wide = to_wide(path);
        let attrs = unsafe { GetFileAttributesW(wide.as_ptr()) };
        if attrs == INVALID_FILE_ATTRIBUTES {
            let code = unsafe { GetLastError() };
            return Err(EraserError::TargetInaccessible {
                path: path.to_path_buf(),
                reason: format!("GetFileAttributesW error {}", code),
            });
        }

        if attrs & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            match self.query_reparse_tag(path) {
                Ok(ReparseTagType::Symlink) => {
                    if attrs & FILE_ATTRIBUTE_DIRECTORY != 0 {
                        Ok(TargetKind::SymlinkDirectory)
                    } else {
                        Ok(TargetKind::SymlinkFile)
                    }
                }
                Ok(ReparseTagType::MountPointJunction) => Ok(TargetKind::Junction),
                Ok(ReparseTagType::None) => Ok(TargetKind::UnknownReparsePoint {
                    reason: "Reparse attribute flag present but tag was reported as none".into(),
                }),
                Ok(ReparseTagType::QueryFailed(reason)) => {
                    Ok(TargetKind::UnknownReparsePoint { reason })
                }
                Ok(other) => Ok(TargetKind::OtherReparsePoint {
                    tag_hex: format!("{:?}", other),
                }),
                Err(e) => Ok(TargetKind::UnknownReparsePoint {
                    reason: e.to_string(),
                }),
            }
        } else if attrs & FILE_ATTRIBUTE_DIRECTORY != 0 {
            Ok(TargetKind::Directory)
        } else if attrs & FILE_ATTRIBUTE_DEVICE != 0 {
            Ok(TargetKind::SpecialDevice)
        } else {
            Ok(TargetKind::RegularFile)
        }
    }

    /// Read-only reparse tag inspection using FindFirstFileW.
    /// Invariant: UNKNOWN MUST NOT BE TREATED AS NONE.
    fn query_reparse_tag(&self, path: &Path) -> EraserResult<ReparseTagType> {
        let wide = to_wide(path);
        let mut find_data: WIN32_FIND_DATAW = unsafe { std::mem::zeroed() };
        let h_find = unsafe { FindFirstFileW(wide.as_ptr(), &mut find_data) };
        if h_find == INVALID_HANDLE_VALUE {
            let code = unsafe { GetLastError() };
            return Err(EraserError::ReparseQueryFailed {
                path: path.to_path_buf(),
                reason: format!("FindFirstFileW failed with OS error {}", code),
            });
        }
        unsafe { FindClose(h_find) };

        if find_data.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT == 0 {
            return Ok(ReparseTagType::None);
        }

        let tag = find_data.dwReserved0;
        match tag {
            IO_REPARSE_TAG_SYMLINK => Ok(ReparseTagType::Symlink),
            IO_REPARSE_TAG_MOUNT_POINT => Ok(ReparseTagType::MountPointJunction),
            IO_REPARSE_TAG_APPEXECLINK => Ok(ReparseTagType::AppExecLink),
            0x80000018 => Ok(ReparseTagType::WslSymlink),
            0x9000001A | 0x9000101A | 0x9000201A => Ok(ReparseTagType::CloudPlaceholder),
            other => Ok(ReparseTagType::Unknown(other)),
        }
    }

    fn query_file_size(&self, path: &Path) -> EraserResult<u64> {
        let wide = to_wide(path);
        let mut find_data: WIN32_FIND_DATAW = unsafe { std::mem::zeroed() };
        let h_find = unsafe { FindFirstFileW(wide.as_ptr(), &mut find_data) };
        if h_find == INVALID_HANDLE_VALUE {
            let code = unsafe { GetLastError() };
            return Err(EraserError::TargetInaccessible {
                path: path.to_path_buf(),
                reason: format!("FindFirstFileW for size failed: code {}", code),
            });
        }
        unsafe { FindClose(h_find) };
        let size = ((find_data.nFileSizeHigh as u64) << 32) | (find_data.nFileSizeLow as u64);
        Ok(size)
    }

    fn query_allocated_size(&self, path: &Path) -> EraserResult<u64> {
        let wide = to_wide(path);
        let mut high: u32 = 0;
        let low = unsafe { GetCompressedFileSizeW(wide.as_ptr(), &mut high) };
        if low == INVALID_FILE_SIZE {
            let code = unsafe { GetLastError() };
            if code != 0 {
                return Err(EraserError::IoError {
                    path: path.to_path_buf(),
                    message: format!("GetCompressedFileSizeW failed: {}", code),
                });
            }
        }
        Ok(((high as u64) << 32) | (low as u64))
    }

    fn query_file_identity(&self, path: &Path) -> EraserResult<FileIdentityToken> {
        let wide = to_wide(path);
        // STRICT READ-ONLY: 0 access flags (query metadata only), null template handle.
        let handle = unsafe {
            CreateFileW(
                wide.as_ptr(),
                0,
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                std::ptr::null(),
                OPEN_EXISTING,
                FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT,
                std::ptr::null_mut(),
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            let code = unsafe { GetLastError() };
            return Err(EraserError::IoError {
                path: path.to_path_buf(),
                message: format!("CreateFileW query identity failed: {}", code),
            });
        }

        let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
        let ok = unsafe { GetFileInformationByHandle(handle, &mut info) };
        unsafe { CloseHandle(handle) };

        if ok == 0 {
            let code = unsafe { GetLastError() };
            return Err(EraserError::IoError {
                path: path.to_path_buf(),
                message: format!("GetFileInformationByHandle failed: {}", code),
            });
        }

        let file_index = ((info.nFileIndexHigh as u64) << 32) | (info.nFileIndexLow as u64);
        Ok(FileIdentityToken {
            volume_serial_number: info.dwVolumeSerialNumber as u64,
            file_index,
        })
    }

    fn query_file_attributes(&self, path: &Path) -> EraserResult<PlatformFileAttributes> {
        let wide = to_wide(path);
        let attrs = unsafe { GetFileAttributesW(wide.as_ptr()) };
        if attrs == INVALID_FILE_ATTRIBUTES {
            let code = unsafe { GetLastError() };
            return Err(EraserError::TargetInaccessible {
                path: path.to_path_buf(),
                reason: format!("GetFileAttributesW failed: {}", code),
            });
        }
        Ok(PlatformFileAttributes {
            is_sparse: (attrs & FILE_ATTRIBUTE_SPARSE_FILE) != 0,
            is_compressed: (attrs & FILE_ATTRIBUTE_COMPRESSED) != 0,
            is_encrypted: (attrs & FILE_ATTRIBUTE_ENCRYPTED) != 0,
            is_reparse_point: (attrs & FILE_ATTRIBUTE_REPARSE_POINT) != 0,
            is_directory: (attrs & FILE_ATTRIBUTE_DIRECTORY) != 0,
        })
    }

    fn query_alternate_data_streams(
        &self,
        path: &Path,
    ) -> EraserResult<Vec<AlternateDataStreamInfo>> {
        let wide = to_wide(path);
        let mut find_stream_data: WIN32_FIND_STREAM_DATA = unsafe { std::mem::zeroed() };
        let h_find = unsafe {
            FindFirstStreamW(
                wide.as_ptr(),
                FindStreamInfoStandard,
                &mut find_stream_data as *mut _ as *mut _,
                0,
            )
        };

        if h_find == INVALID_HANDLE_VALUE {
            let code = unsafe { GetLastError() };
            if code == ERROR_FILE_NOT_FOUND || code == 38 {
                // No secondary streams or stream enumeration not supported
                return Ok(Vec::new());
            }
            return Err(EraserError::IoError {
                path: path.to_path_buf(),
                message: format!("FindFirstStreamW failed with OS error {}", code),
            });
        }

        let mut streams = Vec::new();
        loop {
            let len = find_stream_data
                .cStreamName
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(find_stream_data.cStreamName.len());
            let s_name = OsString::from_wide(&find_stream_data.cStreamName[..len])
                .to_string_lossy()
                .to_string();

            if !s_name.is_empty() && s_name != "::$DATA" {
                streams.push(AlternateDataStreamInfo {
                    stream_name: s_name,
                    stream_size_bytes: find_stream_data.StreamSize as u64,
                });
            }

            let next_ok =
                unsafe { FindNextStreamW(h_find, &mut find_stream_data as *mut _ as *mut _) };
            if next_ok == 0 {
                break;
            }
        }
        unsafe { FindClose(h_find) };
        Ok(streams)
    }

    fn query_volume_info(&self, path: &Path) -> EraserResult<VolumeProbeData> {
        let wide = to_wide(path);
        let mut volume_path = vec![0u16; 1024];
        let ok = unsafe {
            GetVolumePathNameW(
                wide.as_ptr(),
                volume_path.as_mut_ptr(),
                volume_path.len() as u32,
            )
        };
        if ok == 0 {
            let code = unsafe { GetLastError() };
            return Err(EraserError::VolumeQueryFailed {
                path: path.to_path_buf(),
                reason: format!("GetVolumePathNameW failed: {}", code),
            });
        }

        let mut fs_flags: u32 = 0;
        let mut fs_name_buf = vec![0u16; 256];
        let ok_info = unsafe {
            GetVolumeInformationW(
                volume_path.as_ptr(),
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut fs_flags,
                fs_name_buf.as_mut_ptr(),
                fs_name_buf.len() as u32,
            )
        };
        if ok_info == 0 {
            let code = unsafe { GetLastError() };
            return Err(EraserError::VolumeQueryFailed {
                path: path.to_path_buf(),
                reason: format!("GetVolumeInformationW failed: {}", code),
            });
        }

        let len = fs_name_buf
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(fs_name_buf.len());
        let fs_name = OsString::from_wide(&fs_name_buf[..len])
            .to_string_lossy()
            .to_string();

        let mut sectors_per_cluster: u32 = 0;
        let mut bytes_per_sector: u32 = 0;
        let mut free_clusters: u32 = 0;
        let mut total_clusters: u32 = 0;

        let cluster_result = unsafe {
            GetDiskFreeSpaceW(
                volume_path.as_ptr(),
                &mut sectors_per_cluster,
                &mut bytes_per_sector,
                &mut free_clusters,
                &mut total_clusters,
            )
        };

        let (sector_size_bytes, cluster_size_bytes) = if cluster_result != 0 {
            (
                ProbedValue::Verified(bytes_per_sector),
                ProbedValue::Verified(sectors_per_cluster * bytes_per_sector),
            )
        } else {
            let code = unsafe { GetLastError() };
            (
                ProbedValue::Unavailable {
                    reason: format!("GetDiskFreeSpaceW failed: {}", code),
                },
                ProbedValue::Unavailable {
                    reason: format!("GetDiskFreeSpaceW failed: {}", code),
                },
            )
        };

        let is_read_only = ProbedValue::Verified((fs_flags & FILE_READ_ONLY_VOLUME) != 0);

        Ok(VolumeProbeData {
            volume_guid_path: ProbedValue::Unavailable {
                reason: "Volume GUID lookup omitted to avoid elevated privileged volume opens"
                    .into(),
            },
            filesystem_name: ProbedValue::Verified(fs_name),
            sector_size_bytes,
            cluster_size_bytes,
            supports_sparse_files: ProbedValue::Verified(
                (fs_flags & FILE_SUPPORTS_SPARSE_FILES) != 0,
            ),
            supports_alternate_streams: ProbedValue::Verified((fs_flags & FILE_NAMED_STREAMS) != 0),
            supports_compression: ProbedValue::Verified((fs_flags & FILE_FILE_COMPRESSION) != 0),
            supports_encryption: ProbedValue::Verified((fs_flags & FILE_SUPPORTS_ENCRYPTION) != 0),
            is_read_only,
        })
    }

    fn get_protected_system_paths(&self) -> EraserResult<Vec<PathBuf>> {
        let mut paths = Vec::new();

        let win_dir = std::env::var_os("SystemRoot")
            .or_else(|| std::env::var_os("windir"))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(r"C:\Windows"));

        paths.push(win_dir.clone());
        paths.push(win_dir.join("System32"));
        paths.push(win_dir.join("SysWOW64"));
        paths.push(win_dir.join("WinSxS"));

        if let Some(prog_files) = std::env::var_os("ProgramFiles") {
            paths.push(PathBuf::from(prog_files));
        }
        if let Some(prog_files_x86) = std::env::var_os("ProgramFiles(x86)") {
            paths.push(PathBuf::from(prog_files_x86));
        }
        if let Some(prog_data) = std::env::var_os("ProgramData") {
            paths.push(PathBuf::from(prog_data));
        }

        Ok(paths)
    }

    fn read_dir_entries(&self, path: &Path) -> EraserResult<Vec<PathBuf>> {
        let search_pattern = path.join("*");
        let wide = to_wide(&search_pattern);
        let mut find_data: WIN32_FIND_DATAW = unsafe { std::mem::zeroed() };
        let h_find = unsafe { FindFirstFileW(wide.as_ptr(), &mut find_data) };
        if h_find == INVALID_HANDLE_VALUE {
            let code = unsafe { GetLastError() };
            return Err(EraserError::IoError {
                path: path.to_path_buf(),
                message: format!("FindFirstFileW error {}", code),
            });
        }

        let mut entries = Vec::new();
        loop {
            let len = find_data
                .cFileName
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(find_data.cFileName.len());
            let name = OsString::from_wide(&find_data.cFileName[..len]);
            if name != "." && name != ".." {
                entries.push(path.join(name));
            }

            let next_ok = unsafe { FindNextFileW(h_find, &mut find_data) };
            if next_ok == 0 {
                let code = unsafe { GetLastError() };
                if code == ERROR_NO_MORE_FILES {
                    break;
                }
                unsafe { FindClose(h_find) };
                return Err(EraserError::IoError {
                    path: path.to_path_buf(),
                    message: format!("FindNextFileW error {}", code),
                });
            }
        }
        unsafe { FindClose(h_find) };
        Ok(entries)
    }
}
