//! Step 5.2 — Sanitization Method Selection & Proposal Layer.
//!
//! # Strict Design Invariants
//! 1. Non-Destructive: Pure selection logic only. It does not execute commands,
//!    issue IOCTLs, perform write operations, modify partition tables, or format drives.
//! 2. Epistemic Certainty: Method proposals are based solely on explicit, verified capability
//!    evidence (`TriState::Supported`).
//! 3. No Guessing: Bus type, media type, vendor/model strings, and drive capacity MUST NEVER be
//!    used to infer method availability.
//! 4. TriState Discipline:
//!    - `TriState::Supported` -> Eligible for proposal.
//!    - `TriState::Unsupported` -> Ineligible; strictly excluded from supported proposals.
//!    - `TriState::Unknown` -> Ineligible; never assumed to be supported.
//! 5. Global Policy, Not Protocol-Biased: Prioritization strictly follows the ForenX functional
//!    tier hierarchy across all protocols (Crypto -> Block Erase -> Overwrite -> Firmware Secure Erase -> SCSI Sanitize).

use crate::model::{DriveCapabilities, TriState};
use serde::{Deserialize, Serialize};

/// Strongly typed device-level sanitization methods that ForenX can evaluate and propose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SanitizationMethod {
    // ATA Methods
    AtaSanitizeCrypto,
    AtaSanitizeBlock,
    AtaSanitizeOverwrite,
    AtaEnhancedSecureErase,
    AtaSecureErase,

    // NVMe Methods
    NvmeSanitizeCrypto,
    NvmeSanitizeBlock,
    NvmeSanitizeOverwrite,
    NvmeFormatCrypto,

    // SCSI Methods
    ScsiSanitize,
}

impl SanitizationMethod {
    /// Human-readable display label for UI and reporting.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::AtaSanitizeCrypto => "ATA Sanitize (Crypto Scramble)",
            Self::AtaSanitizeBlock => "ATA Sanitize (Block Erase)",
            Self::AtaSanitizeOverwrite => "ATA Sanitize (Overwrite)",
            Self::AtaEnhancedSecureErase => "ATA Enhanced Secure Erase",
            Self::AtaSecureErase => "ATA Secure Erase",
            Self::NvmeSanitizeCrypto => "NVMe Sanitize (Crypto Erase)",
            Self::NvmeSanitizeBlock => "NVMe Sanitize (Block Erase)",
            Self::NvmeSanitizeOverwrite => "NVMe Sanitize (Overwrite)",
            Self::NvmeFormatCrypto => "NVMe Format (User Data Erase with Crypto)",
            Self::ScsiSanitize => "SCSI Sanitize",
        }
    }

    /// Technical mechanism summary describing the command interface without absolute physical guarantees.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::AtaSanitizeCrypto => {
                "Issues an ATA Sanitize command requesting cryptographic key destruction per ATA specifications."
            }
            Self::AtaSanitizeBlock => {
                "Issues an ATA Sanitize command requesting controller-managed block erasure per ATA specifications."
            }
            Self::AtaSanitizeOverwrite => {
                "Issues an ATA Sanitize command requesting controller-managed pattern overwrite per ATA specifications."
            }
            Self::AtaEnhancedSecureErase => {
                "Issues an ATA Security Erase Prepare/Unit command requesting vendor-defined enhanced erase routines."
            }
            Self::AtaSecureErase => {
                "Issues an ATA Security Erase Prepare/Unit command requesting standard firmware-level erase routines."
            }
            Self::NvmeSanitizeCrypto => {
                "Issues an NVMe Sanitize command requesting cryptographic key destruction per NVMe specifications."
            }
            Self::NvmeSanitizeBlock => {
                "Issues an NVMe Sanitize command requesting controller-managed block erasure per NVMe specifications."
            }
            Self::NvmeSanitizeOverwrite => {
                "Issues an NVMe Sanitize command requesting controller-managed pattern overwrite per NVMe specifications."
            }
            Self::NvmeFormatCrypto => {
                "Issues an NVMe Format NVM command specifying cryptographic erase in the User Data Erase field."
            }
            Self::ScsiSanitize => {
                "Issues a SCSI Sanitize command requesting controller-managed sanitization per SPC specifications."
            }
        }
    }

    /// Global deterministic ranking rank based on ForenX internal selection policy.
    ///
    /// Lower rank number corresponds to higher priority:
    /// Tier 1: Controller-level cryptographic erase (NvmeSanitizeCrypto, AtaSanitizeCrypto, NvmeFormatCrypto)
    /// Tier 2: Controller-level physical block erase (NvmeSanitizeBlock, AtaSanitizeBlock)
    /// Tier 3: Controller-level overwrite (NvmeSanitizeOverwrite, AtaSanitizeOverwrite)
    /// Tier 4: Firmware-level security erase (AtaEnhancedSecureErase, AtaSecureErase)
    /// Tier 5: SCSI controller-level sanitize (ScsiSanitize)
    #[must_use]
    pub const fn global_rank(&self) -> u8 {
        match self {
            // Tier 1: Cryptographic Invalidation
            Self::NvmeSanitizeCrypto => 10,
            Self::AtaSanitizeCrypto => 11,
            Self::NvmeFormatCrypto => 12,

            // Tier 2: Block Erase
            Self::NvmeSanitizeBlock => 20,
            Self::AtaSanitizeBlock => 21,

            // Tier 3: Controller Overwrite
            Self::NvmeSanitizeOverwrite => 30,
            Self::AtaSanitizeOverwrite => 31,

            // Tier 4: Firmware Security Erase
            Self::AtaEnhancedSecureErase => 40,
            Self::AtaSecureErase => 41,

            // Tier 5: SCSI Sanitize
            Self::ScsiSanitize => 50,
        }
    }
}

