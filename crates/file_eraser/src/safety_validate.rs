//! Step 5.3 — Final In-Memory Safety Validation Layer.
//!
//! # Strict Design Invariants
//! 1. Non-Destructive: Pure in-memory snapshot validation. Performs NO filesystem writes,
//!    NO deletions, NO truncations, NO IOCTL calls, and NO hardware access.
//! 2. Snapshot Scope: Validates the supplied assessment and plan data structures. It does
//!    NOT claim to re-open or physically revalidate the underlying target; pre-execution
//!    handle/identity revalidation remains the responsibility of future execution layers.
//! 3. Fail-Closed: If safety classifications are not `SafeToAnalyze`, paths mismatch,
//!    methods are out of order, duplicates exist, or methods lack verified `Supported`
//!    capability backing, the plan is BLOCKED.
//! 4. No Silent Repair: Invalid plans are never re-sorted, pruned, or mutated.
//! 5. Empty Plan Discipline: Plans with zero proposed methods are never authorized for execution
//!    (`has_executable_methods == false`).

use crate::capability::{CapabilityState, FileCapabilityAssessment};
use crate::method_select::{FileSanitizationMethod, FileSanitizationPlan, MethodProposal};
use crate::model::SafetyClassification;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

/// High-level verdict indicating whether a plan passed in-memory snapshot validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SafetyValidationStatus {
    /// The plan passed all consistency and safety snapshot rules.
    Allowed,
    /// The plan failed one or more safety rules and must not proceed.
    Blocked,
}

impl SafetyValidationStatus {
    #[must_use]
    pub const fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed)
    }

    #[must_use]
    pub const fn is_blocked(&self) -> bool {
        matches!(self, Self::Blocked)
    }
}

/// Comprehensive outcome of the in-memory safety validation evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafetyValidationResult {
    pub status: SafetyValidationStatus,
    pub target_path: PathBuf,
    pub reasons: Vec<String>,
    pub validated_method_count: usize,
    /// True ONLY if status is Allowed AND at least one executable method is validated.
    pub has_executable_methods: bool,
}

impl SafetyValidationResult {
    /// Returns true if the plan is both Allowed and contains executable methods.
    #[must_use]
    pub fn is_ready_for_execution(&self) -> bool {
        self.status == SafetyValidationStatus::Allowed && self.has_executable_methods
    }
}

/// Canonical ordering rank for sanitization methods.
const fn method_order_rank(method: FileSanitizationMethod) -> u8 {
    match method {
        FileSanitizationMethod::ContentOverwrite => 1,
        FileSanitizationMethod::MetadataSanitization => 2,
        FileSanitizationMethod::AlternateDataStreamSanitization => 3,
        FileSanitizationMethod::SlackSpaceSanitization => 4,
        FileSanitizationMethod::FreeSpaceSanitization => 5,
    }
}

