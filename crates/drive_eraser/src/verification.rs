//! Step 6.2 — Drive Verification (Mock Layer).
//!
//! # Strict Design Invariants
//! 1. Non-Destructive: Pure in-memory evaluation logic. Performs NO physical storage reads,
//!    NO PhysicalDrive opens, NO IOCTL queries, and NO device-level modifications.
//! 2. Explicit Epistemic Verdicts: Clearly distinguishes `Verified`, `VerificationFailed`, and
//!    `NotVerifiable`. Missing or unobservable evidence produces `NotVerifiable`, never false failure or false pass.
//! 3. No Physical Erasure Guarantees: All outcomes explicitly state that this is simulated verification
//!    and does not prove physical media, NAND cell, reallocated sector, or magnetic surface erasure.
//! 4. Determinism: Identical inputs produce identical verification verdicts and audit records.

use crate::engine::MockExecutionResult;
use crate::error::{DriveEraserError, Result};
use crate::method_select::SanitizationMethod;
use serde::{Deserialize, Serialize};

/// High-level verdict of the verification evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationVerdict {
    /// The mock verification criteria were fully evaluated and passed.
    Verified,
    /// The mock verification criteria were evaluated and definitively failed.
    VerificationFailed,
    /// The available mock evidence is missing or insufficient to make a definitive determination.
    NotVerifiable,
}

impl VerificationVerdict {
    /// Returns true only if the verdict is definitively verified.
    #[must_use]
    pub const fn is_verified(&self) -> bool {
        matches!(self, Self::Verified)
    }
}

/// Request to evaluate verification of a simulated drive erasure operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MockVerificationRequest {
    /// Target physical drive identifier (e.g., "MockDrive_Verify1").
    pub target_device_id: String,
    /// The sanitization method that was evaluated.
    pub method: SanitizationMethod,
    /// Observed simulated verification token, typically captured from `MockExecutionResult`.
    pub observed_token: Option<String>,
    /// Expected verification token used for deterministic validation matching.
    pub expected_token: Option<String>,
}

/// Comprehensive outcome of the mock verification assessment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MockVerificationResult {
    /// Target physical drive identifier.
    pub target_device_id: String,
    /// Sanitization method associated with the verification check.
    pub method: SanitizationMethod,
    /// Definitive verification verdict.
    pub verdict: VerificationVerdict,
    /// Detailed diagnostic / audit message explaining the outcome.
    pub details: String,
    /// Expected verification token, if supplied.
    pub expected_token: Option<String>,
    /// Observed verification token, if supplied.
    pub observed_token: Option<String>,
    /// Explicit flag marking that this is a simulated verification and NOT a physical drive audit.
    pub is_simulated_mock: bool,
}

/// Evaluates a mock verification request deterministically in memory.
///
/// # Semantics
/// - Missing target identifier -> `DriveEraserError::ExecutionFailed`.
/// - Missing observed token OR missing expected token -> `VerificationVerdict::NotVerifiable`.
/// - Observed token matches expected token -> `VerificationVerdict::Verified`.
/// - Observed token does not match expected token -> `VerificationVerdict::VerificationFailed`.
pub fn verify_mock_erasure(request: &MockVerificationRequest) -> Result<MockVerificationResult> {
    if request.target_device_id.trim().is_empty() {
        return Err(DriveEraserError::ExecutionFailed {
            device_id: request.target_device_id.clone(),
            message: "Target device identifier cannot be empty for verification".into(),
        });
    }

    match (&request.observed_token, &request.expected_token) {
        (None, None) => Ok(MockVerificationResult {
            target_device_id: request.target_device_id.clone(),
            method: request.method,
            verdict: VerificationVerdict::NotVerifiable,
            details: "SIMULATED MOCK VERIFICATION: Neither observed nor expected verification evidence was provided."
                .into(),
            expected_token: None,
            observed_token: None,
            is_simulated_mock: true,
        }),
        (None, Some(expected)) => Ok(MockVerificationResult {
            target_device_id: request.target_device_id.clone(),
            method: request.method,
            verdict: VerificationVerdict::NotVerifiable,
            details: "SIMULATED MOCK VERIFICATION: Observed verification evidence is missing from the execution outcome."
                .into(),
            expected_token: Some(expected.clone()),
            observed_token: None,
            is_simulated_mock: true,
        }),
        (Some(observed), None) => Ok(MockVerificationResult {
            target_device_id: request.target_device_id.clone(),
            method: request.method,
            verdict: VerificationVerdict::NotVerifiable,
            details: "SIMULATED MOCK VERIFICATION: Expected verification evidence was omitted; cannot perform comparison."
                .into(),
            expected_token: None,
            observed_token: Some(observed.clone()),
            is_simulated_mock: true,
        }),
        (Some(observed), Some(expected)) => {
            if observed == expected {
                Ok(MockVerificationResult {
                    target_device_id: request.target_device_id.clone(),
                    method: request.method,
                    verdict: VerificationVerdict::Verified,
                    details: "SIMULATED MOCK VERIFICATION: Simulated verification passed. Observed token matches expected token. No physical sectors were checked."
                        .into(),
                    expected_token: Some(expected.clone()),
                    observed_token: Some(observed.clone()),
                    is_simulated_mock: true,
                })
            } else {
                Ok(MockVerificationResult {
                    target_device_id: request.target_device_id.clone(),
                    method: request.method,
                    verdict: VerificationVerdict::VerificationFailed,
                    details: format!(
                        "SIMULATED MOCK VERIFICATION: Verification criteria mismatch. Expected token '{expected}', found observed token '{observed}'."
                    ),
                    expected_token: Some(expected.clone()),
                    observed_token: Some(observed.clone()),
                    is_simulated_mock: true,
                })
            }
        }
    }
}

