use crate::errors::{EraserError, EraserResult};
use crate::model::*;
use crate::platform::{PlatformFileAttributes, PlatformProvider};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct MockEntryConfig {
    pub kind: TargetKind,
    pub size_bytes: u64,
    pub allocated_size: u64,
    pub identity: FileIdentityToken,
    pub attributes: PlatformFileAttributes,
    pub reparse_tag: Result<ReparseTagType, String>,
    pub streams: Vec<AlternateDataStreamInfo>,
    pub children: Vec<PathBuf>,
}

impl Default for MockEntryConfig {
    fn default() -> Self {
        Self {
            kind: TargetKind::RegularFile,
            size_bytes: 1024,
            allocated_size: 4096,
            identity: FileIdentityToken {
                volume_serial_number: 1,
                file_index: 100,
            },
            attributes: PlatformFileAttributes {
                is_sparse: false,
                is_compressed: false,
                is_encrypted: false,
                is_reparse_point: false,
                is_directory: false,
            },
            reparse_tag: Ok(ReparseTagType::None),
            streams: Vec::new(),
            children: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MockPlatformProvider {
    pub entries: HashMap<PathBuf, MockEntryConfig>,
    pub protected_paths: Vec<PathBuf>,
    pub volume_info_result: Result<VolumeProbeData, String>,
}

impl Default for MockPlatformProvider {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
            protected_paths: vec![
                PathBuf::from(r"C:\Windows"),
                PathBuf::from(r"C:\Windows\System32"),
                PathBuf::from(r"C:\Program Files"),
            ],
            volume_info_result: Ok(VolumeProbeData {
                volume_guid_path: ProbedValue::Unavailable {
                    reason: "Mock volume guid lookup unavailable".into(),
                },
                filesystem_name: ProbedValue::Verified("NTFS".into()),
                sector_size_bytes: ProbedValue::Verified(512),
                cluster_size_bytes: ProbedValue::Verified(4096),
                supports_sparse_files: ProbedValue::Verified(true),
                supports_alternate_streams: ProbedValue::Verified(true),
                supports_compression: ProbedValue::Verified(true),
                supports_encryption: ProbedValue::Verified(true),
                is_read_only: ProbedValue::Verified(false),
            }),
        }
    }
}

impl MockPlatformProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_entry(&mut self, path: PathBuf, config: MockEntryConfig) {
        self.entries.insert(path, config);
    }
}

impl PlatformProvider for MockPlatformProvider {
    fn normalize_path(&self, path: &Path) -> EraserResult<PathBuf> {
        let s = path.to_string_lossy().replace('/', "\\");
        Ok(PathBuf::from(s))
    }

    fn query_target_kind(&self, path: &Path) -> EraserResult<TargetKind> {
        self.entries
            .get(path)
            .map(|e| e.kind.clone())
            .ok_or_else(|| EraserError::TargetInaccessible {
                path: path.to_path_buf(),
                reason: "Mock path not found".into(),
            })
    }

    fn query_reparse_tag(&self, path: &Path) -> EraserResult<ReparseTagType> {
        let entry = self
            .entries
            .get(path)
            .ok_or_else(|| EraserError::TargetInaccessible {
                path: path.to_path_buf(),
                reason: "Mock path not found".into(),
            })?;
        match &entry.reparse_tag {
            Ok(tag) => Ok(tag.clone()),
            Err(err) => Err(EraserError::ReparseQueryFailed {
                path: path.to_path_buf(),
                reason: err.clone(),
            }),
        }
    }

    fn query_file_size(&self, path: &Path) -> EraserResult<u64> {
        self.entries.get(path).map(|e| e.size_bytes).ok_or_else(|| {
            EraserError::TargetInaccessible {
                path: path.to_path_buf(),
                reason: "Mock path not found".into(),
            }
        })
    }

    fn query_allocated_size(&self, path: &Path) -> EraserResult<u64> {
        self.entries
            .get(path)
            .map(|e| e.allocated_size)
            .ok_or_else(|| EraserError::TargetInaccessible {
                path: path.to_path_buf(),
                reason: "Mock path not found".into(),
            })
    }

    fn query_file_identity(&self, path: &Path) -> EraserResult<FileIdentityToken> {
        self.entries
            .get(path)
            .map(|e| e.identity.clone())
            .ok_or_else(|| EraserError::TargetInaccessible {
                path: path.to_path_buf(),
                reason: "Mock path not found".into(),
            })
    }

    fn query_file_attributes(&self, path: &Path) -> EraserResult<PlatformFileAttributes> {
        self.entries.get(path).map(|e| e.attributes).ok_or_else(|| {
            EraserError::TargetInaccessible {
                path: path.to_path_buf(),
                reason: "Mock path not found".into(),
            }
        })
    }

    fn query_alternate_data_streams(
        &self,
        path: &Path,
    ) -> EraserResult<Vec<AlternateDataStreamInfo>> {
        self.entries
            .get(path)
            .map(|e| e.streams.clone())
            .ok_or_else(|| EraserError::TargetInaccessible {
                path: path.to_path_buf(),
                reason: "Mock path not found".into(),
            })
    }

    fn query_volume_info(&self, path: &Path) -> EraserResult<VolumeProbeData> {
        self.volume_info_result
            .clone()
            .map_err(|e| EraserError::VolumeQueryFailed {
                path: path.to_path_buf(),
                reason: e,
            })
    }

    fn get_protected_system_paths(&self) -> EraserResult<Vec<PathBuf>> {
        Ok(self.protected_paths.clone())
    }

    fn read_dir_entries(&self, path: &Path) -> EraserResult<Vec<PathBuf>> {
        self.entries
            .get(path)
            .map(|e| e.children.clone())
            .ok_or_else(|| EraserError::IoError {
                path: path.to_path_buf(),
                message: "Mock directory read failed".into(),
            })
    }
}