/// Validates an in-memory `FileSanitizationPlan` against its supporting `FileCapabilityAssessment`.
#[must_use]
pub fn validate_sanitization_plan(
    assessment: &FileCapabilityAssessment,
    plan: &FileSanitizationPlan,
) -> SafetyValidationResult {
    let mut reasons = Vec::new();
    let mut blocked = false;

    // 1. Target path consistency
    if assessment.target_path != plan.target_path {
        blocked = true;
        reasons.push(format!(
            "Target path mismatch: assessment target is {:?}, but plan target is {:?}",
            assessment.target_path, plan.target_path
        ));
    }

    // 2. Safety classification consistency
    if assessment.safety != plan.safety {
        blocked = true;
        reasons.push(format!(
            "Safety classification mismatch: assessment is {:?}, but plan is {:?}",
            assessment.safety, plan.safety
        ));
    }

    // 3. Mandatory SafeToAnalyze enforcement
    if !matches!(assessment.safety, SafetyClassification::SafeToAnalyze) {
        blocked = true;
        reasons.push(format!(
            "Target is not safe to analyze; safety classification is {:?}",
            assessment.safety
        ));
    }
    if !matches!(plan.safety, SafetyClassification::SafeToAnalyze) {
        blocked = true;
        reasons.push(format!(
            "Plan safety classification is not SafeToAnalyze; plan safety is {:?}",
            plan.safety
        ));
    }

    // 4. Duplicate method detection & order validation
    let mut seen_methods = HashSet::new();
    let mut last_rank = 0u8;

    for proposal in &plan.proposed_methods {
        // Duplicates check
        if !seen_methods.insert(proposal.method) {
            blocked = true;
            reasons.push(format!(
                "Duplicate sanitization method detected in plan: {:?}",
                proposal.method
            ));
        }

        // Relative canonical order check
        let current_rank = method_order_rank(proposal.method);
        if current_rank < last_rank {
            blocked = true;
            reasons.push(format!(
                "Method {:?} is out of deterministic order (rank {} after rank {})",
                proposal.method, current_rank, last_rank
            ));
        }
        last_rank = current_rank;

        // 5. Capability state consistency check
        if let Err(msg) = validate_proposal_capability(assessment, proposal) {
            blocked = true;
            reasons.push(msg);
        }
    }

    if blocked {
        SafetyValidationResult {
            status: SafetyValidationStatus::Blocked,
            target_path: plan.target_path.clone(),
            reasons,
            validated_method_count: 0,
            has_executable_methods: false,
        }
    } else if plan.proposed_methods.is_empty() {
        SafetyValidationResult {
            status: SafetyValidationStatus::Allowed,
            target_path: plan.target_path.clone(),
            reasons: vec![
                "Plan snapshot is consistent and safe, but zero executable methods were proposed."
                    .into(),
            ],
            validated_method_count: 0,
            has_executable_methods: false,
        }
    } else {
        SafetyValidationResult {
            status: SafetyValidationStatus::Allowed,
            target_path: plan.target_path.clone(),
            reasons: vec![format!(
                "Plan snapshot validated successfully with {} executable method(s).",
                plan.proposed_methods.len()
            )],
            validated_method_count: plan.proposed_methods.len(),
            has_executable_methods: true,
        }
    }
}

