//! Step 5.2 — Sanitization Method Selection Layer.
//!
//! # Strict Design Invariants
//! 1. Non-Destructive: Pure in-memory selection logic only. Performs NO filesystem writes,
//!    NO metadata alterations, NO stream deletions, NO file truncation, and NO overwriting.
//! 2. Epistemic Discipline: Methods are proposed ONLY when the corresponding capability
//!    state is `CapabilityState::Supported`. `Unsupported` and `Unknown` capability states
//!    NEVER produce an automatic method proposal.
//! 3. Deterministic Order: Proposed methods are ordered in a fixed sequence:
//!    `ContentOverwrite` -> `MetadataSanitization` -> `AlternateDataStreamSanitization` ->
//!    `SlackSpaceSanitization` -> `FreeSpaceSanitization`.
//! 4. Conservative Descriptions: Does not claim "NIST certification", physical NAND/flash
//!    erasure guarantees, or universal recovery prevention across all media.
//! 5. Safety Gating: If `assessment.safety` is anything other than `SafeToAnalyze`,
//!    the plan MUST contain zero proposed methods.

use crate::capability::{CapabilityState, FileCapabilityAssessment};
use crate::model::SafetyClassification;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Strongly typed file sanitization methods that ForenX can evaluate and propose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FileSanitizationMethod {
    /// Host-level pattern overwrite of target file allocation content.
    ContentOverwrite,
    /// Reset/scrub of standard file metadata attributes and timestamps.
    MetadataSanitization,
    /// Clearing and truncation of secondary alternate data streams.
    AlternateDataStreamSanitization,
    /// Clearing of residual slack bytes within the file's terminal allocation cluster.
    SlackSpaceSanitization,
    /// Scrubbing of volume-wide unallocated free space.
    FreeSpaceSanitization,
}

impl FileSanitizationMethod {
    /// Human-readable display label for presentation and audit logging.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::ContentOverwrite => "Content Overwrite",
            Self::MetadataSanitization => "Metadata Sanitization",
            Self::AlternateDataStreamSanitization => "Alternate Data Stream Sanitization",
            Self::SlackSpaceSanitization => "Slack Space Sanitization",
            Self::FreeSpaceSanitization => "Free Space Sanitization",
        }
    }

    /// Technical mechanism summary describing the method without overclaiming physical guarantees.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::ContentOverwrite => {
                "Host-level pattern overwrite pass; physical media-level sanitization guarantees are outside this file-target selection layer."
            }
            Self::MetadataSanitization => {
                "Host-level resetting of standard filesystem attributes and timestamps; does not guarantee removal of OS journaling, shadow copies, or backup artifacts."
            }
            Self::AlternateDataStreamSanitization => {
                "Host-level clearing and truncation of secondary alternate data streams on supported filesystems."
            }
            Self::SlackSpaceSanitization => {
                "Host-level clearing of residual slack bytes within the terminal allocated cluster; requires explicit driver support."
            }
            Self::FreeSpaceSanitization => {
                "Volume-wide unallocated space pass; outside the scope of individual file target handles."
            }
        }
    }
}

/// A proposed sanitization method accompanied by the rationale and supporting capability state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MethodProposal {
    pub method: FileSanitizationMethod,
    pub label: String,
    pub rationale: String,
    pub capability_state: CapabilityState,
}

/// The complete sanitization proposal plan for a specific file target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileSanitizationPlan {
    pub target_path: PathBuf,
    pub safety: SafetyClassification,
    pub proposed_methods: Vec<MethodProposal>,
    pub summary: String,
}

impl FileSanitizationPlan {
    /// Returns true if at least one sanitization method is proposed.
    #[must_use]
    pub fn has_proposals(&self) -> bool {
        !self.proposed_methods.is_empty()
    }
}

