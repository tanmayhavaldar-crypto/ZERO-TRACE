//! Step 5.3 — Safety Rules & Drive Validation Layer.
//!
//! # Strict Design Invariants
//! 1. Non-Destructive: Pure in-memory evaluation logic only. No command execution,
//!    IOCTLs, partition modification, formatting, or sector writes.
//! 2. Fail-Closed: When classification or safety flag structures are unavailable (`None`),
//!    or when critical protection flags are `TriState::Unknown`, the drive is
//!    conservatively blocked.
//! 3. System & Boot Protection: Drives determined to host system or boot partitions are blocked.
//! 4. Read-Only Protection: Drives determined to be read-only are blocked.
//! 5. Conservatism Enforcement: If `requires_conservatism` is set, authorization is blocked.
//! 6. Multi-Reason Retention: All applicable blocking reasons are preserved in deterministic order.
//! 7. Independent from Capability: Capability and method selection never bypass safety blocks.

use crate::model::{DiscoveredDrive, DriveClassification, SafetyFlags, TriState};
use serde::{Deserialize, Serialize};

/// High-level safety verdict indicating whether a physical drive is permitted
/// to proceed toward future sanitization staging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafetyVerdict {
    /// The target drive has passed all safety checks and is sufficiently determined.
    Allowed,

    /// The target drive is blocked due to unsafe or unknown safety state.
    Blocked,
}

/// Strongly typed blocking reasons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BlockReasonCode {
    SystemDrive,
    BootDrive,
    ReadOnlyDrive,
    RemovableDriveUnsupported,
    RequiresConservatismBlocked,
    SystemStateUnknown,
    BootStateUnknown,
    ReadOnlyStateUnknown,
    CriticalSafetyInfoUnavailable,
}

impl BlockReasonCode {
    /// Human-readable explanation of the blocking finding.
    #[must_use]
    pub const fn message(&self) -> &'static str {
        match self {
            Self::SystemDrive => {
                "Target physical drive contains the active Windows system partition."
            }
            Self::BootDrive => {
                "Target physical drive contains the active Windows boot partition."
            }
            Self::ReadOnlyDrive => {
                "Target physical drive is reported as read-only; sanitization cannot proceed."
            }
            Self::RemovableDriveUnsupported => {
                "Target physical drive is classified as removable media."
            }
            Self::RequiresConservatismBlocked => {
                "Target physical drive has a conservative safety policy flag asserted."
            }
            Self::SystemStateUnknown => {
                "System drive status cannot be reliably determined; fail-closed policy applied."
            }
            Self::BootStateUnknown => {
                "Boot drive status cannot be reliably determined; fail-closed policy applied."
            }
            Self::ReadOnlyStateUnknown => {
                "Read-only state cannot be reliably determined; fail-closed policy applied."
            }
            Self::CriticalSafetyInfoUnavailable => {
                "Critical drive safety flags or classification data are unavailable; \
                 fail-closed policy applied."
            }
        }
    }
}

/// A single auditable safety finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafetyFinding {
    pub code: BlockReasonCode,
    pub description: String,
}

/// Comprehensive safety validation outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafetyAssessment {
    /// Whether the drive is permitted or blocked.
    pub verdict: SafetyVerdict,

    /// Deterministic list of all detected blocking conditions.
    pub blocking_reasons: Vec<SafetyFinding>,
}

impl SafetyAssessment {
    /// Returns true only if the drive is explicitly permitted.
    #[must_use]
    pub fn is_allowed(&self) -> bool {
        self.verdict == SafetyVerdict::Allowed
    }
}