/// Convenience helper to verify the output of a `MockExecutionResult` directly against an expected token.
pub fn verify_mock_execution_result(
    execution_result: &MockExecutionResult,
    expected_token: Option<&str>,
) -> Result<MockVerificationResult> {
    let request = MockVerificationRequest {
        target_device_id: execution_result.target_device_id.clone(),
        method: execution_result.method,
        observed_token: execution_result.simulated_verification_token.clone(),
        expected_token: expected_token.map(ToOwned::to_owned),
    };
    verify_mock_erasure(&request)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::MockExecutionOutcome;

    #[test]
    fn test_1_matching_token_produces_verified() {
        let request = MockVerificationRequest {
            target_device_id: "MockDrive_Verify1".into(),
            method: SanitizationMethod::NvmeSanitizeCrypto,
            observed_token: Some("MOCK-TOKEN-1234".into()),
            expected_token: Some("MOCK-TOKEN-1234".into()),
        };

        let result = verify_mock_erasure(&request).expect("Verification logic must succeed");
        assert_eq!(result.verdict, VerificationVerdict::Verified);
        assert!(result.verdict.is_verified());
        assert!(result.is_simulated_mock);
        assert!(result.details.contains("Simulated verification passed"));
        assert_eq!(result.target_device_id, "MockDrive_Verify1");
    }

    #[test]
    fn test_2_mismatching_token_produces_verification_failed() {
        let request = MockVerificationRequest {
            target_device_id: "MockDrive_Verify2".into(),
            method: SanitizationMethod::AtaSanitizeBlock,
            observed_token: Some("MOCK-TOKEN-OBSERVED".into()),
            expected_token: Some("MOCK-TOKEN-EXPECTED".into()),
        };

        let result = verify_mock_erasure(&request).expect("Verification logic must succeed");
        assert_eq!(result.verdict, VerificationVerdict::VerificationFailed);
        assert!(!result.verdict.is_verified());
        assert!(result.is_simulated_mock);
        assert!(result.details.contains("criteria mismatch"));
    }

    #[test]
    fn test_3_missing_expected_token_produces_not_verifiable() {
        let request = MockVerificationRequest {
            target_device_id: "MockDrive_Verify3".into(),
            method: SanitizationMethod::AtaSecureErase,
            observed_token: Some("MOCK-TOKEN-XYZ".into()),
            expected_token: None,
        };

        let result = verify_mock_erasure(&request).expect("Verification logic must succeed");
        assert_eq!(result.verdict, VerificationVerdict::NotVerifiable);
        assert!(!result.verdict.is_verified());
        assert!(result.is_simulated_mock);
        assert!(result
            .details
            .contains("Expected verification evidence was omitted"));
    }

    #[test]
    fn test_4_missing_observed_token_produces_not_verifiable() {
        let request = MockVerificationRequest {
            target_device_id: "MockDrive_Verify4".into(),
            method: SanitizationMethod::NvmeSanitizeBlock,
            observed_token: None,
            expected_token: Some("MOCK-TOKEN-XYZ".into()),
        };

        let result = verify_mock_erasure(&request).expect("Verification logic must succeed");
        assert_eq!(result.verdict, VerificationVerdict::NotVerifiable);
        assert!(!result.verdict.is_verified());
        assert!(result.is_simulated_mock);
        assert!(result
            .details
            .contains("Observed verification evidence is missing"));
    }

    #[test]
    fn test_5_both_tokens_missing_produces_not_verifiable() {
        let request = MockVerificationRequest {
            target_device_id: "MockDrive_Verify5".into(),
            method: SanitizationMethod::ScsiSanitize,
            observed_token: None,
            expected_token: None,
        };

        let result = verify_mock_erasure(&request).expect("Verification logic must succeed");
        assert_eq!(result.verdict, VerificationVerdict::NotVerifiable);
        assert!(!result.verdict.is_verified());
        assert!(result.is_simulated_mock);
        assert!(result
            .details
            .contains("Neither observed nor expected verification evidence was provided"));
    }

    #[test]
    fn test_6_empty_or_invalid_target_identifier_handled_safely() {
        let request = MockVerificationRequest {
            target_device_id: "   ".into(),
            method: SanitizationMethod::NvmeSanitizeOverwrite,
            observed_token: Some("TOKEN".into()),
            expected_token: Some("TOKEN".into()),
        };

        let err = verify_mock_erasure(&request).expect_err("Empty device ID must return error");
        match err {
            DriveEraserError::ExecutionFailed { message, .. } => {
                assert!(message.contains("cannot be empty"));
            }
            other => panic!("Expected ExecutionFailed, got: {:?}", other),
        }
    }

    #[test]
    fn test_7_correct_sanitization_method_preserved() {
        let methods = [
            SanitizationMethod::AtaSanitizeCrypto,
            SanitizationMethod::AtaSanitizeBlock,
            SanitizationMethod::AtaSanitizeOverwrite,
            SanitizationMethod::AtaEnhancedSecureErase,
            SanitizationMethod::AtaSecureErase,
            SanitizationMethod::NvmeSanitizeCrypto,
            SanitizationMethod::NvmeSanitizeBlock,
            SanitizationMethod::NvmeSanitizeOverwrite,
            SanitizationMethod::NvmeFormatCrypto,
            SanitizationMethod::ScsiSanitize,
        ];

        for m in methods {
            let request = MockVerificationRequest {
                target_device_id: "MockDrive_Preserve".into(),
                method: m,
                observed_token: Some("T".into()),
                expected_token: Some("T".into()),
            };
            let res = verify_mock_erasure(&request).expect("Verification must succeed");
            assert_eq!(res.method, m);
        }
    }

    #[test]
    fn test_8_result_clearly_identifies_as_simulated_mock() {
        let request = MockVerificationRequest {
            target_device_id: "MockDrive_SimCheck".into(),
            method: SanitizationMethod::AtaSecureErase,
            observed_token: Some("T".into()),
            expected_token: Some("T".into()),
        };

        let res = verify_mock_erasure(&request).expect("Must succeed");
        assert!(res.is_simulated_mock);
        assert!(res.details.contains("SIMULATED MOCK VERIFICATION"));
        assert!(res.details.contains("No physical sectors were checked"));
    }

    #[test]
    fn test_9_deterministic_identical_input_produces_identical_result() {
        let request1 = MockVerificationRequest {
            target_device_id: "MockDrive_DetVerify".into(),
            method: SanitizationMethod::NvmeFormatCrypto,
            observed_token: Some("TOKEN_ABC".into()),
            expected_token: Some("TOKEN_ABC".into()),
        };
        let request2 = request1.clone();

        let res1 = verify_mock_erasure(&request1).expect("Success");
        let res2 = verify_mock_erasure(&request2).expect("Success");

        assert_eq!(res1, res2);
    }

    #[test]
    fn test_10_verify_from_mock_execution_result_integration() {
        let exec_result = MockExecutionResult {
            target_device_id: "MockDrive_ExecInteg".into(),
            method: SanitizationMethod::NvmeSanitizeCrypto,
            outcome: MockExecutionOutcome::Completed,
            was_successful: true,
            is_simulated_mock: true,
            note: "Mock run".into(),
            simulated_verification_token: Some("GENERATED_TOKEN_XYZ".into()),
        };

        // Matching expected token -> Verified
        let ver_res_ok = verify_mock_execution_result(&exec_result, Some("GENERATED_TOKEN_XYZ"))
            .expect("Verification must succeed");
        assert_eq!(ver_res_ok.verdict, VerificationVerdict::Verified);
        assert_eq!(
            ver_res_ok.observed_token.as_deref(),
            Some("GENERATED_TOKEN_XYZ")
        );

        // Mismatched expected token -> VerificationFailed
        let ver_res_fail = verify_mock_execution_result(&exec_result, Some("DIFFERENT_TOKEN"))
            .expect("Verification must succeed");
        assert_eq!(
            ver_res_fail.verdict,
            VerificationVerdict::VerificationFailed
        );

        // Omitted expected token -> NotVerifiable
        let ver_res_omit =
            verify_mock_execution_result(&exec_result, None).expect("Verification must succeed");
        assert_eq!(ver_res_omit.verdict, VerificationVerdict::NotVerifiable);
    }

    #[test]
    fn test_11_no_physical_storage_access_or_claims() {
        let request = MockVerificationRequest {
            target_device_id: "NonexistentDriveVirtual99".into(),
            method: SanitizationMethod::AtaSanitizeCrypto,
            observed_token: Some("VAL".into()),
            expected_token: Some("VAL".into()),
        };

        let res = verify_mock_erasure(&request).expect("Must succeed in-memory");
        assert_eq!(res.verdict, VerificationVerdict::Verified);
        // Ensure no absolute claims of physical media erasure
        assert!(!res.details.contains("permanently erased"));
        assert!(!res.details.contains("physically unrecoverable"));
        assert!(!res.details.contains("certified sanitized"));
    }
}