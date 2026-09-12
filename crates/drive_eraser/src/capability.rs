//! Capability detection and evaluation for storage devices.
//!
//! # Strict Design Invariants
//! 1. Epistemic Certainty: An operation is only marked `TriState::Supported` or
//!    `TriState::Unsupported` when backed by authoritative, verified evidence.
//! 2. No Inference from Classification: Bus type (e.g. NVMe, SATA), media type (e.g. SSD, HDD),
//!    or high-level drive classification MUST NEVER be used to infer hardware erase capabilities.
//! 3. Unknown != Unsupported: Absence of evidence or unqueried commands must evaluate to
//!    `TriState::Unknown`, never `TriState::Unsupported`.
//! 4. Read-Only Guarantee: This layer performs no destructive operations, no command execution,
//!    no overwrite, and no device sanitization.

use crate::model::{DriveCapabilities, TriState};

/// Verified capability evidence for ATA-based drives (e.g., SATA/PATA).
///
/// Fields represent explicit bit flags parsed from authoritative ATA IDENTIFY DEVICE
/// or IDENTIFY PACKET DEVICE data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AtaCapabilityEvidence {
    pub secure_erase_supported: TriState,
    pub enhanced_secure_erase_supported: TriState,
    pub sanitize_block_supported: TriState,
    pub sanitize_crypto_supported: TriState,
    pub sanitize_overwrite_supported: TriState,
}

/// Verified capability evidence for NVMe-based drives.
///
/// Fields represent explicit bit flags parsed from authoritative NVMe Identify Controller data
/// (e.g., OACS - Optional Admin Command Support, and SANICAP - Sanitize Capabilities).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NvmeCapabilityEvidence {
    pub format_supported: TriState,
    pub format_crypto_supported: TriState,
    pub sanitize_block_supported: TriState,
    pub sanitize_crypto_supported: TriState,
    pub sanitize_overwrite_supported: TriState,
}

/// Verified capability evidence for SCSI/SAS-based drives.
///
/// Fields represent explicit bit flags parsed from authoritative SCSI inquiry / VPC pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ScsiCapabilityEvidence {
    pub sanitize_supported: TriState,
}

/// Aggregated, verified capability evidence parsed from non-destructive queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CapabilityEvidence {
    pub ata: Option<AtaCapabilityEvidence>,
    pub nvme: Option<NvmeCapabilityEvidence>,
    pub scsi: Option<ScsiCapabilityEvidence>,
}

impl CapabilityEvidence {
    /// Creates an empty evidence set with no verified facts.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            ata: None,
            nvme: None,
            scsi: None,
        }
    }
}

/// Returns a default `DriveCapabilities` structure where every hardware capability
/// is conservatively marked as `TriState::Unknown`.
#[must_use]
pub const fn default_readonly_capabilities() -> DriveCapabilities {
    DriveCapabilities {
        ata_secure_erase: TriState::Unknown,
        ata_enhanced_secure_erase: TriState::Unknown,
        ata_sanitize_block: TriState::Unknown,
        ata_sanitize_crypto: TriState::Unknown,
        ata_sanitize_overwrite: TriState::Unknown,
        nvme_format: TriState::Unknown,
        nvme_format_crypto: TriState::Unknown,
        nvme_sanitize_block: TriState::Unknown,
        nvme_sanitize_crypto: TriState::Unknown,
        nvme_sanitize_overwrite: TriState::Unknown,
        scsi_sanitize: TriState::Unknown,
    }
}

/// Evaluates verified non-destructive evidence into explicit `DriveCapabilities`.
///
/// Invariant: Capabilities remain `TriState::Unknown` unless explicit evidence confirms
/// support or lack thereof.
#[must_use]
pub fn evaluate_capabilities_from_evidence(evidence: &CapabilityEvidence) -> DriveCapabilities {
    let mut caps = default_readonly_capabilities();

    if let Some(ata) = &evidence.ata {
        caps.ata_secure_erase = ata.secure_erase_supported;
        caps.ata_enhanced_secure_erase = ata.enhanced_secure_erase_supported;
        caps.ata_sanitize_block = ata.sanitize_block_supported;
        caps.ata_sanitize_crypto = ata.sanitize_crypto_supported;
        caps.ata_sanitize_overwrite = ata.sanitize_overwrite_supported;
    }

    if let Some(nvme) = &evidence.nvme {
        caps.nvme_format = nvme.format_supported;
        caps.nvme_format_crypto = nvme.format_crypto_supported;
        caps.nvme_sanitize_block = nvme.sanitize_block_supported;
        caps.nvme_sanitize_crypto = nvme.sanitize_crypto_supported;
        caps.nvme_sanitize_overwrite = nvme.sanitize_overwrite_supported;
    }

    if let Some(scsi) = &evidence.scsi {
        caps.scsi_sanitize = scsi.sanitize_supported;
    }

    caps
}

