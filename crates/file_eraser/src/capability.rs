//! Step 5.1 — File Sanitization Capability Assessment Layer.
//!
//! # Strict Design Invariants
//! 1. Non-Destructive: Pure in-memory assessment logic only. Performs NO filesystem writes,
//!    NO metadata alterations, NO stream deletions, NO file truncation, and NO slack/free-space wiping.
//! 2. Epistemic Discipline:
//!    - `CapabilityState::Supported`: Positive, verified evidence establishes that a sanitization
//!      primitive is authoritatively supported.
//!    - `CapabilityState::Unsupported`: Negative verified evidence or safety constraints confirm
//!      that the operation cannot proceed.
//!    - `CapabilityState::Unknown`: Evidence is absent, unprobed, or only ordinary filesystem
//!      presence is known; secure sanitization support MUST NOT be assumed or guessed.
//! 3. Content Sanitization Constraint: Being an accessible regular file on a writable volume
//!    establishes eligibility for analysis and ordinary modification, but does NOT constitute
//!    authoritative evidence that secure content sanitization primitives are supported.
//! 4. Safety Enforcement: Targets classified under `SafetyClassification::ProtectedSystemPath`,
//!    `BlockedReparsePoint`, `BlockedDevicePath`, or `Inaccessible` are strictly `Unsupported`.
//! 5. Separation of Concerns: Assesses capabilities only. Does not select methods or execute operations.

use crate::model::{
    ProbedValue, SafetyClassification, TargetKind, TargetProbeData, VolumeProbeData,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// TriState capability status reflecting epistemic certainty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CapabilityState {
    /// Explicit evidence establishes that the sanitization operation is supported.
    Supported,
    /// Explicit evidence or safety constraints confirm the operation cannot proceed.
    Unsupported,
    /// Evidence is absent, unprobed, or insufficient to establish secure sanitization support.
    Unknown,
}

impl CapabilityState {
    #[must_use]
    pub const fn is_supported(&self) -> bool {
        matches!(self, Self::Supported)
    }

    #[must_use]
    pub const fn is_unsupported(&self) -> bool {
        matches!(self, Self::Unsupported)
    }

    #[must_use]
    pub const fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }
}

/// A capability evaluation outcome accompanied by auditable diagnostic reasoning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityFinding {
    pub state: CapabilityState,
    pub reason: String,
}

/// Full capability assessment for a single target based on Step 4 probe evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileCapabilityAssessment {
    pub target_path: PathBuf,
    pub kind: TargetKind,
    pub safety: SafetyClassification,
    pub content_sanitization: CapabilityFinding,
    pub metadata_sanitization: CapabilityFinding,
    pub alternate_data_streams: CapabilityFinding,
    pub slack_space: CapabilityFinding,
    pub free_space: CapabilityFinding,
}

/// Evaluates file sanitization capabilities deterministically from existing `TargetProbeData`.
#[must_use]
pub fn assess_target_capabilities(probe: &TargetProbeData) -> FileCapabilityAssessment {
    let content = assess_content_sanitization(probe);
    let metadata = assess_metadata_sanitization(probe);
    let ads = assess_ads_capability(probe);
    let slack = assess_slack_space_capability(probe);
    let free_space = assess_free_space_capability(probe);

    FileCapabilityAssessment {
        target_path: probe.normalized_path.clone(),
        kind: probe.kind.clone(),
        safety: probe.safety.clone(),
        content_sanitization: content,
        metadata_sanitization: metadata,
        alternate_data_streams: ads,
        slack_space: slack,
        free_space,
    }
}

