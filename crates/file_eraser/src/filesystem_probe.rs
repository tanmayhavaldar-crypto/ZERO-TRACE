use crate::errors::EraserResult;
use crate::model::*;
use crate::platform::PlatformProvider;
use crate::safety::SafetyEngine;
use std::path::Path;

pub struct FilesystemProber<P: PlatformProvider> {
    platform: P,
    safety_engine: SafetyEngine<P>,
}

impl<P: PlatformProvider + Clone> FilesystemProber<P> {
    pub fn new(platform: P) -> EraserResult<Self> {
        let safety_engine = SafetyEngine::new(platform.clone())?;
        Ok(Self {
            platform,
            safety_engine,
        })
    }

    /// Performs thorough, non-destructive, read-only probing of a single target path.
    pub fn probe_target(&self, path: &Path) -> EraserResult<TargetProbeData> {
        let normalized = self
            .platform
            .normalize_path(path)
            .unwrap_or_else(|_| path.to_path_buf());

        let (kind, reparse_tag_result) = match self.platform.query_target_kind(&normalized) {
            Ok(k) => {
                if k.is_reparse_boundary() {
                    match self.platform.query_reparse_tag(&normalized) {
                        Ok(tag) => (k, ProbedValue::Verified(tag)),
                        Err(e) => (
                            TargetKind::UnknownReparsePoint {
                                reason: format!(
                                    "Reparse point detected but tag query failed: {}",
                                    e
                                ),
                            },
                            ProbedValue::Unavailable {
                                reason: format!("Failed to query reparse tag: {}", e),
                            },
                        ),
                    }
                } else {
                    (k, ProbedValue::Verified(ReparseTagType::None))
                }
            }
            Err(e) => (
                TargetKind::Inaccessible {
                    reason: e.to_string(),
                },
                ProbedValue::Unavailable {
                    reason: e.to_string(),
                },
            ),
        };

        let safety = self.safety_engine.classify_target(&normalized, &kind);

        let size_bytes = match self.platform.query_file_size(&normalized) {
            Ok(s) => ProbedValue::Verified(s),
            Err(e) => ProbedValue::Unavailable {
                reason: format!("File size query failed: {}", e),
            },
        };

        let allocated_size_bytes = match self.platform.query_allocated_size(&normalized) {
            Ok(s) => ProbedValue::Verified(s),
            Err(e) => ProbedValue::Unavailable {
                reason: format!("Allocated size query failed: {}", e),
            },
        };

        let identity = match self.platform.query_file_identity(&normalized) {
            Ok(id) => ProbedValue::Verified(id),
            Err(e) => ProbedValue::Unavailable {
                reason: format!("File identity query failed: {}", e),
            },
        };

        let attributes = self.platform.query_file_attributes(&normalized);
        let (is_sparse, is_compressed, is_encrypted, is_reparse_point) = match attributes {
            Ok(attrs) => (
                ProbedValue::Verified(attrs.is_sparse),
                ProbedValue::Verified(attrs.is_compressed),
                ProbedValue::Verified(attrs.is_encrypted),
                ProbedValue::Verified(attrs.is_reparse_point),
            ),
            Err(e) => (
                ProbedValue::Unavailable {
                    reason: format!("Attributes query failed: {}", e),
                },
                ProbedValue::Unavailable {
                    reason: format!("Attributes query failed: {}", e),
                },
                ProbedValue::Unavailable {
                    reason: format!("Attributes query failed: {}", e),
                },
                ProbedValue::Unavailable {
                    reason: format!("Attributes query failed: {}", e),
                },
            ),
        };

        let alternate_data_streams = match self.platform.query_alternate_data_streams(&normalized) {
            Ok(streams) => ProbedValue::Verified(streams),
            Err(e) => ProbedValue::Unavailable {
                reason: format!("Alternate streams query failed: {}", e),
            },
        };

        let volume_info = match self.platform.query_volume_info(&normalized) {
            Ok(vol) => ProbedValue::Verified(vol),
            Err(e) => ProbedValue::Unavailable {
                reason: format!("Volume information query failed: {}", e),
            },
        };

        Ok(TargetProbeData {
            original_path: path.to_path_buf(),
            normalized_path: normalized,
            kind,
            size_bytes,
            allocated_size_bytes,
            identity,
            is_sparse,
            is_compressed,
            is_encrypted,
            is_reparse_point,
            reparse_tag: reparse_tag_result,
            alternate_data_streams,
            volume_info,
            safety,
        })
    }
}