/// Backward-compatible Step 3 capability evaluator.
///
/// Preserves the original contract:
/// - Evaluates boolean existence of identify evidence.
/// - If `Some(false)`, marks the respective protocol's capabilities as `Unsupported`.
/// - If `None`, leaves the respective capabilities as `Unknown`.
/// - SCSI capability remains `Unknown` as this helper has no SCSI input.
#[must_use]
pub fn evaluate_safe_capabilities(
    has_ata_identify_evidence: Option<bool>,
    has_nvme_identify_evidence: Option<bool>,
) -> DriveCapabilities {
    let mut caps = default_readonly_capabilities();

    if has_ata_identify_evidence == Some(false) {
        caps.ata_secure_erase = TriState::Unsupported;
        caps.ata_enhanced_secure_erase = TriState::Unsupported;
        caps.ata_sanitize_block = TriState::Unsupported;
        caps.ata_sanitize_crypto = TriState::Unsupported;
        caps.ata_sanitize_overwrite = TriState::Unsupported;
    }

    if has_nvme_identify_evidence == Some(false) {
        caps.nvme_format = TriState::Unsupported;
        caps.nvme_format_crypto = TriState::Unsupported;
        caps.nvme_sanitize_block = TriState::Unsupported;
        caps.nvme_sanitize_crypto = TriState::Unsupported;
        caps.nvme_sanitize_overwrite = TriState::Unsupported;
    }

    caps
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::TriState;

    #[test]
    fn test_legacy_safe_capabilities_none_none_all_unknown() {
        let caps = evaluate_safe_capabilities(None, None);

        assert_eq!(caps.ata_secure_erase, TriState::Unknown);
        assert_eq!(caps.ata_enhanced_secure_erase, TriState::Unknown);
        assert_eq!(caps.ata_sanitize_block, TriState::Unknown);
        assert_eq!(caps.ata_sanitize_crypto, TriState::Unknown);
        assert_eq!(caps.ata_sanitize_overwrite, TriState::Unknown);
        assert_eq!(caps.nvme_format, TriState::Unknown);
        assert_eq!(caps.nvme_format_crypto, TriState::Unknown);
        assert_eq!(caps.nvme_sanitize_block, TriState::Unknown);
        assert_eq!(caps.nvme_sanitize_crypto, TriState::Unknown);
        assert_eq!(caps.nvme_sanitize_overwrite, TriState::Unknown);
        assert_eq!(caps.scsi_sanitize, TriState::Unknown);
    }

    #[test]
    fn test_legacy_safe_capabilities_ata_false_nvme_none() {
        let caps = evaluate_safe_capabilities(Some(false), None);

        assert_eq!(caps.ata_secure_erase, TriState::Unsupported);
        assert_eq!(caps.ata_enhanced_secure_erase, TriState::Unsupported);
        assert_eq!(caps.ata_sanitize_block, TriState::Unsupported);
        assert_eq!(caps.ata_sanitize_crypto, TriState::Unsupported);
        assert_eq!(caps.ata_sanitize_overwrite, TriState::Unsupported);

        assert_eq!(caps.nvme_format, TriState::Unknown);
        assert_eq!(caps.nvme_format_crypto, TriState::Unknown);
        assert_eq!(caps.nvme_sanitize_block, TriState::Unknown);
        assert_eq!(caps.nvme_sanitize_crypto, TriState::Unknown);
        assert_eq!(caps.nvme_sanitize_overwrite, TriState::Unknown);
        assert_eq!(caps.scsi_sanitize, TriState::Unknown);
    }

    #[test]
    fn test_legacy_safe_capabilities_ata_none_nvme_false() {
        let caps = evaluate_safe_capabilities(None, Some(false));

        assert_eq!(caps.ata_secure_erase, TriState::Unknown);
        assert_eq!(caps.ata_enhanced_secure_erase, TriState::Unknown);
        assert_eq!(caps.ata_sanitize_block, TriState::Unknown);
        assert_eq!(caps.ata_sanitize_crypto, TriState::Unknown);
        assert_eq!(caps.ata_sanitize_overwrite, TriState::Unknown);

        assert_eq!(caps.nvme_format, TriState::Unsupported);
        assert_eq!(caps.nvme_format_crypto, TriState::Unsupported);
        assert_eq!(caps.nvme_sanitize_block, TriState::Unsupported);
        assert_eq!(caps.nvme_sanitize_crypto, TriState::Unsupported);
        assert_eq!(caps.nvme_sanitize_overwrite, TriState::Unsupported);
        assert_eq!(caps.scsi_sanitize, TriState::Unknown);
    }

    #[test]
    fn test_legacy_safe_capabilities_both_false() {
        let caps = evaluate_safe_capabilities(Some(false), Some(false));

        assert_eq!(caps.ata_secure_erase, TriState::Unsupported);
        assert_eq!(caps.ata_enhanced_secure_erase, TriState::Unsupported);
        assert_eq!(caps.ata_sanitize_block, TriState::Unsupported);
        assert_eq!(caps.ata_sanitize_crypto, TriState::Unsupported);
        assert_eq!(caps.ata_sanitize_overwrite, TriState::Unsupported);

        assert_eq!(caps.nvme_format, TriState::Unsupported);
        assert_eq!(caps.nvme_format_crypto, TriState::Unsupported);
        assert_eq!(caps.nvme_sanitize_block, TriState::Unsupported);
        assert_eq!(caps.nvme_sanitize_crypto, TriState::Unsupported);
        assert_eq!(caps.nvme_sanitize_overwrite, TriState::Unsupported);

        assert_eq!(caps.scsi_sanitize, TriState::Unknown);
    }

    #[test]
    fn test_a_no_evidence_all_unknown() {
        let evidence = CapabilityEvidence::empty();
        let caps = evaluate_capabilities_from_evidence(&evidence);

        assert_eq!(caps.ata_secure_erase, TriState::Unknown);
        assert_eq!(caps.ata_enhanced_secure_erase, TriState::Unknown);
        assert_eq!(caps.ata_sanitize_block, TriState::Unknown);
        assert_eq!(caps.ata_sanitize_crypto, TriState::Unknown);
        assert_eq!(caps.ata_sanitize_overwrite, TriState::Unknown);
        assert_eq!(caps.nvme_format, TriState::Unknown);
        assert_eq!(caps.nvme_format_crypto, TriState::Unknown);
        assert_eq!(caps.nvme_sanitize_block, TriState::Unknown);
        assert_eq!(caps.nvme_sanitize_crypto, TriState::Unknown);
        assert_eq!(caps.nvme_sanitize_overwrite, TriState::Unknown);
        assert_eq!(caps.scsi_sanitize, TriState::Unknown);
    }

    #[test]
    fn test_b_ata_evidence_supported_and_unsupported_mapped_correctly() {
        let evidence = CapabilityEvidence {
            ata: Some(AtaCapabilityEvidence {
                secure_erase_supported: TriState::Supported,
                enhanced_secure_erase_supported: TriState::Unsupported,
                sanitize_block_supported: TriState::Supported,
                sanitize_crypto_supported: TriState::Unsupported,
                sanitize_overwrite_supported: TriState::Unknown,
            }),
            nvme: None,
            scsi: None,
        };

        let caps = evaluate_capabilities_from_evidence(&evidence);

        assert_eq!(caps.ata_secure_erase, TriState::Supported);
        assert_eq!(caps.ata_enhanced_secure_erase, TriState::Unsupported);
        assert_eq!(caps.ata_sanitize_block, TriState::Supported);
        assert_eq!(caps.ata_sanitize_crypto, TriState::Unsupported);
        assert_eq!(caps.ata_sanitize_overwrite, TriState::Unknown);
    }

    #[test]
    fn test_c_nvme_evidence_supported_and_unsupported_mapped_correctly() {
        let evidence = CapabilityEvidence {
            ata: None,
            nvme: Some(NvmeCapabilityEvidence {
                format_supported: TriState::Supported,
                format_crypto_supported: TriState::Supported,
                sanitize_block_supported: TriState::Unsupported,
                sanitize_crypto_supported: TriState::Supported,
                sanitize_overwrite_supported: TriState::Unsupported,
            }),
            scsi: None,
        };

        let caps = evaluate_capabilities_from_evidence(&evidence);

        assert_eq!(caps.nvme_format, TriState::Supported);
        assert_eq!(caps.nvme_format_crypto, TriState::Supported);
        assert_eq!(caps.nvme_sanitize_block, TriState::Unsupported);
        assert_eq!(caps.nvme_sanitize_crypto, TriState::Supported);
        assert_eq!(caps.nvme_sanitize_overwrite, TriState::Unsupported);
    }

    #[test]
    fn test_d_unknown_fields_remain_unknown() {
        let evidence = CapabilityEvidence {
            ata: None,
            nvme: Some(NvmeCapabilityEvidence {
                format_supported: TriState::Supported,
                format_crypto_supported: TriState::Unknown,
                sanitize_block_supported: TriState::Unknown,
                sanitize_crypto_supported: TriState::Supported,
                sanitize_overwrite_supported: TriState::Unknown,
            }),
            scsi: None,
        };

        let caps = evaluate_capabilities_from_evidence(&evidence);

        assert_eq!(caps.nvme_format, TriState::Supported);
        assert_eq!(caps.nvme_format_crypto, TriState::Unknown);
        assert_eq!(caps.nvme_sanitize_block, TriState::Unknown);
        assert_eq!(caps.nvme_sanitize_crypto, TriState::Supported);
        assert_eq!(caps.nvme_sanitize_overwrite, TriState::Unknown);
    }

    #[test]
    fn test_f_ata_evidence_does_not_accidentally_modify_nvme_capabilities() {
        let evidence = CapabilityEvidence {
            ata: Some(AtaCapabilityEvidence {
                secure_erase_supported: TriState::Supported,
                enhanced_secure_erase_supported: TriState::Supported,
                sanitize_block_supported: TriState::Supported,
                sanitize_crypto_supported: TriState::Supported,
                sanitize_overwrite_supported: TriState::Supported,
            }),
            nvme: None,
            scsi: None,
        };

        let caps = evaluate_capabilities_from_evidence(&evidence);

        assert_eq!(caps.ata_secure_erase, TriState::Supported);
        assert_eq!(caps.nvme_format, TriState::Unknown);
        assert_eq!(caps.nvme_format_crypto, TriState::Unknown);
        assert_eq!(caps.nvme_sanitize_block, TriState::Unknown);
        assert_eq!(caps.nvme_sanitize_crypto, TriState::Unknown);
        assert_eq!(caps.nvme_sanitize_overwrite, TriState::Unknown);
    }

    #[test]
    fn test_g_nvme_evidence_does_not_accidentally_modify_ata_capabilities() {
        let evidence = CapabilityEvidence {
            ata: None,
            nvme: Some(NvmeCapabilityEvidence {
                format_supported: TriState::Supported,
                format_crypto_supported: TriState::Supported,
                sanitize_block_supported: TriState::Supported,
                sanitize_crypto_supported: TriState::Supported,
                sanitize_overwrite_supported: TriState::Supported,
            }),
            scsi: None,
        };

        let caps = evaluate_capabilities_from_evidence(&evidence);

        assert_eq!(caps.nvme_sanitize_crypto, TriState::Supported);
        assert_eq!(caps.ata_secure_erase, TriState::Unknown);
        assert_eq!(caps.ata_enhanced_secure_erase, TriState::Unknown);
        assert_eq!(caps.ata_sanitize_block, TriState::Unknown);
        assert_eq!(caps.ata_sanitize_crypto, TriState::Unknown);
        assert_eq!(caps.ata_sanitize_overwrite, TriState::Unknown);
    }

    #[test]
    fn test_h_scsi_capability_remains_unknown_unless_explicit_scsi_evidence() {
        let no_scsi = CapabilityEvidence {
            ata: Some(AtaCapabilityEvidence::default()),
            nvme: None,
            scsi: None,
        };
        let caps1 = evaluate_capabilities_from_evidence(&no_scsi);
        assert_eq!(caps1.scsi_sanitize, TriState::Unknown);

        let with_scsi = CapabilityEvidence {
            ata: None,
            nvme: None,
            scsi: Some(ScsiCapabilityEvidence {
                sanitize_supported: TriState::Supported,
            }),
        };
        let caps2 = evaluate_capabilities_from_evidence(&with_scsi);
        assert_eq!(caps2.scsi_sanitize, TriState::Supported);
    }
}