/// Evaluate the safety of a physical drive.
///
/// This function is pure in-memory logic. It does not access hardware,
/// execute commands, or modify any storage.
///
/// # Fail-Closed Rules
///
/// - Missing classification or safety flags -> Blocked.
/// - `TriState::Supported` for system, boot, or read-only -> Blocked.
/// - `TriState::Unknown` for system, boot, or read-only -> Blocked.
/// - `TriState::Unsupported` confirms the relevant condition is absent.
/// - `requires_conservatism == true` -> Blocked.
///
/// The classification value is currently required to ensure that the caller
/// supplied a complete classification snapshot, but safety decisions are based
/// on the authoritative `SafetyFlags`.
#[must_use]
pub fn validate_drive_safety(
    classification: Option<&DriveClassification>,
    safety_flags: Option<&SafetyFlags>,
) -> SafetyAssessment {
    let mut reasons = Vec::new();

    // Both pieces of information are required.
    let (_classification, flags) = match (classification, safety_flags) {
        (Some(classification), Some(flags)) => (classification, flags),
        _ => {
            reasons.push(SafetyFinding {
                code: BlockReasonCode::CriticalSafetyInfoUnavailable,
                description: BlockReasonCode::CriticalSafetyInfoUnavailable
                    .message()
                    .to_string(),
            });

            return SafetyAssessment {
                verdict: SafetyVerdict::Blocked,
                blocking_reasons: reasons,
            };
        }
    };

    // 1. System drive evaluation.
    match flags.is_system {
        TriState::Supported => {
            reasons.push(SafetyFinding {
                code: BlockReasonCode::SystemDrive,
                description: BlockReasonCode::SystemDrive.message().to_string(),
            });
        }
        TriState::Unknown => {
            reasons.push(SafetyFinding {
                code: BlockReasonCode::SystemStateUnknown,
                description: BlockReasonCode::SystemStateUnknown.message().to_string(),
            });
        }
        TriState::Unsupported => {}
    }

    // 2. Boot drive evaluation.
    match flags.is_boot {
        TriState::Supported => {
            reasons.push(SafetyFinding {
                code: BlockReasonCode::BootDrive,
                description: BlockReasonCode::BootDrive.message().to_string(),
            });
        }
        TriState::Unknown => {
            reasons.push(SafetyFinding {
                code: BlockReasonCode::BootStateUnknown,
                description: BlockReasonCode::BootStateUnknown.message().to_string(),
            });
        }
        TriState::Unsupported => {}
    }

    // 3. Read-only evaluation.
    match flags.is_read_only {
        TriState::Supported => {
            reasons.push(SafetyFinding {
                code: BlockReasonCode::ReadOnlyDrive,
                description: BlockReasonCode::ReadOnlyDrive.message().to_string(),
            });
        }
        TriState::Unknown => {
            reasons.push(SafetyFinding {
                code: BlockReasonCode::ReadOnlyStateUnknown,
                description: BlockReasonCode::ReadOnlyStateUnknown
                    .message()
                    .to_string(),
            });
        }
        TriState::Unsupported => {}
    }

    // 4. Conservative safety policy.
    if flags.requires_conservatism {
        reasons.push(SafetyFinding {
            code: BlockReasonCode::RequiresConservatismBlocked,
            description: BlockReasonCode::RequiresConservatismBlocked
                .message()
                .to_string(),
        });
    }

    if reasons.is_empty() {
        SafetyAssessment {
            verdict: SafetyVerdict::Allowed,
            blocking_reasons: Vec::new(),
        }
    } else {
        SafetyAssessment {
            verdict: SafetyVerdict::Blocked,
            blocking_reasons: reasons,
        }
    }
}