fn assess_content_sanitization(probe: &TargetProbeData) -> CapabilityFinding {
    match &probe.safety {
        SafetyClassification::ProtectedSystemPath { reason } => {
            return CapabilityFinding {
                state: CapabilityState::Unsupported,
                reason: format!("Target is within a protected system directory: {reason}"),
            };
        }
        SafetyClassification::BlockedReparsePoint { reason } => {
            return CapabilityFinding {
                state: CapabilityState::Unsupported,
                reason: format!("Target is a blocked reparse boundary: {reason}"),
            };
        }
        SafetyClassification::BlockedDevicePath { reason } => {
            return CapabilityFinding {
                state: CapabilityState::Unsupported,
                reason: format!("Target is a blocked raw device path: {reason}"),
            };
        }
        SafetyClassification::Inaccessible { reason } => {
            return CapabilityFinding {
                state: CapabilityState::Unsupported,
                reason: format!("Target is inaccessible: {reason}"),
            };
        }
        SafetyClassification::SafeToAnalyze => {}
    }

    match &probe.kind {
        TargetKind::RegularFile => {
            if let ProbedValue::Verified(read_only) = probe
                .volume_info
                .value()
                .map(|v| &v.is_read_only)
                .unwrap_or(&ProbedValue::Unavailable {
                    reason: "Volume probe missing".into(),
                })
            {
                if *read_only {
                    return CapabilityFinding {
                        state: CapabilityState::Unsupported,
                        reason: "Containing volume is explicitly marked read-only".into(),
                    };
                }
            }

            // Regular file on a writable volume with standard Step 4 probe evidence establishes
            // analysis eligibility and ordinary file access, but does NOT constitute authoritative
            // evidence of a secure file-content sanitization primitive. Under epistemic certainty,
            // this remains Unknown.
            CapabilityFinding {
                state: CapabilityState::Unknown,
                reason: "Regular file is safe to analyze, but secure content-sanitization capability is not established by standard filesystem probe evidence alone".into(),
            }
        }
        TargetKind::Directory => CapabilityFinding {
            state: CapabilityState::Unsupported,
            reason: "Target is a directory container, not a content data file".into(),
        },
        TargetKind::SymlinkFile
        | TargetKind::SymlinkDirectory
        | TargetKind::Junction
        | TargetKind::OtherReparsePoint { .. }
        | TargetKind::UnknownReparsePoint { .. } => CapabilityFinding {
            state: CapabilityState::Unsupported,
            reason: "Reparse point boundaries cannot receive regular file content sanitization"
                .into(),
        },
        TargetKind::SpecialDevice => CapabilityFinding {
            state: CapabilityState::Unsupported,
            reason: "Target is a special device path; file content sanitization not applicable"
                .into(),
        },
        TargetKind::Inaccessible { reason } => CapabilityFinding {
            state: CapabilityState::Unsupported,
            reason: format!("Target file is inaccessible: {reason}"),
        },
    }
}

fn assess_metadata_sanitization(probe: &TargetProbeData) -> CapabilityFinding {
    if !matches!(probe.safety, SafetyClassification::SafeToAnalyze) {
        return CapabilityFinding {
            state: CapabilityState::Unsupported,
            reason: "Safety classification prohibits target analysis/modification".into(),
        };
    }

    if !probe.identity.is_verified() {
        return CapabilityFinding {
            state: CapabilityState::Unknown,
            reason: "Target file identity token is unavailable for metadata tracking".into(),
        };
    }

    match &probe.volume_info {
        ProbedValue::Verified(vol) => {
            if let ProbedValue::Verified(true) = vol.is_read_only {
                return CapabilityFinding {
                    state: CapabilityState::Unsupported,
                    reason: "Containing volume is marked read-only; metadata cannot be modified"
                        .into(),
                };
            }
            CapabilityFinding {
                state: CapabilityState::Supported,
                reason: "File identity and writable volume metadata support verified".into(),
            }
        }
        ProbedValue::Unavailable { reason } => CapabilityFinding {
            state: CapabilityState::Unknown,
            reason: format!("Volume information is unavailable: {reason}"),
        },
    }
}

fn assess_ads_capability(probe: &TargetProbeData) -> CapabilityFinding {
    if !matches!(probe.safety, SafetyClassification::SafeToAnalyze) {
        return CapabilityFinding {
            state: CapabilityState::Unsupported,
            reason: "Safety classification prohibits stream processing".into(),
        };
    }

    let vol = match &probe.volume_info {
        ProbedValue::Verified(v) => v,
        ProbedValue::Unavailable { reason } => {
            return CapabilityFinding {
                state: CapabilityState::Unknown,
                reason: format!("Volume information unavailable to verify ADS capability: {reason}"),
            };
        }
    };

    match &vol.supports_alternate_streams {
        ProbedValue::Verified(true) => {
            let stream_info = match &probe.alternate_data_streams {
                ProbedValue::Verified(streams) => {
                    format!(
                        "Filesystem supports alternate data streams; {} secondary stream(s) observed",
                        streams.len()
                    )
                }
                ProbedValue::Unavailable { reason } => {
                    format!(
                        "Filesystem supports alternate data streams; stream probe unavailable: {reason}"
                    )
                }
            };
            CapabilityFinding {
                state: CapabilityState::Supported,
                reason: stream_info,
            }
        }
        ProbedValue::Verified(false) => CapabilityFinding {
            state: CapabilityState::Unsupported,
            reason: "Containing filesystem does not support alternate data streams".into(),
        },
        ProbedValue::Unavailable { reason } => CapabilityFinding {
            state: CapabilityState::Unknown,
            reason: format!("Filesystem ADS support flag is unavailable: {reason}"),
        },
    }
}