fn validate_proposal_capability(
    assessment: &FileCapabilityAssessment,
    proposal: &MethodProposal,
) -> Result<(), String> {
    if proposal.capability_state != CapabilityState::Supported {
        return Err(format!(
            "Method proposal {:?} has non-supported capability_state {:?}",
            proposal.method, proposal.capability_state
        ));
    }

    let assessed_state = match proposal.method {
        FileSanitizationMethod::ContentOverwrite => assessment.content_sanitization.state,
        FileSanitizationMethod::MetadataSanitization => assessment.metadata_sanitization.state,
        FileSanitizationMethod::AlternateDataStreamSanitization => {
            assessment.alternate_data_streams.state
        }
        FileSanitizationMethod::SlackSpaceSanitization => assessment.slack_space.state,
        FileSanitizationMethod::FreeSpaceSanitization => assessment.free_space.state,
    };

    if assessed_state != CapabilityState::Supported {
        return Err(format!(
            "Method {:?} is proposed, but supporting assessment capability state is {:?}",
            proposal.method, assessed_state
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::{CapabilityFinding, CapabilityState, FileCapabilityAssessment};
    use crate::method_select::{
        FileSanitizationMethod, FileSanitizationPlan, MethodProposal,
    };
    use crate::model::{SafetyClassification, TargetKind};
    use std::path::PathBuf;

    fn make_assessment(
        path: &str,
        safety: SafetyClassification,
        content: CapabilityState,
        metadata: CapabilityState,
        ads: CapabilityState,
        slack: CapabilityState,
        free_space: CapabilityState,
    ) -> FileCapabilityAssessment {
        FileCapabilityAssessment {
            target_path: PathBuf::from(path),
            kind: TargetKind::RegularFile,
            safety,
            content_sanitization: CapabilityFinding {
                state: content,
                reason: "content reason".into(),
            },
            metadata_sanitization: CapabilityFinding {
                state: metadata,
                reason: "metadata reason".into(),
            },
            alternate_data_streams: CapabilityFinding {
                state: ads,
                reason: "ads reason".into(),
            },
            slack_space: CapabilityFinding {
                state: slack,
                reason: "slack reason".into(),
            },
            free_space: CapabilityFinding {
                state: free_space,
                reason: "free space reason".into(),
            },
        }
    }

    fn make_plan(
        path: &str,
        safety: SafetyClassification,
        methods: Vec<FileSanitizationMethod>,
    ) -> FileSanitizationPlan {
        let proposals = methods
            .into_iter()
            .map(|m| MethodProposal {
                method: m,
                label: m.label().to_string(),
                rationale: "valid rationale".into(),
                capability_state: CapabilityState::Supported,
            })
            .collect();

        FileSanitizationPlan {
            target_path: PathBuf::from(path),
            safety,
            proposed_methods: proposals,
            summary: "plan summary".into(),
        }
    }

    #[test]
    fn test_1_valid_safe_plan_one_supported_method_allowed() {
        let assessment = make_assessment(
            r"D:\Data\file.txt",
            SafetyClassification::SafeToAnalyze,
            CapabilityState::Supported,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
        );
        let plan = make_plan(
            r"D:\Data\file.txt",
            SafetyClassification::SafeToAnalyze,
            vec![FileSanitizationMethod::ContentOverwrite],
        );

        let res = validate_sanitization_plan(&assessment, &plan);
        assert_eq!(res.status, SafetyValidationStatus::Allowed);
        assert!(res.has_executable_methods);
        assert_eq!(res.validated_method_count, 1);
        assert!(res.is_ready_for_execution());
    }

    #[test]
    fn test_2_valid_safe_plan_multiple_supported_methods_allowed() {
        let assessment = make_assessment(
            r"D:\Data\file.txt",
            SafetyClassification::SafeToAnalyze,
            CapabilityState::Supported,
            CapabilityState::Supported,
            CapabilityState::Supported,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
        );
        let plan = make_plan(
            r"D:\Data\file.txt",
            SafetyClassification::SafeToAnalyze,
            vec![
                FileSanitizationMethod::ContentOverwrite,
                FileSanitizationMethod::MetadataSanitization,
                FileSanitizationMethod::AlternateDataStreamSanitization,
            ],
        );

        let res = validate_sanitization_plan(&assessment, &plan);
        assert_eq!(res.status, SafetyValidationStatus::Allowed);
        assert!(res.has_executable_methods);
        assert_eq!(res.validated_method_count, 3);
    }

    #[test]
    fn test_3_valid_empty_plan_safe_to_analyze_non_authorizing() {
        let assessment = make_assessment(
            r"D:\Data\file.txt",
            SafetyClassification::SafeToAnalyze,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
        );
        let plan = make_plan(
            r"D:\Data\file.txt",
            SafetyClassification::SafeToAnalyze,
            vec![],
        );

        let res = validate_sanitization_plan(&assessment, &plan);
        assert_eq!(res.status, SafetyValidationStatus::Allowed);
        assert!(!res.has_executable_methods);
        assert_eq!(res.validated_method_count, 0);
        assert!(!res.is_ready_for_execution());
        assert!(res.reasons[0].contains("zero executable methods were proposed"));
    }

    #[test]
    fn test_4_unsafe_protected_target_blocked() {
        let assessment = make_assessment(
            r"C:\Windows\System32\cmd.exe",
            SafetyClassification::ProtectedSystemPath {
                reason: "Protected system path".into(),
            },
            CapabilityState::Supported,
            CapabilityState::Supported,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
        );
        let plan = make_plan(
            r"C:\Windows\System32\cmd.exe",
            SafetyClassification::ProtectedSystemPath {
                reason: "Protected system path".into(),
            },
            vec![FileSanitizationMethod::ContentOverwrite],
        );

        let res = validate_sanitization_plan(&assessment, &plan);
        assert_eq!(res.status, SafetyValidationStatus::Blocked);
        assert!(!res.has_executable_methods);
        assert!(!res.is_ready_for_execution());
        assert!(res.reasons.iter().any(|r| r.contains("not safe to analyze")));
    }

    #[test]
    fn test_5_blocked_reparse_target_blocked() {
        let assessment = make_assessment(
            r"D:\JunctionDir",
            SafetyClassification::BlockedReparsePoint {
                reason: "Reparse boundary".into(),
            },
            CapabilityState::Supported,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
        );
        let plan = make_plan(
            r"D:\JunctionDir",
            SafetyClassification::BlockedReparsePoint {
                reason: "Reparse boundary".into(),
            },
            vec![FileSanitizationMethod::ContentOverwrite],
        );

        let res = validate_sanitization_plan(&assessment, &plan);
        assert_eq!(res.status, SafetyValidationStatus::Blocked);
    }

    #[test]
    fn test_6_inaccessible_target_blocked() {
        let assessment = make_assessment(
            r"D:\Locked\doc.txt",
            SafetyClassification::Inaccessible {
                reason: "Access denied".into(),
            },
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
        );
        let plan = make_plan(
            r"D:\Locked\doc.txt",
            SafetyClassification::Inaccessible {
                reason: "Access denied".into(),
            },
            vec![],
        );

        let res = validate_sanitization_plan(&assessment, &plan);
        assert_eq!(res.status, SafetyValidationStatus::Blocked);
    }

    #[test]
    fn test_7_plan_target_path_mismatch_blocked() {
        let assessment = make_assessment(
            r"D:\Data\file_a.txt",
            SafetyClassification::SafeToAnalyze,
            CapabilityState::Supported,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
        );
        let plan = make_plan(
            r"D:\Data\file_b.txt",
            SafetyClassification::SafeToAnalyze,
            vec![FileSanitizationMethod::ContentOverwrite],
        );

        let res = validate_sanitization_plan(&assessment, &plan);
        assert_eq!(res.status, SafetyValidationStatus::Blocked);
        assert!(res.reasons.iter().any(|r| r.contains("Target path mismatch")));
    }

    #[test]
    fn test_8_safety_classification_mismatch_blocked() {
        let assessment = make_assessment(
            r"D:\Data\file.txt",
            SafetyClassification::SafeToAnalyze,
            CapabilityState::Supported,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
        );
        let plan = make_plan(
            r"D:\Data\file.txt",
            SafetyClassification::ProtectedSystemPath {
                reason: "Simulated mismatch".into(),
            },
            vec![FileSanitizationMethod::ContentOverwrite],
        );

        let res = validate_sanitization_plan(&assessment, &plan);
        assert_eq!(res.status, SafetyValidationStatus::Blocked);
        assert!(res.reasons.iter().any(|r| r.contains("Safety classification mismatch")));
    }

    #[test]
    fn test_9_proposal_with_unsupported_capability_blocked() {
        let assessment = make_assessment(
            r"D:\Data\file.txt",
            SafetyClassification::SafeToAnalyze,
            CapabilityState::Unsupported,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
        );
        let plan = make_plan(
            r"D:\Data\file.txt",
            SafetyClassification::SafeToAnalyze,
            vec![FileSanitizationMethod::ContentOverwrite],
        );

        let res = validate_sanitization_plan(&assessment, &plan);
        assert_eq!(res.status, SafetyValidationStatus::Blocked);
        assert!(res.reasons.iter().any(|r| r.contains("assessment capability state is Unsupported")));
    }

    #[test]
    fn test_10_proposal_with_unknown_capability_blocked() {
        let assessment = make_assessment(
            r"D:\Data\file.txt",
            SafetyClassification::SafeToAnalyze,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
        );
        let plan = make_plan(
            r"D:\Data\file.txt",
            SafetyClassification::SafeToAnalyze,
            vec![FileSanitizationMethod::ContentOverwrite],
        );

        let res = validate_sanitization_plan(&assessment, &plan);
        assert_eq!(res.status, SafetyValidationStatus::Blocked);
        assert!(res.reasons.iter().any(|r| r.contains("assessment capability state is Unknown")));
    }

    #[test]
    fn test_11_proposal_with_incorrect_proposal_capability_state_blocked() {
        let assessment = make_assessment(
            r"D:\Data\file.txt",
            SafetyClassification::SafeToAnalyze,
            CapabilityState::Supported,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
        );
        let mut plan = make_plan(
            r"D:\Data\file.txt",
            SafetyClassification::SafeToAnalyze,
            vec![FileSanitizationMethod::ContentOverwrite],
        );
        plan.proposed_methods[0].capability_state = CapabilityState::Unknown;

        let res = validate_sanitization_plan(&assessment, &plan);
        assert_eq!(res.status, SafetyValidationStatus::Blocked);
        assert!(res.reasons.iter().any(|r| r.contains("non-supported capability_state Unknown")));
    }

    #[test]
    fn test_12_proposal_method_capability_mismatch_blocked() {
        let assessment = make_assessment(
            r"D:\Data\file.txt",
            SafetyClassification::SafeToAnalyze,
            CapabilityState::Supported, // Content is supported
            CapabilityState::Unknown,   // Metadata is NOT supported
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
        );
        // Plan proposes MetadataSanitization while only content is supported
        let plan = make_plan(
            r"D:\Data\file.txt",
            SafetyClassification::SafeToAnalyze,
            vec![FileSanitizationMethod::MetadataSanitization],
        );

        let res = validate_sanitization_plan(&assessment, &plan);
        assert_eq!(res.status, SafetyValidationStatus::Blocked);
        assert!(res.reasons.iter().any(|r| r.contains("Method MetadataSanitization is proposed, but supporting assessment capability state is Unknown")));
    }

    #[test]
    fn test_13_unexpected_method_relative_to_capability_evidence_blocked() {
        let assessment = make_assessment(
            r"D:\Data\file.txt",
            SafetyClassification::SafeToAnalyze,
            CapabilityState::Supported,
            CapabilityState::Supported,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
        );
        let plan = make_plan(
            r"D:\Data\file.txt",
            SafetyClassification::SafeToAnalyze,
            vec![
                FileSanitizationMethod::ContentOverwrite,
                FileSanitizationMethod::MetadataSanitization,
                FileSanitizationMethod::FreeSpaceSanitization, // Unsupported/Unknown
            ],
        );

        let res = validate_sanitization_plan(&assessment, &plan);
        assert_eq!(res.status, SafetyValidationStatus::Blocked);
        assert!(res.reasons.iter().any(|r| r.contains("FreeSpaceSanitization")));
    }

    #[test]
    fn test_14_duplicate_method_blocked() {
        let assessment = make_assessment(
            r"D:\Data\file.txt",
            SafetyClassification::SafeToAnalyze,
            CapabilityState::Supported,
            CapabilityState::Supported,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
        );
        let plan = make_plan(
            r"D:\Data\file.txt",
            SafetyClassification::SafeToAnalyze,
            vec![
                FileSanitizationMethod::ContentOverwrite,
                FileSanitizationMethod::ContentOverwrite, // Duplicate
            ],
        );

        let res = validate_sanitization_plan(&assessment, &plan);
        assert_eq!(res.status, SafetyValidationStatus::Blocked);
        assert!(res.reasons.iter().any(|r| r.contains("Duplicate sanitization method detected")));
    }

    #[test]
    fn test_15_incorrect_ordering_blocked() {
        let assessment = make_assessment(
            r"D:\Data\file.txt",
            SafetyClassification::SafeToAnalyze,
            CapabilityState::Supported,
            CapabilityState::Supported,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
        );
        // Inverted order: Metadata before ContentOverwrite
        let plan = make_plan(
            r"D:\Data\file.txt",
            SafetyClassification::SafeToAnalyze,
            vec![
                FileSanitizationMethod::MetadataSanitization,
                FileSanitizationMethod::ContentOverwrite,
            ],
        );

        let res = validate_sanitization_plan(&assessment, &plan);
        assert_eq!(res.status, SafetyValidationStatus::Blocked);
        assert!(res.reasons.iter().any(|r| r.contains("out of deterministic order")));
    }

    #[test]
    fn test_16_correct_deterministic_ordering_allowed() {
        let assessment = make_assessment(
            r"D:\Data\file.txt",
            SafetyClassification::SafeToAnalyze,
            CapabilityState::Supported,
            CapabilityState::Supported,
            CapabilityState::Supported,
            CapabilityState::Supported,
            CapabilityState::Supported,
        );
        let plan = make_plan(
            r"D:\Data\file.txt",
            SafetyClassification::SafeToAnalyze,
            vec![
                FileSanitizationMethod::ContentOverwrite,
                FileSanitizationMethod::MetadataSanitization,
                FileSanitizationMethod::AlternateDataStreamSanitization,
                FileSanitizationMethod::SlackSpaceSanitization,
                FileSanitizationMethod::FreeSpaceSanitization,
            ],
        );

        let res = validate_sanitization_plan(&assessment, &plan);
        assert_eq!(res.status, SafetyValidationStatus::Allowed);
        assert!(res.has_executable_methods);
        assert_eq!(res.validated_method_count, 5);
    }

    #[test]
    fn test_17_identical_input_produces_identical_validation_result() {
        let assessment = make_assessment(
            r"D:\Data\file.txt",
            SafetyClassification::SafeToAnalyze,
            CapabilityState::Supported,
            CapabilityState::Supported,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
        );
        let plan = make_plan(
            r"D:\Data\file.txt",
            SafetyClassification::SafeToAnalyze,
            vec![
                FileSanitizationMethod::ContentOverwrite,
                FileSanitizationMethod::MetadataSanitization,
            ],
        );

        let res1 = validate_sanitization_plan(&assessment, &plan);
        let res2 = validate_sanitization_plan(&assessment, &plan);
        assert_eq!(res1, res2);
    }

    #[test]
    fn test_18_invalid_plan_is_not_silently_repaired() {
        let assessment = make_assessment(
            r"D:\Data\file.txt",
            SafetyClassification::SafeToAnalyze,
            CapabilityState::Supported,
            CapabilityState::Supported,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
        );
        let plan = make_plan(
            r"D:\Data\file.txt",
            SafetyClassification::SafeToAnalyze,
            vec![
                FileSanitizationMethod::MetadataSanitization,
                FileSanitizationMethod::ContentOverwrite,
            ],
        );

        let res = validate_sanitization_plan(&assessment, &plan);
        assert_eq!(res.status, SafetyValidationStatus::Blocked);
        // Ensure plan inside was not mutated
        assert_eq!(
            plan.proposed_methods[0].method,
            FileSanitizationMethod::MetadataSanitization
        );
        assert_eq!(
            plan.proposed_methods[1].method,
            FileSanitizationMethod::ContentOverwrite
        );
    }

    #[test]
    fn test_19_zero_methods_never_reports_executable_authorization() {
        let assessment = make_assessment(
            r"D:\Data\file.txt",
            SafetyClassification::SafeToAnalyze,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
        );
        let plan = make_plan(
            r"D:\Data\file.txt",
            SafetyClassification::SafeToAnalyze,
            vec![],
        );

        let res = validate_sanitization_plan(&assessment, &plan);
        assert!(!res.has_executable_methods);
        assert!(!res.is_ready_for_execution());
    }

    #[test]
    fn test_20_no_filesystem_hardware_access_performed() {
        let assessment = make_assessment(
            r"Z:\Nonexistent\Fake\path.txt",
            SafetyClassification::SafeToAnalyze,
            CapabilityState::Supported,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
            CapabilityState::Unknown,
        );
        let plan = make_plan(
            r"Z:\Nonexistent\Fake\path.txt",
            SafetyClassification::SafeToAnalyze,
            vec![FileSanitizationMethod::ContentOverwrite],
        );

        // Operates purely on provided in-memory references
        let res = validate_sanitization_plan(&assessment, &plan);
        assert_eq!(res.status, SafetyValidationStatus::Allowed);
        assert!(res.is_ready_for_execution());
    }
}