/// Validate a fully populated discovered drive.
///
/// Uses the existing nested `drive.safety` information and converts the
/// existing `drive.is_read_only: bool` into the `TriState` representation
/// required by `SafetyFlags`.
#[must_use]
pub fn validate_discovered_drive(drive: &DiscoveredDrive) -> SafetyAssessment {
    let is_read_only = if drive.is_read_only {
        TriState::Supported
    } else {
        TriState::Unsupported
    };

    let flags = SafetyFlags {
        is_system: drive.safety.is_system,
        is_boot: drive.safety.is_boot,
        is_removable: drive.safety.is_removable,
        is_read_only,
        requires_conservatism: drive.safety.requires_conservatism,
    };

    validate_drive_safety(Some(&drive.classification), Some(&flags))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_classification() -> DriveClassification {
        DriveClassification::SataSsd
    }

    fn make_safe_safety_flags() -> SafetyFlags {
        SafetyFlags {
            is_system: TriState::Unsupported,
            is_boot: TriState::Unsupported,
            is_removable: TriState::Unsupported,
            is_read_only: TriState::Unsupported,
            requires_conservatism: false,
        }
    }

    #[test]
    fn test_1_clearly_safe_snapshot_is_allowed() {
        let classification = make_test_classification();
        let flags = make_safe_safety_flags();

        let assessment = validate_drive_safety(Some(&classification), Some(&flags));

        assert!(assessment.is_allowed());
        assert_eq!(assessment.verdict, SafetyVerdict::Allowed);
        assert!(assessment.blocking_reasons.is_empty());
    }

    #[test]
    fn test_2_system_drive_blocked() {
        let classification = make_test_classification();
        let mut flags = make_safe_safety_flags();

        flags.is_system = TriState::Supported;

        let assessment = validate_drive_safety(Some(&classification), Some(&flags));

        assert!(!assessment.is_allowed());
        assert_eq!(assessment.verdict, SafetyVerdict::Blocked);
        assert_eq!(assessment.blocking_reasons.len(), 1);
        assert_eq!(
            assessment.blocking_reasons[0].code,
            BlockReasonCode::SystemDrive
        );
    }

    #[test]
    fn test_3_boot_drive_blocked() {
        let classification = make_test_classification();
        let mut flags = make_safe_safety_flags();

        flags.is_boot = TriState::Supported;

        let assessment = validate_drive_safety(Some(&classification), Some(&flags));

        assert!(!assessment.is_allowed());
        assert_eq!(assessment.verdict, SafetyVerdict::Blocked);
        assert_eq!(assessment.blocking_reasons.len(), 1);
        assert_eq!(
            assessment.blocking_reasons[0].code,
            BlockReasonCode::BootDrive
        );
    }

    #[test]
    fn test_4_read_only_drive_blocked() {
        let classification = make_test_classification();
        let mut flags = make_safe_safety_flags();

        flags.is_read_only = TriState::Supported;

        let assessment = validate_drive_safety(Some(&classification), Some(&flags));

        assert!(!assessment.is_allowed());
        assert_eq!(assessment.verdict, SafetyVerdict::Blocked);
        assert_eq!(assessment.blocking_reasons.len(), 1);
        assert_eq!(
            assessment.blocking_reasons[0].code,
            BlockReasonCode::ReadOnlyDrive
        );
    }

    #[test]
    fn test_5_removable_drive_does_not_override_other_safety_rules() {
        let classification = make_test_classification();
        let mut flags = make_safe_safety_flags();

        flags.is_removable = TriState::Supported;

        // Removable status alone is not a blocking condition in the current
        // safety policy. Other future policy layers may make a decision.
        let assessment = validate_drive_safety(Some(&classification), Some(&flags));

        assert!(assessment.is_allowed());

        flags.requires_conservatism = true;

        let conservative_assessment =
            validate_drive_safety(Some(&classification), Some(&flags));

        assert!(!conservative_assessment.is_allowed());
        assert_eq!(
            conservative_assessment.blocking_reasons[0].code,
            BlockReasonCode::RequiresConservatismBlocked
        );
    }

    #[test]
    fn test_6_multiple_applicable_reasons_preserved() {
        let classification = make_test_classification();
        let mut flags = make_safe_safety_flags();

        flags.is_system = TriState::Supported;
        flags.is_boot = TriState::Supported;
        flags.is_read_only = TriState::Supported;

        let assessment = validate_drive_safety(Some(&classification), Some(&flags));

        assert!(!assessment.is_allowed());
        assert_eq!(assessment.blocking_reasons.len(), 3);

        assert_eq!(
            assessment.blocking_reasons[0].code,
            BlockReasonCode::SystemDrive
        );
        assert_eq!(
            assessment.blocking_reasons[1].code,
            BlockReasonCode::BootDrive
        );
        assert_eq!(
            assessment.blocking_reasons[2].code,
            BlockReasonCode::ReadOnlyDrive
        );
    }

    #[test]
    fn test_7_missing_classification_blocked() {
        let flags = make_safe_safety_flags();

        let assessment = validate_drive_safety(None, Some(&flags));

        assert!(!assessment.is_allowed());
        assert_eq!(assessment.verdict, SafetyVerdict::Blocked);
        assert_eq!(
            assessment.blocking_reasons[0].code,
            BlockReasonCode::CriticalSafetyInfoUnavailable
        );
    }

    #[test]
    fn test_8_missing_safety_flags_blocked() {
        let classification = make_test_classification();

        let assessment = validate_drive_safety(Some(&classification), None);

        assert!(!assessment.is_allowed());
        assert_eq!(assessment.verdict, SafetyVerdict::Blocked);
        assert_eq!(
            assessment.blocking_reasons[0].code,
            BlockReasonCode::CriticalSafetyInfoUnavailable
        );
    }

    #[test]
    fn test_9_deterministic_result_and_reason_ordering() {
        let classification = make_test_classification();
        let mut flags = make_safe_safety_flags();

        flags.is_system = TriState::Supported;
        flags.is_read_only = TriState::Supported;

        let first = validate_drive_safety(Some(&classification), Some(&flags));
        let second = validate_drive_safety(Some(&classification), Some(&flags));

        assert_eq!(first, second);

        assert_eq!(
            first.blocking_reasons[0].code,
            BlockReasonCode::SystemDrive
        );
        assert_eq!(
            first.blocking_reasons[1].code,
            BlockReasonCode::ReadOnlyDrive
        );
    }

    #[test]
    fn test_10_no_capability_or_method_selection_dependency() {
        let classification = make_test_classification();
        let mut flags = make_safe_safety_flags();

        flags.is_system = TriState::Supported;

        let assessment = validate_drive_safety(Some(&classification), Some(&flags));

        assert!(!assessment.is_allowed());
        assert_eq!(
            assessment.blocking_reasons[0].code,
            BlockReasonCode::SystemDrive
        );
    }

    #[test]
    fn test_11_pure_in_memory_evaluation() {
        let classification = make_test_classification();
        let mut flags = make_safe_safety_flags();

        flags.requires_conservatism = true;

        let assessment = validate_drive_safety(Some(&classification), Some(&flags));

        assert!(!assessment.is_allowed());
        assert_eq!(
            assessment.blocking_reasons[0].code,
            BlockReasonCode::RequiresConservatismBlocked
        );
    }

    #[test]
    fn test_12_tristate_unknown_handled_conservatively() {
        let classification = make_test_classification();

        // Unknown system state -> Blocked.
        let mut system_unknown = make_safe_safety_flags();
        system_unknown.is_system = TriState::Unknown;

        let assessment = validate_drive_safety(
            Some(&classification),
            Some(&system_unknown),
        );

        assert!(!assessment.is_allowed());
        assert_eq!(
            assessment.blocking_reasons[0].code,
            BlockReasonCode::SystemStateUnknown
        );

        // Unknown boot state -> Blocked.
        let mut boot_unknown = make_safe_safety_flags();
        boot_unknown.is_boot = TriState::Unknown;

        let assessment =
            validate_drive_safety(Some(&classification), Some(&boot_unknown));

        assert!(!assessment.is_allowed());
        assert_eq!(
            assessment.blocking_reasons[0].code,
            BlockReasonCode::BootStateUnknown
        );

        // Unknown read-only state -> Blocked.
        let mut readonly_unknown = make_safe_safety_flags();
        readonly_unknown.is_read_only = TriState::Unknown;

        let assessment =
            validate_drive_safety(Some(&classification), Some(&readonly_unknown));

        assert!(!assessment.is_allowed());
        assert_eq!(
            assessment.blocking_reasons[0].code,
            BlockReasonCode::ReadOnlyStateUnknown
        );
    }
}