/// A method proposal returned by the selection engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MethodCandidate {
    pub method: SanitizationMethod,
    pub reason: String,
}

/// Comprehensive method proposal result for a target drive.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MethodProposal {
    /// The primary proposed method, deterministically selected according to ForenX internal selection policy.
    pub primary_recommendation: Option<MethodCandidate>,
    /// Additional explicitly supported candidate methods, ordered deterministically by ForenX internal selection policy.
    pub alternative_candidates: Vec<MethodCandidate>,
}

/// Evaluates verified drive capabilities and generates a deterministic sanitization proposal.
///
/// # ForenX Global Selection Policy
/// Candidate evaluation collects all methods explicitly marked as `TriState::Supported`,
/// then sorts them globally by functional tier rather than by storage protocol:
/// 1. Cryptographic sanitize / format operations
/// 2. Controller-level block erase operations
/// 3. Controller-level pattern overwrite operations
/// 4. Firmware-level security erase operations
/// 5. SCSI controller-level sanitize operations
///
/// If no method is confirmed as `TriState::Supported`, `primary_recommendation` is `None`
/// and `alternative_candidates` is empty.
#[must_use]
pub fn select_sanitization_methods(caps: &DriveCapabilities) -> MethodProposal {
    let mut supported: Vec<MethodCandidate> = Vec::new();

    // Check all capability flags independently against TriState::Supported
    if caps.nvme_sanitize_crypto == TriState::Supported {
        supported.push(MethodCandidate {
            method: SanitizationMethod::NvmeSanitizeCrypto,
            reason: "NVMe controller explicitly reports Sanitize Crypto Erase support".into(),
        });
    }
    if caps.nvme_format_crypto == TriState::Supported {
        supported.push(MethodCandidate {
            method: SanitizationMethod::NvmeFormatCrypto,
            reason: "NVMe controller explicitly reports Format with Crypto Erase support".into(),
        });
    }
    if caps.nvme_sanitize_block == TriState::Supported {
        supported.push(MethodCandidate {
            method: SanitizationMethod::NvmeSanitizeBlock,
            reason: "NVMe controller explicitly reports Sanitize Block Erase support".into(),
        });
    }
    if caps.nvme_sanitize_overwrite == TriState::Supported {
        supported.push(MethodCandidate {
            method: SanitizationMethod::NvmeSanitizeOverwrite,
            reason: "NVMe controller explicitly reports Sanitize Overwrite support".into(),
        });
    }
    if caps.ata_sanitize_crypto == TriState::Supported {
        supported.push(MethodCandidate {
            method: SanitizationMethod::AtaSanitizeCrypto,
            reason: "ATA device explicitly reports Sanitize Crypto Scramble support".into(),
        });
    }
    if caps.ata_sanitize_block == TriState::Supported {
        supported.push(MethodCandidate {
            method: SanitizationMethod::AtaSanitizeBlock,
            reason: "ATA device explicitly reports Sanitize Block Erase support".into(),
        });
    }
    if caps.ata_sanitize_overwrite == TriState::Supported {
        supported.push(MethodCandidate {
            method: SanitizationMethod::AtaSanitizeOverwrite,
            reason: "ATA device explicitly reports Sanitize Overwrite support".into(),
        });
    }
    if caps.ata_enhanced_secure_erase == TriState::Supported {
        supported.push(MethodCandidate {
            method: SanitizationMethod::AtaEnhancedSecureErase,
            reason: "ATA device explicitly reports Enhanced Secure Erase support".into(),
        });
    }
    if caps.ata_secure_erase == TriState::Supported {
        supported.push(MethodCandidate {
            method: SanitizationMethod::AtaSecureErase,
            reason: "ATA device explicitly reports Secure Erase support".into(),
        });
    }
    if caps.scsi_sanitize == TriState::Supported {
        supported.push(MethodCandidate {
            method: SanitizationMethod::ScsiSanitize,
            reason: "SCSI controller explicitly reports Sanitize support".into(),
        });
    }

    if supported.is_empty() {
        return MethodProposal {
            primary_recommendation: None,
            alternative_candidates: Vec::new(),
        };
    }

    // Sort globally by documented ForenX policy priority
    supported.sort_by_key(|c| c.method.global_rank());

    let primary = supported.remove(0);
    MethodProposal {
        primary_recommendation: Some(primary),
        alternative_candidates: supported,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::default_readonly_capabilities;

    #[test]
    fn test_1_no_capability_evidence_yields_no_recommendation() {
        let caps = default_readonly_capabilities();
        let proposal = select_sanitization_methods(&caps);

        assert!(proposal.primary_recommendation.is_none());
        assert!(proposal.alternative_candidates.is_empty());
    }

    #[test]
    fn test_2_ata_secure_erase_supported_proposed() {
        let mut caps = default_readonly_capabilities();
        caps.ata_secure_erase = TriState::Supported;

        let proposal = select_sanitization_methods(&caps);
        let primary = proposal
            .primary_recommendation
            .expect("Primary recommendation expected");

        assert_eq!(primary.method, SanitizationMethod::AtaSecureErase);
        assert!(proposal.alternative_candidates.is_empty());
    }

    #[test]
    fn test_3_ata_secure_erase_unsupported_not_proposed() {
        let mut caps = default_readonly_capabilities();
        caps.ata_secure_erase = TriState::Unsupported;

        let proposal = select_sanitization_methods(&caps);
        assert!(proposal.primary_recommendation.is_none());
        assert!(proposal.alternative_candidates.is_empty());
    }

    #[test]
    fn test_4_ata_secure_erase_unknown_not_proposed() {
        let mut caps = default_readonly_capabilities();
        caps.ata_secure_erase = TriState::Unknown;

        let proposal = select_sanitization_methods(&caps);
        assert!(proposal.primary_recommendation.is_none());
        assert!(proposal.alternative_candidates.is_empty());
    }

    #[test]
    fn test_5_ata_multiple_supported_methods_deterministic_ordering() {
        let mut caps = default_readonly_capabilities();
        caps.ata_secure_erase = TriState::Supported;
        caps.ata_enhanced_secure_erase = TriState::Supported;
        caps.ata_sanitize_block = TriState::Supported;
        caps.ata_sanitize_crypto = TriState::Supported;

        let proposal = select_sanitization_methods(&caps);
        let primary = proposal.primary_recommendation.expect("Must have primary");

        assert_eq!(primary.method, SanitizationMethod::AtaSanitizeCrypto);

        let alt_methods: Vec<SanitizationMethod> = proposal
            .alternative_candidates
            .iter()
            .map(|c| c.method)
            .collect();

        assert_eq!(
            alt_methods,
            vec![
                SanitizationMethod::AtaSanitizeBlock,
                SanitizationMethod::AtaEnhancedSecureErase,
                SanitizationMethod::AtaSecureErase,
            ]
        );
    }

    #[test]
    fn test_6_ata_mixed_supported_unsupported_unknown() {
        let mut caps = default_readonly_capabilities();
        caps.ata_secure_erase = TriState::Supported;
        caps.ata_enhanced_secure_erase = TriState::Unsupported;
        caps.ata_sanitize_block = TriState::Supported;
        caps.ata_sanitize_crypto = TriState::Unknown;
        caps.ata_sanitize_overwrite = TriState::Unsupported;

        let proposal = select_sanitization_methods(&caps);
        let primary = proposal.primary_recommendation.expect("Must have primary");

        assert_eq!(primary.method, SanitizationMethod::AtaSanitizeBlock);
        assert_eq!(proposal.alternative_candidates.len(), 1);
        assert_eq!(
            proposal.alternative_candidates[0].method,
            SanitizationMethod::AtaSecureErase
        );
    }

    #[test]
    fn test_7_nvme_crypto_supported_proposed() {
        let mut caps = default_readonly_capabilities();
        caps.nvme_sanitize_crypto = TriState::Supported;

        let proposal = select_sanitization_methods(&caps);
        let primary = proposal.primary_recommendation.expect("Must have primary");

        assert_eq!(primary.method, SanitizationMethod::NvmeSanitizeCrypto);
    }

    #[test]
    fn test_8_nvme_crypto_unknown_not_proposed() {
        let mut caps = default_readonly_capabilities();
        caps.nvme_sanitize_crypto = TriState::Unknown;
        caps.nvme_format_crypto = TriState::Unknown;

        let proposal = select_sanitization_methods(&caps);
        assert!(proposal.primary_recommendation.is_none());
    }

    #[test]
    fn test_9_nvme_capability_unsupported_not_proposed() {
        let mut caps = default_readonly_capabilities();
        caps.nvme_sanitize_block = TriState::Unsupported;
        caps.nvme_sanitize_crypto = TriState::Unsupported;

        let proposal = select_sanitization_methods(&caps);
        assert!(proposal.primary_recommendation.is_none());
    }

    #[test]
    fn test_10_scsi_sanitize_supported_proposed() {
        let mut caps = default_readonly_capabilities();
        caps.scsi_sanitize = TriState::Supported;

        let proposal = select_sanitization_methods(&caps);
        let primary = proposal.primary_recommendation.expect("Must have primary");

        assert_eq!(primary.method, SanitizationMethod::ScsiSanitize);
    }

    #[test]
    fn test_11_scsi_sanitize_unknown_not_proposed() {
        let mut caps = default_readonly_capabilities();
        caps.scsi_sanitize = TriState::Unknown;

        let proposal = select_sanitization_methods(&caps);
        assert!(proposal.primary_recommendation.is_none());
    }

    #[test]
    fn test_12_determinism_identical_results_across_calls() {
        let mut caps = default_readonly_capabilities();
        caps.nvme_sanitize_crypto = TriState::Supported;
        caps.nvme_sanitize_block = TriState::Supported;
        caps.nvme_format_crypto = TriState::Supported;

        let proposal_1 = select_sanitization_methods(&caps);
        let proposal_2 = select_sanitization_methods(&caps);

        assert_eq!(proposal_1, proposal_2);
    }

    #[test]
    fn test_13_no_hardware_access_pure_logic() {
        let mut caps = default_readonly_capabilities();
        caps.scsi_sanitize = TriState::Supported;
        caps.nvme_sanitize_block = TriState::Supported;

        let proposal = select_sanitization_methods(&caps);
        assert!(proposal.primary_recommendation.is_some());
    }

    // --- Cross-protocol tests verifying global priority ordering ---

    #[test]
    fn test_cross_protocol_a_nvme_block_vs_ata_crypto() {
        let mut caps = default_readonly_capabilities();
        caps.nvme_sanitize_block = TriState::Supported;
        caps.ata_sanitize_crypto = TriState::Supported;

        let proposal = select_sanitization_methods(&caps);
        let primary = proposal
            .primary_recommendation
            .expect("Primary recommendation expected");

        assert_eq!(primary.method, SanitizationMethod::AtaSanitizeCrypto);
        assert_eq!(
            proposal.alternative_candidates[0].method,
            SanitizationMethod::NvmeSanitizeBlock
        );
    }

    #[test]
    fn test_cross_protocol_b_ata_block_vs_nvme_crypto() {
        let mut caps = default_readonly_capabilities();
        caps.ata_sanitize_block = TriState::Supported;
        caps.nvme_sanitize_crypto = TriState::Supported;

        let proposal = select_sanitization_methods(&caps);
        let primary = proposal
            .primary_recommendation
            .expect("Primary recommendation expected");

        assert_eq!(primary.method, SanitizationMethod::NvmeSanitizeCrypto);
        assert_eq!(
            proposal.alternative_candidates[0].method,
            SanitizationMethod::AtaSanitizeBlock
        );
    }

    #[test]
    fn test_cross_protocol_c_ata_secure_erase_vs_nvme_block() {
        let mut caps = default_readonly_capabilities();
        caps.ata_secure_erase = TriState::Supported;
        caps.nvme_sanitize_block = TriState::Supported;

        let proposal = select_sanitization_methods(&caps);
        let primary = proposal
            .primary_recommendation
            .expect("Primary recommendation expected");

        assert_eq!(primary.method, SanitizationMethod::NvmeSanitizeBlock);
        assert_eq!(
            proposal.alternative_candidates[0].method,
            SanitizationMethod::AtaSecureErase
        );
    }

    #[test]
    fn test_cross_protocol_d_ata_overwrite_vs_nvme_block() {
        let mut caps = default_readonly_capabilities();
        caps.ata_sanitize_overwrite = TriState::Supported;
        caps.nvme_sanitize_block = TriState::Supported;

        let proposal = select_sanitization_methods(&caps);
        let primary = proposal
            .primary_recommendation
            .expect("Primary recommendation expected");

        assert_eq!(primary.method, SanitizationMethod::NvmeSanitizeBlock);
        assert_eq!(
            proposal.alternative_candidates[0].method,
            SanitizationMethod::AtaSanitizeOverwrite
        );
    }

    #[test]
    fn test_cross_protocol_e_multiple_supported_across_ata_nvme_scsi() {
        let mut caps = default_readonly_capabilities();
        caps.scsi_sanitize = TriState::Supported;
        caps.ata_secure_erase = TriState::Supported;
        caps.ata_sanitize_overwrite = TriState::Supported;
        caps.ata_sanitize_crypto = TriState::Supported;
        caps.nvme_sanitize_block = TriState::Supported;
        caps.nvme_format_crypto = TriState::Supported;

        let proposal = select_sanitization_methods(&caps);
        let primary = proposal.primary_recommendation.expect("Must have primary");

        // Tier 1 Crypto (ATA Sanitize Crypto or NVMe Format Crypto)
        assert_eq!(primary.method, SanitizationMethod::AtaSanitizeCrypto);

        let alt_methods: Vec<SanitizationMethod> = proposal
            .alternative_candidates
            .iter()
            .map(|c| c.method)
            .collect();

        // Tier 1 -> Tier 2 -> Tier 3 -> Tier 4 -> Tier 5
        assert_eq!(
            alt_methods,
            vec![
                SanitizationMethod::NvmeFormatCrypto,     // Tier 1 (Crypto)
                SanitizationMethod::NvmeSanitizeBlock,    // Tier 2 (Block Erase)
                SanitizationMethod::AtaSanitizeOverwrite, // Tier 3 (Overwrite)
                SanitizationMethod::AtaSecureErase,       // Tier 4 (Firmware Secure Erase)
                SanitizationMethod::ScsiSanitize,         // Tier 5 (SCSI Sanitize)
            ]
        );
    }
}