/// Evaluates a `FileCapabilityAssessment` and produces a deterministic `FileSanitizationPlan`.
///
/// # Selection Invariants
/// - If `safety` is not `SafeToAnalyze`, zero methods are proposed.
/// - `ContentOverwrite` is proposed iff `content_sanitization == Supported`.
/// - `MetadataSanitization` is proposed iff `metadata_sanitization == Supported`.
/// - `AlternateDataStreamSanitization` is proposed iff `alternate_data_streams == Supported`.
/// - `SlackSpaceSanitization` is proposed iff `slack_space == Supported`.
/// - `FreeSpaceSanitization` is proposed iff `free_space == Supported`.
/// - `CapabilityState::Unsupported` and `CapabilityState::Unknown` NEVER produce proposals.
#[must_use]
pub fn plan_target_sanitization(assessment: &FileCapabilityAssessment) -> FileSanitizationPlan {
    let mut proposals = Vec::new();

    // 1. Safety barrier: If safety is anything other than SafeToAnalyze, return an empty plan.
    if !matches!(assessment.safety, SafetyClassification::SafeToAnalyze) {
        return FileSanitizationPlan {
            target_path: assessment.target_path.clone(),
            safety: assessment.safety.clone(),
            proposed_methods: Vec::new(),
            summary: "No sanitization methods proposed: target is restricted by safety policy or is inaccessible."
                .into(),
        };
    }

    // 2. Fixed deterministic order evaluation:
    // Slot 1: Content Overwrite
    if assessment.content_sanitization.state == CapabilityState::Supported {
        proposals.push(MethodProposal {
            method: FileSanitizationMethod::ContentOverwrite,
            label: FileSanitizationMethod::ContentOverwrite.label().to_string(),
            rationale: format!(
                "Content sanitization verified supported: {}",
                assessment.content_sanitization.reason
            ),
            capability_state: CapabilityState::Supported,
        });
    }

    // Slot 2: Metadata Sanitization
    if assessment.metadata_sanitization.state == CapabilityState::Supported {
        proposals.push(MethodProposal {
            method: FileSanitizationMethod::MetadataSanitization,
            label: FileSanitizationMethod::MetadataSanitization.label().to_string(),
            rationale: format!(
                "Metadata sanitization verified supported: {}",
                assessment.metadata_sanitization.reason
            ),
            capability_state: CapabilityState::Supported,
        });
    }

    // Slot 3: Alternate Data Streams
    if assessment.alternate_data_streams.state == CapabilityState::Supported {
        proposals.push(MethodProposal {
            method: FileSanitizationMethod::AlternateDataStreamSanitization,
            label: FileSanitizationMethod::AlternateDataStreamSanitization
                .label()
                .to_string(),
            rationale: format!(
                "Alternate data stream capability verified supported: {}",
                assessment.alternate_data_streams.reason
            ),
            capability_state: CapabilityState::Supported,
        });
    }

    // Slot 4: Slack Space
    if assessment.slack_space.state == CapabilityState::Supported {
        proposals.push(MethodProposal {
            method: FileSanitizationMethod::SlackSpaceSanitization,
            label: FileSanitizationMethod::SlackSpaceSanitization.label().to_string(),
            rationale: format!(
                "Slack space capability verified supported: {}",
                assessment.slack_space.reason
            ),
            capability_state: CapabilityState::Supported,
        });
    }

    // Slot 5: Free Space
    if assessment.free_space.state == CapabilityState::Supported {
        proposals.push(MethodProposal {
            method: FileSanitizationMethod::FreeSpaceSanitization,
            label: FileSanitizationMethod::FreeSpaceSanitization.label().to_string(),
            rationale: format!(
                "Free space capability verified supported: {}",
                assessment.free_space.reason
            ),
            capability_state: CapabilityState::Supported,
        });
    }

    let summary = if proposals.is_empty() {
        "No sanitization methods proposed: no capability areas have verified supported status.".into()
    } else {
        format!(
            "Proposed {} sanitization method(s) based on verified capabilities.",
            proposals.len()
        )
    };

    FileSanitizationPlan {
        target_path: assessment.target_path.clone(),
        safety: assessment.safety.clone(),
        proposed_methods: proposals,
        summary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::{CapabilityFinding, CapabilityState, FileCapabilityAssessment};
    use crate::model::{SafetyClassification, TargetKind};

    fn make_test_assessment(
        content: CapabilityState,
        metadata: CapabilityState,
        ads: CapabilityState,
        slack: CapabilityState,
        free_space: CapabilityState,
        safety: SafetyClassification,
    ) -> FileCapabilityAssessment {
        FileCapabilityAssessment {
            target_path: PathBuf::from(r"D:\Data\sample.txt"),
            kind: TargetKind::RegularFile,
            safety,
            content_sanitization: CapabilityFinding {
                state: content,
                reason: "Content rationale".into(),
            },
            metadata_sanitization: CapabilityFinding {
                state: metadata,
                reason: "Metadata rationale".into(),
            },
            alternate_data_streams: CapabilityFinding {
                state: ads,
                reason: "ADS rationale".into(),
            },
            slack_space: CapabilityFinding {
                state: slack,
                reason: "Slack rationale".into(),
            },
            free_space: CapabilityFinding {
                state: free_space,
                reason: "Free space rationale".into(),
            },
        }
    }

    #[test]
    fn test_1_safe_target_all_unknown_yields_zero_proposals() {
        let assessment = make_test_assessment(
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            SafetyClassification::SafeToAnalyze,
        );

        let plan = plan_target_sanitization(&assessment);
        assert!(!plan.has_proposals());
        assert!(plan.proposed_methods.is_empty());
        assert!(plan.summary.contains("No sanitization methods proposed"));
    }

    #[test]
    fn test_2_content_supported_proposes_content_overwrite() {
        let assessment = make_test_assessment(
            CapabilityState::Supported,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            SafetyClassification::SafeToAnalyze,
        );

        let plan = plan_target_sanitization(&assessment);
        assert_eq!(plan.proposed_methods.len(), 1);
        assert_eq!(
            plan.proposed_methods[0].method,
            FileSanitizationMethod::ContentOverwrite
        );
        assert_eq!(
            plan.proposed_methods[0].capability_state,
            CapabilityState::Supported
        );
    }

    #[test]
    fn test_3_content_unsupported_not_proposed() {
        let assessment = make_test_assessment(
            CapabilityState::Unsupported,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            SafetyClassification::SafeToAnalyze,
        );

        let plan = plan_target_sanitization(&assessment);
        assert!(!plan
            .proposed_methods
            .iter()
            .any(|m| m.method == FileSanitizationMethod::ContentOverwrite));
    }

    #[test]
    fn test_4_content_unknown_not_proposed() {
        let assessment = make_test_assessment(
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            SafetyClassification::SafeToAnalyze,
        );

        let plan = plan_target_sanitization(&assessment);
        assert!(!plan
            .proposed_methods
            .iter()
            .any(|m| m.method == FileSanitizationMethod::ContentOverwrite));
    }

    #[test]
    fn test_5_metadata_supported_proposes_metadata_sanitization() {
        let assessment = make_test_assessment(
            CapabilityState::Unknown,
            CapabilityState::Supported,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            SafetyClassification::SafeToAnalyze,
        );

        let plan = plan_target_sanitization(&assessment);
        assert_eq!(plan.proposed_methods.len(), 1);
        assert_eq!(
            plan.proposed_methods[0].method,
            FileSanitizationMethod::MetadataSanitization
        );
    }

    #[test]
    fn test_6_metadata_unsupported_not_proposed() {
        let assessment = make_test_assessment(
            CapabilityState::Unknown,
            CapabilityState::Unsupported,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            SafetyClassification::SafeToAnalyze,
        );

        let plan = plan_target_sanitization(&assessment);
        assert!(!plan
            .proposed_methods
            .iter()
            .any(|m| m.method == FileSanitizationMethod::MetadataSanitization));
    }

    #[test]
    fn test_7_metadata_unknown_not_proposed() {
        let assessment = make_test_assessment(
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            SafetyClassification::SafeToAnalyze,
        );

        let plan = plan_target_sanitization(&assessment);
        assert!(!plan
            .proposed_methods
            .iter()
            .any(|m| m.method == FileSanitizationMethod::MetadataSanitization));
    }

    #[test]
    fn test_8_ads_supported_proposes_ads_method() {
        let assessment = make_test_assessment(
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Supported,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            SafetyClassification::SafeToAnalyze,
        );

        let plan = plan_target_sanitization(&assessment);
        assert_eq!(plan.proposed_methods.len(), 1);
        assert_eq!(
            plan.proposed_methods[0].method,
            FileSanitizationMethod::AlternateDataStreamSanitization
        );
    }

    #[test]
    fn test_9_ads_unsupported_not_proposed() {
        let assessment = make_test_assessment(
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unsupported,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            SafetyClassification::SafeToAnalyze,
        );

        let plan = plan_target_sanitization(&assessment);
        assert!(!plan
            .proposed_methods
            .iter()
            .any(|m| m.method == FileSanitizationMethod::AlternateDataStreamSanitization));
    }

    #[test]
    fn test_10_ads_unknown_not_proposed() {
        let assessment = make_test_assessment(
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            SafetyClassification::SafeToAnalyze,
        );

        let plan = plan_target_sanitization(&assessment);
        assert!(!plan
            .proposed_methods
            .iter()
            .any(|m| m.method == FileSanitizationMethod::AlternateDataStreamSanitization));
    }

    #[test]
    fn test_11_slack_supported_proposes_slack_method() {
        let assessment = make_test_assessment(
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Supported,
            CapabilityState::Unknown,
            SafetyClassification::SafeToAnalyze,
        );

        let plan = plan_target_sanitization(&assessment);
        assert_eq!(plan.proposed_methods.len(), 1);
        assert_eq!(
            plan.proposed_methods[0].method,
            FileSanitizationMethod::SlackSpaceSanitization
        );
    }

    #[test]
    fn test_12_slack_unsupported_not_proposed() {
        let assessment = make_test_assessment(
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unsupported,
            CapabilityState::Unknown,
            SafetyClassification::SafeToAnalyze,
        );

        let plan = plan_target_sanitization(&assessment);
        assert!(!plan
            .proposed_methods
            .iter()
            .any(|m| m.method == FileSanitizationMethod::SlackSpaceSanitization));
    }

    #[test]
    fn test_13_slack_unknown_not_proposed() {
        let assessment = make_test_assessment(
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            SafetyClassification::SafeToAnalyze,
        );

        let plan = plan_target_sanitization(&assessment);
        assert!(!plan
            .proposed_methods
            .iter()
            .any(|m| m.method == FileSanitizationMethod::SlackSpaceSanitization));
    }

    #[test]
    fn test_14_free_space_supported_proposes_free_space_method() {
        let assessment = make_test_assessment(
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Supported,
            SafetyClassification::SafeToAnalyze,
        );

        let plan = plan_target_sanitization(&assessment);
        assert_eq!(plan.proposed_methods.len(), 1);
        assert_eq!(
            plan.proposed_methods[0].method,
            FileSanitizationMethod::FreeSpaceSanitization
        );
    }

    #[test]
    fn test_15_free_space_unsupported_not_proposed() {
        let assessment = make_test_assessment(
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unsupported,
            SafetyClassification::SafeToAnalyze,
        );

        let plan = plan_target_sanitization(&assessment);
        assert!(!plan
            .proposed_methods
            .iter()
            .any(|m| m.method == FileSanitizationMethod::FreeSpaceSanitization));
    }

    #[test]
    fn test_16_free_space_unknown_not_proposed() {
        let assessment = make_test_assessment(
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            SafetyClassification::SafeToAnalyze,
        );

        let plan = plan_target_sanitization(&assessment);
        assert!(!plan
            .proposed_methods
            .iter()
            .any(|m| m.method == FileSanitizationMethod::FreeSpaceSanitization));
    }

    #[test]
    fn test_17_unsafe_protected_target_yields_zero_proposals() {
        let assessment = make_test_assessment(
            CapabilityState::Supported,
            CapabilityState::Supported,
            CapabilityState::Supported,
            CapabilityState::Supported,
            CapabilityState::Supported,
            SafetyClassification::ProtectedSystemPath {
                reason: "System directory".into(),
            },
        );

        let plan = plan_target_sanitization(&assessment);
        assert!(plan.proposed_methods.is_empty());
        assert!(!plan.has_proposals());
        assert!(plan.summary.contains("safety policy"));
    }

    #[test]
    fn test_18_reparse_blocked_target_yields_zero_proposals() {
        let assessment = make_test_assessment(
            CapabilityState::Supported,
            CapabilityState::Supported,
            CapabilityState::Supported,
            CapabilityState::Supported,
            CapabilityState::Supported,
            SafetyClassification::BlockedReparsePoint {
                reason: "Reparse boundary".into(),
            },
        );

        let plan = plan_target_sanitization(&assessment);
        assert!(plan.proposed_methods.is_empty());
        assert!(!plan.has_proposals());
    }

    #[test]
    fn test_19_inaccessible_target_yields_zero_proposals() {
        let assessment = make_test_assessment(
            CapabilityState::Supported,
            CapabilityState::Supported,
            CapabilityState::Supported,
            CapabilityState::Supported,
            CapabilityState::Supported,
            SafetyClassification::Inaccessible {
                reason: "Cannot access file".into(),
            },
        );

        let plan = plan_target_sanitization(&assessment);
        assert!(plan.proposed_methods.is_empty());
        assert!(!plan.has_proposals());
    }

    #[test]
    fn test_20_multiple_supported_capabilities_exact_deterministic_order() {
        let assessment = make_test_assessment(
            CapabilityState::Supported,
            CapabilityState::Supported,
            CapabilityState::Supported,
            CapabilityState::Supported,
            CapabilityState::Supported,
            SafetyClassification::SafeToAnalyze,
        );

        let plan = plan_target_sanitization(&assessment);
        assert_eq!(plan.proposed_methods.len(), 5);

        assert_eq!(
            plan.proposed_methods[0].method,
            FileSanitizationMethod::ContentOverwrite
        );
        assert_eq!(
            plan.proposed_methods[1].method,
            FileSanitizationMethod::MetadataSanitization
        );
        assert_eq!(
            plan.proposed_methods[2].method,
            FileSanitizationMethod::AlternateDataStreamSanitization
        );
        assert_eq!(
            plan.proposed_methods[3].method,
            FileSanitizationMethod::SlackSpaceSanitization
        );
        assert_eq!(
            plan.proposed_methods[4].method,
            FileSanitizationMethod::FreeSpaceSanitization
        );
    }

    #[test]
    fn test_21_repeated_identical_input_yields_identical_output() {
        let assessment1 = make_test_assessment(
            CapabilityState::Supported,
            CapabilityState::Supported,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            SafetyClassification::SafeToAnalyze,
        );
        let assessment2 = assessment1.clone();

        let plan1 = plan_target_sanitization(&assessment1);
        let plan2 = plan_target_sanitization(&assessment2);

        assert_eq!(plan1, plan2);
    }

    #[test]
    fn test_22_human_readable_labels_and_descriptions_are_deterministic() {
        let methods = [
            FileSanitizationMethod::ContentOverwrite,
            FileSanitizationMethod::MetadataSanitization,
            FileSanitizationMethod::AlternateDataStreamSanitization,
            FileSanitizationMethod::SlackSpaceSanitization,
            FileSanitizationMethod::FreeSpaceSanitization,
        ];

        for m in methods {
            let label = m.label();
            let desc = m.description();

            assert!(!label.is_empty());
            assert!(!desc.is_empty());

            assert!(!desc.contains("NIST certified"));
            assert!(!desc.contains("guaranteed physical erasure"));
            assert!(!desc.contains("universally secure"));
        }
    }
}