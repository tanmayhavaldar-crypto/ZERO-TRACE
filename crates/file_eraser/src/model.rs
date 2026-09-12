use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Epistemic certainty wrapper: prevents guessing unverified states.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", content = "data")]
pub enum ProbedValue<T> {
    Verified(T),
    Unavailable { reason: String },
}

impl<T> ProbedValue<T> {
    pub fn is_verified(&self) -> bool {
        matches!(self, ProbedValue::Verified(_))
    }

    pub fn value(&self) -> Option<&T> {
        match self {
            ProbedValue::Verified(v) => Some(v),
            ProbedValue::Unavailable { .. } => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetKind {
    RegularFile,
    Directory,
    SymlinkFile,
    SymlinkDirectory,
    Junction,
    OtherReparsePoint { tag_hex: String },
    UnknownReparsePoint { reason: String },
    SpecialDevice,
    Inaccessible { reason: String },
}

impl TargetKind {
    pub fn is_reparse_boundary(&self) -> bool {
        matches!(
            self,
            TargetKind::SymlinkFile
                | TargetKind::SymlinkDirectory
                | TargetKind::Junction
                | TargetKind::OtherReparsePoint { .. }
                | TargetKind::UnknownReparsePoint { .. }
        )
    }

    pub fn is_traversable_directory(&self) -> bool {
        matches!(self, TargetKind::Directory)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReparseTagType {
    None,
    Symlink,
    MountPointJunction,
    AppExecLink,
    WslSymlink,
    CloudPlaceholder,
    Unknown(u32),
    QueryFailed(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafetyClassification {
    SafeToAnalyze,
    ProtectedSystemPath { reason: String },
    BlockedReparsePoint { reason: String },
    BlockedDevicePath { reason: String },
    Inaccessible { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlternateDataStreamInfo {
    pub stream_name: String,
    pub stream_size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileIdentityToken {
    pub volume_serial_number: u64,
    pub file_index: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VolumeProbeData {
    pub volume_guid_path: ProbedValue<String>,
    pub filesystem_name: ProbedValue<String>,
    pub sector_size_bytes: ProbedValue<u32>,
    pub cluster_size_bytes: ProbedValue<u32>,
    pub supports_sparse_files: ProbedValue<bool>,
    pub supports_alternate_streams: ProbedValue<bool>,
    pub supports_compression: ProbedValue<bool>,
    pub supports_encryption: ProbedValue<bool>,
    pub is_read_only: ProbedValue<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetProbeData {
    pub original_path: PathBuf,
    pub normalized_path: PathBuf,
    pub kind: TargetKind,
    pub size_bytes: ProbedValue<u64>,
    pub allocated_size_bytes: ProbedValue<u64>,
    pub identity: ProbedValue<FileIdentityToken>,
    pub is_sparse: ProbedValue<bool>,
    pub is_compressed: ProbedValue<bool>,
    pub is_encrypted: ProbedValue<bool>,
    pub is_reparse_point: ProbedValue<bool>,
    pub reparse_tag: ProbedValue<ReparseTagType>,
    pub alternate_data_streams: ProbedValue<Vec<AlternateDataStreamInfo>>,
    pub volume_info: ProbedValue<VolumeProbeData>,
    pub safety: SafetyClassification,
}

#[derive(Debug, Clone, Default)]
pub struct ResolutionStatistics {
    pub total_input_targets: usize,
    pub unique_targets: usize,
    pub regular_files_discovered: usize,
    pub directories_discovered: usize,
    pub reparse_boundaries_encountered: usize,
    pub inaccessible_targets_encountered: usize,
    pub protected_targets_encountered: usize,
}