fn assess_slack_space_capability(probe: &TargetProbeData) -> CapabilityFinding {
    if !matches!(probe.safety, SafetyClassification::SafeToAnalyze) {
        return CapabilityFinding {
            state: CapabilityState::Unsupported,
            reason: "Safety classification prohibits slack-space analysis".into(),
        };
    }

    if !matches!(probe.kind, TargetKind::RegularFile) {
        return CapabilityFinding {
            state: CapabilityState::Unsupported,
            reason: "Slack space handling is only applicable to regular file content allocations"
                .into(),
        };
    }

    let cluster_ok = probe
        .volume_info
        .value()
        .and_then(|v: &VolumeProbeData| v.cluster_size_bytes.value())
        .is_some();
    let size_ok = probe.size_bytes.is_verified() && probe.allocated_size_bytes.is_verified();

    if cluster_ok && size_ok {
        CapabilityFinding {
            state: CapabilityState::Unknown,
            reason: "Cluster geometry is probed, but low-level slack-space sanitization primitives remain unverified on this filesystem".into(),
        }
    } else {
        CapabilityFinding {
            state: CapabilityState::Unknown,
            reason: "Insufficient cluster or allocation geometry probed to assess slack space"
                .into(),
        }
    }
}

fn assess_free_space_capability(probe: &TargetProbeData) -> CapabilityFinding {
    if !matches!(probe.safety, SafetyClassification::SafeToAnalyze) {
        return CapabilityFinding {
            state: CapabilityState::Unsupported,
            reason: "Safety classification prohibits volume-level analysis".into(),
        };
    }

    CapabilityFinding {
        state: CapabilityState::Unknown,
        reason: "Volume-wide free space sanitization primitives are not established by file-target probing"
            .into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;
    use std::path::PathBuf;

    fn make_valid_volume() -> VolumeProbeData {
        VolumeProbeData {
            volume_guid_path: ProbedValue::Verified(r"\\?\Volume{mock}\".into()),
            filesystem_name: ProbedValue::Verified("NTFS".into()),
            sector_size_bytes: ProbedValue::Verified(512),
            cluster_size_bytes: ProbedValue::Verified(4096),
            supports_sparse_files: ProbedValue::Verified(true),
            supports_alternate_streams: ProbedValue::Verified(true),
            supports_compression: ProbedValue::Verified(true),
            supports_encryption: ProbedValue::Verified(true),
            is_read_only: ProbedValue::Verified(false),
        }
    }

    fn make_regular_file_probe() -> TargetProbeData {
        TargetProbeData {
            original_path: PathBuf::from(r"D:\SafeData\report.txt"),
            normalized_path: PathBuf::from(r"D:\SafeData\report.txt"),
            kind: TargetKind::RegularFile,
            size_bytes: ProbedValue::Verified(2048),
            allocated_size_bytes: ProbedValue::Verified(4096),
            identity: ProbedValue::Verified(FileIdentityToken {
                volume_serial_number: 11111,
                file_index: 22222,
            }),
            is_sparse: ProbedValue::Verified(false),
            is_compressed: ProbedValue::Verified(false),
            is_encrypted: ProbedValue::Verified(false),
            is_reparse_point: ProbedValue::Verified(false),
            reparse_tag: ProbedValue::Verified(ReparseTagType::None),
            alternate_data_streams: ProbedValue::Verified(vec![AlternateDataStreamInfo {
                stream_name: ":Zone.Identifier:$DATA".into(),
                stream_size_bytes: 32,
            }]),
            volume_info: ProbedValue::Verified(make_valid_volume()),
            safety: SafetyClassification::SafeToAnalyze,
        }
    }

    #[test]
    fn test_regular_file_content_capability_remains_unknown() {
        let probe = make_regular_file_probe();
        let assessment = assess_target_capabilities(&probe);

        assert_eq!(
            assessment.content_sanitization.state,
            CapabilityState::Unknown
        );
        assert!(assessment
            .content_sanitization
            .reason
            .contains("not established by standard filesystem probe evidence alone"));
    }

    #[test]
    fn test_read_only_volume_marks_appropriate_capabilities_unsupported() {
        let mut probe = make_regular_file_probe();
        let mut vol = make_valid_volume();
        vol.is_read_only = ProbedValue::Verified(true);
        probe.volume_info = ProbedValue::Verified(vol);

        let assessment = assess_target_capabilities(&probe);

        assert_eq!(
            assessment.content_sanitization.state,
            CapabilityState::Unsupported
        );
        assert!(assessment.content_sanitization.reason.contains("read-only"));

        assert_eq!(
            assessment.metadata_sanitization.state,
            CapabilityState::Unsupported
        );
        assert!(assessment.metadata_sanitization.reason.contains("read-only"));
    }

    #[test]
    fn test_verified_metadata_prerequisites_produce_metadata_supported() {
        let probe = make_regular_file_probe();
        let assessment = assess_target_capabilities(&probe);

        assert_eq!(
            assessment.metadata_sanitization.state,
            CapabilityState::Supported
        );
        assert!(assessment.metadata_sanitization.reason.contains("verified"));
    }

    #[test]
    fn test_missing_identity_produces_metadata_unknown() {
        let mut probe = make_regular_file_probe();
        probe.identity = ProbedValue::Unavailable {
            reason: "Access denied querying file index".into(),
        };

        let assessment = assess_target_capabilities(&probe);

        assert_eq!(
            assessment.metadata_sanitization.state,
            CapabilityState::Unknown
        );
        assert!(assessment
            .metadata_sanitization
            .reason
            .contains("token is unavailable"));
    }

    #[test]
    fn test_ads_supported_evidence() {
        let probe = make_regular_file_probe();
        let assessment = assess_target_capabilities(&probe);

        assert_eq!(
            assessment.alternate_data_streams.state,
            CapabilityState::Supported
        );
        assert!(assessment
            .alternate_data_streams
            .reason
            .contains("1 secondary stream(s) observed"));
    }

    #[test]
    fn test_ads_unsupported_evidence() {
        let mut probe = make_regular_file_probe();
        let mut vol = make_valid_volume();
        vol.supports_alternate_streams = ProbedValue::Verified(false);
        probe.volume_info = ProbedValue::Verified(vol);

        let assessment = assess_target_capabilities(&probe);

        assert_eq!(
            assessment.alternate_data_streams.state,
            CapabilityState::Unsupported
        );
        assert!(assessment
            .alternate_data_streams
            .reason
            .contains("does not support alternate data streams"));
    }

    #[test]
    fn test_ads_unknown_evidence() {
        let mut probe = make_regular_file_probe();
        let mut vol = make_valid_volume();
        vol.supports_alternate_streams = ProbedValue::Unavailable {
            reason: "Volume flags query failed".into(),
        };
        probe.volume_info = ProbedValue::Verified(vol);

        let assessment = assess_target_capabilities(&probe);

        assert_eq!(
            assessment.alternate_data_streams.state,
            CapabilityState::Unknown
        );
        assert!(assessment
            .alternate_data_streams
            .reason
            .contains("unavailable"));
    }

    #[test]
    fn test_reparse_blocked_target_prohibits_destructive_capabilities() {
        let mut probe = make_regular_file_probe();
        probe.kind = TargetKind::Junction;
        probe.safety = SafetyClassification::BlockedReparsePoint {
            reason: "Junction point boundary".into(),
        };

        let assessment = assess_target_capabilities(&probe);

        assert_eq!(
            assessment.content_sanitization.state,
            CapabilityState::Unsupported
        );
        assert_eq!(
            assessment.metadata_sanitization.state,
            CapabilityState::Unsupported
        );
        assert_eq!(
            assessment.alternate_data_streams.state,
            CapabilityState::Unsupported
        );
        assert_eq!(
            assessment.slack_space.state,
            CapabilityState::Unsupported
        );
        assert_eq!(
            assessment.free_space.state,
            CapabilityState::Unsupported
        );
    }

    #[test]
    fn test_inaccessible_target_prohibits_capabilities() {
        let mut probe = make_regular_file_probe();
        probe.kind = TargetKind::Inaccessible {
            reason: "File not found or locked".into(),
        };
        probe.safety = SafetyClassification::Inaccessible {
            reason: "File not found or locked".into(),
        };

        let assessment = assess_target_capabilities(&probe);

        assert_eq!(
            assessment.content_sanitization.state,
            CapabilityState::Unsupported
        );
        assert_eq!(
            assessment.metadata_sanitization.state,
            CapabilityState::Unsupported
        );
    }

    #[test]
    fn test_slack_space_capability_remains_unknown() {
        let probe = make_regular_file_probe();
        let assessment = assess_target_capabilities(&probe);

        assert_eq!(assessment.slack_space.state, CapabilityState::Unknown);
        assert!(assessment
            .slack_space
            .reason
            .contains("remain unverified"));
    }

    #[test]
    fn test_free_space_capability_remains_unknown() {
        let probe = make_regular_file_probe();
        let assessment = assess_target_capabilities(&probe);

        assert_eq!(assessment.free_space.state, CapabilityState::Unknown);
        assert!(assessment
            .free_space
            .reason
            .contains("not established by file-target probing"));
    }

    #[test]
    fn test_deterministic_reasons_and_results() {
        let p1 = make_regular_file_probe();
        let p2 = p1.clone();

        let a1 = assess_target_capabilities(&p1);
        let a2 = assess_target_capabilities(&p2);

        assert_eq!(a1, a2);
    }
}