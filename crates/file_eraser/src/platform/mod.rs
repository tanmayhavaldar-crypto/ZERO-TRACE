pub mod mock;

#[cfg(windows)]
pub mod windows;

use crate::errors::EraserResult;
use crate::model::{
    AlternateDataStreamInfo, FileIdentityToken, ReparseTagType, TargetKind, VolumeProbeData,
};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlatformFileAttributes {
    pub is_sparse: bool,
    pub is_compressed: bool,
    pub is_encrypted: bool,
    pub is_reparse_point: bool,
    pub is_directory: bool,
}

/// Abstract platform interface to guarantee testability and strict read-only operation.
pub trait PlatformProvider: Send + Sync + 'static {
    fn normalize_path(&self, path: &Path) -> EraserResult<PathBuf>;
    fn query_target_kind(&self, path: &Path) -> EraserResult<TargetKind>;
    fn query_reparse_tag(&self, path: &Path) -> EraserResult<ReparseTagType>;
    fn query_file_size(&self, path: &Path) -> EraserResult<u64>;
    fn query_allocated_size(&self, path: &Path) -> EraserResult<u64>;
    fn query_file_identity(&self, path: &Path) -> EraserResult<FileIdentityToken>;
    fn query_file_attributes(&self, path: &Path) -> EraserResult<PlatformFileAttributes>;
    fn query_alternate_data_streams(
        &self,
        path: &Path,
    ) -> EraserResult<Vec<AlternateDataStreamInfo>>;
    fn query_volume_info(&self, path: &Path) -> EraserResult<VolumeProbeData>;
    fn get_protected_system_paths(&self) -> EraserResult<Vec<PathBuf>>;
    fn read_dir_entries(&self, path: &Path) -> EraserResult<Vec<PathBuf>>;
}
