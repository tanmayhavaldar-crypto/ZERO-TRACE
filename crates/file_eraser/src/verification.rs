//! Step 6.2 — Mock File Verification Layer.
//!
//! # Strict Simulation Invariants
//! 1. Non-Destructive: Pure in-memory verification logic. Performs NO disk reads,
//!    NO hashing of file contents, NO metadata queries, NO stream inspection,
//!    and NO slack/free-space probing.
//! 2. Mandatory Step 6.1 Execution Result: A valid `&MockFileExecutionResult` must
//!    always be supplied. Verification cannot be bypassed or fabricated without
//!    evaluating an actual Step 6.1 execution outcome.
//! 3. Epistemic Discipline: Distinguishes `Verified`, `VerificationFailed`, and
//!    `NotVerifiable`. Missing or unobserved token evidence evaluates to
//!    `NotVerifiable`, never coerced into success or failure.
//! 4. Fail-Closed on Execution Anomalies: If the Step 6.1 outcome is cancelled,
//!    safety-refused, marked unsuccessful, not tagged as mock, or has mismatched
//!    methods or paths, verification returns `VerificationFailed`.
//! 5. Deterministic & Pure: Evaluates supplied data structures with no side effects.

use crate::engine::{MockFileExecutionOutcome, MockFileExecutionResult};
use crate::errors::{EraserError, EraserResult};
use crate::method_select::FileSanitizationMethod;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// High-level verdict of the mock file verification assessment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VerificationVerdict {
    /// The mock verification criteria were fully evaluated and passed.
    Verified,
    /// The mock verification criteria were evaluated and definitively failed.
    VerificationFailed,
    /// The available mock verification evidence is missing or insufficient.
    NotVerifiable,
}

impl VerificationVerdict {
    #[must_use]
    pub const fn is_verified(&self) -> bool {
        matches!(self, Self::Verified)
    }

    #[must_use]
    pub const fn is_failed(&self) -> bool {
        matches!(self, Self::VerificationFailed)
    }

    #[must_use]
    pub const fn is_not_verifiable(&self) -> bool {
        matches!(self, Self::NotVerifiable)
    }
}

/// Request to evaluate the mock verification of an actual Step 6.1 execution outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MockFileVerificationRequest {
    /// Target file path expected to have been simulated.
    pub target_path: PathBuf,
    /// Expected list of methods that should have been simulated, in order.
    pub expected_methods: Vec<FileSanitizationMethod>,
    /// Expected simulated verification token used for deterministic validation matching.
    pub expected_token: Option<String>,
}

/// Comprehensive outcome of the mock verification assessment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MockFileVerificationResult {
    /// Target file path of the verified simulation.
    pub target_path: PathBuf,
    /// High-level verdict.
    pub verdict: VerificationVerdict,
    /// Detailed diagnostic / audit note explaining the outcome.
    pub details: String,
    /// Expected verification token, if supplied.
    pub expected_token: Option<String>,
    /// Observed verification token from the execution result, if present.
    pub observed_token: Option<String>,
    /// Number of verified methods that matched the contract.
    pub verified_method_count: usize,
    /// Explicit flag marking that this is a simulated verification and NOT a physical drive audit.
    pub is_simulated_mock: bool,
}

/// Evaluates a `MockFileVerificationRequest` against an actual `MockFileExecutionResult` in memory.
///
/// # Strict Contract
/// 1. Target path in request must not be empty.
/// 2. `execution_result` must be an actual reference (cannot be omitted).
/// 3. Target path must match `execution_result.target_path`.
/// 4. Execution outcome must be `Completed`.
/// 5. `execution_result.was_successful` must be true.
/// 6. `execution_result.is_simulated_mock` must be true.
/// 7. `execution_result.simulated_methods` must match `request.expected_methods` in content and order.
/// 8. If execution result has no `simulated_verification_token` -> `VerificationVerdict::NotVerifiable`.
/// 9. If `request.expected_token` is missing -> `VerificationVerdict::NotVerifiable`.
/// 10. If both tokens exist and match -> `VerificationVerdict::Verified`.
/// 11. If both tokens exist but differ -> `VerificationVerdict::VerificationFailed`.
pub fn verify_mock_file_erasure(
    request: &MockFileVerificationRequest,
    execution_result: &MockFileExecutionResult,
) -> EraserResult<MockFileVerificationResult> {
    if request.target_path.as_os_str().is_empty() {
        return Err(EraserError::IoError {
            path: request.target_path.clone(),
            message: "Target file path cannot be empty for verification assessment".into(),
        });
    }

    // 1. Target path consistency check
    if request.target_path != execution_result.target_path {
        return Ok(MockFileVerificationResult {
            target_path: request.target_path.clone(),
            verdict: VerificationVerdict::VerificationFailed,
            details: format!(
                "SIMULATED MOCK VERIFICATION FAILED: Target path mismatch. Request expected {:?}, but execution result targeted {:?}.",
                request.target_path, execution_result.target_path
            ),
            expected_token: request.expected_token.clone(),
            observed_token: execution_result.simulated_verification_token.clone(),
            verified_method_count: 0,
            is_simulated_mock: true,
        });
    }

    // 2. Mock authenticity check
    if !execution_result.is_simulated_mock {
        return Ok(MockFileVerificationResult {
            target_path: request.target_path.clone(),
            verdict: VerificationVerdict::VerificationFailed,
            details: "SIMULATED MOCK VERIFICATION FAILED: Execution result is not marked as a mock simulation."
                .into(),
            expected_token: request.expected_token.clone(),
            observed_token: execution_result.simulated_verification_token.clone(),
            verified_method_count: 0,
            is_simulated_mock: true,
        });
    }

    // 3. Execution success flag check
    if !execution_result.was_successful {
        return Ok(MockFileVerificationResult {
            target_path: request.target_path.clone(),
            verdict: VerificationVerdict::VerificationFailed,
            details: format!(
                "SIMULATED MOCK VERIFICATION FAILED: Execution was not successful (outcome: {:?}).",
                execution_result.outcome
            ),
            expected_token: request.expected_token.clone(),
            observed_token: execution_result.simulated_verification_token.clone(),
            verified_method_count: 0,
            is_simulated_mock: true,
        });
    }

    // 4. Outcome validation check
    match execution_result.outcome {
        MockFileExecutionOutcome::Completed => {}
        MockFileExecutionOutcome::Cancelled => {
            return Ok(MockFileVerificationResult {
                target_path: request.target_path.clone(),
                verdict: VerificationVerdict::VerificationFailed,
                details: "SIMULATED MOCK VERIFICATION FAILED: Execution was cancelled before completion."
                    .into(),
                expected_token: request.expected_token.clone(),
                observed_token: execution_result.simulated_verification_token.clone(),
                verified_method_count: 0,
                is_simulated_mock: true,
            });
        }
        MockFileExecutionOutcome::RefusedSafetyBlocked => {
            return Ok(MockFileVerificationResult {
                target_path: request.target_path.clone(),
                verdict: VerificationVerdict::VerificationFailed,
                details: "SIMULATED MOCK VERIFICATION FAILED: Execution was refused due to safety validation block."
                    .into(),
                expected_token: request.expected_token.clone(),
                observed_token: execution_result.simulated_verification_token.clone(),
                verified_method_count: 0,
                is_simulated_mock: true,
            });
        }
        MockFileExecutionOutcome::NoExecutableMethods => {
            return Ok(MockFileVerificationResult {
                target_path: request.target_path.clone(),
                verdict: VerificationVerdict::VerificationFailed,
                details: "SIMULATED MOCK VERIFICATION FAILED: Execution plan contained zero executable methods."
                    .into(),
                expected_token: request.expected_token.clone(),
                observed_token: execution_result.simulated_verification_token.clone(),
                verified_method_count: 0,
                is_simulated_mock: true,
            });
        }
    }

    // 5. Method list and order consistency check
    if request.expected_methods != execution_result.simulated_methods {
        return Ok(MockFileVerificationResult {
            target_path: request.target_path.clone(),
            verdict: VerificationVerdict::VerificationFailed,
            details: format!(
                "SIMULATED MOCK VERIFICATION FAILED: Sanitization method list or ordering mismatch. Expected {:?}, observed {:?}.",
                request.expected_methods, execution_result.simulated_methods
            ),
            expected_token: request.expected_token.clone(),
            observed_token: execution_result.simulated_verification_token.clone(),
            verified_method_count: 0,
            is_simulated_mock: true,
        });
    }

    // 6. Token Evaluation
    let observed_token = &execution_result.simulated_verification_token;

    match (observed_token, &request.expected_token) {
        (None, None) => Ok(MockFileVerificationResult {
            target_path: request.target_path.clone(),
            verdict: VerificationVerdict::NotVerifiable,
            details: "SIMULATED MOCK VERIFICATION: Neither observed nor expected verification token was provided. No physical storage verification performed."
                .into(),
            expected_token: None,
            observed_token: None,
            verified_method_count: 0,
            is_simulated_mock: true,
        }),
        (None, Some(expected)) => Ok(MockFileVerificationResult {
            target_path: request.target_path.clone(),
            verdict: VerificationVerdict::NotVerifiable,
            details: "SIMULATED MOCK VERIFICATION: Observed verification token is missing from execution outcome. No physical storage verification performed."
                .into(),
            expected_token: Some(expected.clone()),
            observed_token: None,
            verified_method_count: 0,
            is_simulated_mock: true,
        }),
        (Some(observed), None) => Ok(MockFileVerificationResult {
            target_path: request.target_path.clone(),
            verdict: VerificationVerdict::NotVerifiable,
            details: "SIMULATED MOCK VERIFICATION: Expected verification token was omitted; cannot perform comparison. No physical storage verification performed."
                .into(),
            expected_token: None,
            observed_token: Some(observed.clone()),
            verified_method_count: 0,
            is_simulated_mock: true,
        }),
        (Some(observed), Some(expected)) => {
            if observed == expected {
                Ok(MockFileVerificationResult {
                    target_path: request.target_path.clone(),
                    verdict: VerificationVerdict::Verified,
                    details: "SIMULATED MOCK VERIFICATION: Deterministic simulation artifact matched. No physical storage verification performed."
                        .into(),
                    expected_token: Some(expected.clone()),
                    observed_token: Some(observed.clone()),
                    verified_method_count: request.expected_methods.len(),
                    is_simulated_mock: true,
                })
            } else {
                Ok(MockFileVerificationResult {
                    target_path: request.target_path.clone(),
                    verdict: VerificationVerdict::VerificationFailed,
                    details: format!(
                        "SIMULATED MOCK VERIFICATION FAILED: Token mismatch. Expected '{expected}', observed '{observed}'. No physical storage verification performed."
                    ),
                    expected_token: Some(expected.clone()),
                    observed_token: Some(observed.clone()),
                    verified_method_count: 0,
                    is_simulated_mock: true,
                })
            }
        }
    }
}

/// Convenience helper to verify a Step 6.1 `MockFileExecutionResult` directly.
pub fn verify_mock_execution_result(
    execution_result: &MockFileExecutionResult,
    expected_token: Option<&str>,
    expected_methods: &[FileSanitizationMethod],
) -> EraserResult<MockFileVerificationResult> {
    let request = MockFileVerificationRequest {
        target_path: execution_result.target_path.clone(),
        expected_methods: expected_methods.to_vec(),
        expected_token: expected_token.map(ToOwned::to_owned),
    };
    verify_mock_file_erasure(&request, execution_result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{MockFileExecutionOutcome, MockFileExecutionResult};
    use crate::method_select::FileSanitizationMethod;

    fn make_valid_execution(
        path: &str,
        methods: Vec<FileSanitizationMethod>,
        token: Option<&str>,
    ) -> MockFileExecutionResult {
        MockFileExecutionResult {
            target_path: PathBuf::from(path),
            simulated_methods: methods,
            outcome: MockFileExecutionOutcome::Completed,
            was_successful: true,
            is_simulated_mock: true,
            note: "Simulation completed".into(),
            simulated_verification_token: token.map(ToOwned::to_owned),
        }
    }

    #[test]
    fn test_1_matching_token_produces_verified() {
        let exec = make_valid_execution(
            r"D:\Data\doc.txt",
            vec![FileSanitizationMethod::MetadataSanitization],
            Some("MOCK-TOKEN-XYZ"),
        );
        let request = MockFileVerificationRequest {
            target_path: PathBuf::from(r"D:\Data\doc.txt"),
            expected_methods: vec![FileSanitizationMethod::MetadataSanitization],
            expected_token: Some("MOCK-TOKEN-XYZ".into()),
        };

        let result = verify_mock_file_erasure(&request, &exec).expect("Must succeed");
        assert_eq!(result.verdict, VerificationVerdict::Verified);
        assert!(result.verdict.is_verified());
        assert!(result.is_simulated_mock);
        assert!(result.details.contains("Deterministic simulation artifact matched"));
        assert_eq!(result.verified_method_count, 1);
    }

    #[test]
    fn test_2_mismatching_token_produces_verification_failed() {
        let exec = make_valid_execution(
            r"D:\Data\doc.txt",
            vec![FileSanitizationMethod::MetadataSanitization],
            Some("MOCK-TOKEN-OBSERVED"),
        );
        let request = MockFileVerificationRequest {
            target_path: PathBuf::from(r"D:\Data\doc.txt"),
            expected_methods: vec![FileSanitizationMethod::MetadataSanitization],
            expected_token: Some("MOCK-TOKEN-EXPECTED".into()),
        };

        let result = verify_mock_file_erasure(&request, &exec).expect("Must succeed");
        assert_eq!(result.verdict, VerificationVerdict::VerificationFailed);
        assert!(result.verdict.is_failed());
        assert!(result.details.contains("Token mismatch"));
    }

    #[test]
    fn test_3_missing_expected_token_produces_not_verifiable() {
        let exec = make_valid_execution(
            r"D:\Data\doc.txt",
            vec![FileSanitizationMethod::MetadataSanitization],
            Some("MOCK-TOKEN-OBSERVED"),
        );
        let request = MockFileVerificationRequest {
            target_path: PathBuf::from(r"D:\Data\doc.txt"),
            expected_methods: vec![FileSanitizationMethod::MetadataSanitization],
            expected_token: None,
        };

        let result = verify_mock_file_erasure(&request, &exec).expect("Must succeed");
        assert_eq!(result.verdict, VerificationVerdict::NotVerifiable);
        assert!(result.verdict.is_not_verifiable());
        assert!(result.details.contains("Expected verification token was omitted"));
    }

    #[test]
    fn test_4_missing_observed_token_produces_not_verifiable() {
        let exec = make_valid_execution(
            r"D:\Data\doc.txt",
            vec![FileSanitizationMethod::MetadataSanitization],
            None,
        );
        let request = MockFileVerificationRequest {
            target_path: PathBuf::from(r"D:\Data\doc.txt"),
            expected_methods: vec![FileSanitizationMethod::MetadataSanitization],
            expected_token: Some("MOCK-TOKEN-EXPECTED".into()),
        };

        let result = verify_mock_file_erasure(&request, &exec).expect("Must succeed");
        assert_eq!(result.verdict, VerificationVerdict::NotVerifiable);
        assert!(result.verdict.is_not_verifiable());
        assert!(result.details.contains("Observed verification token is missing"));
    }

    #[test]
    fn test_5_both_tokens_missing_produces_not_verifiable() {
        let exec = make_valid_execution(
            r"D:\Data\doc.txt",
            vec![FileSanitizationMethod::MetadataSanitization],
            None,
        );
        let request = MockFileVerificationRequest {
            target_path: PathBuf::from(r"D:\Data\doc.txt"),
            expected_methods: vec![FileSanitizationMethod::MetadataSanitization],
            expected_token: None,
        };

        let result = verify_mock_file_erasure(&request, &exec).expect("Must succeed");
        assert_eq!(result.verdict, VerificationVerdict::NotVerifiable);
        assert!(result.verdict.is_not_verifiable());
        assert!(result.details.contains("Neither observed nor expected verification token was provided"));
    }

    #[test]
    fn test_6_cancelled_execution_cannot_verify_successfully() {
        let mut exec = make_valid_execution(
            r"D:\Data\doc.txt",
            vec![FileSanitizationMethod::MetadataSanitization],
            Some("MOCK-TOKEN-123"),
        );
        exec.outcome = MockFileExecutionOutcome::Cancelled;
        exec.was_successful = false;

        let result = verify_mock_execution_result(
            &exec,
            Some("MOCK-TOKEN-123"),
            &[FileSanitizationMethod::MetadataSanitization],
        )
        .expect("Verification evaluates safely");

        assert_eq!(result.verdict, VerificationVerdict::VerificationFailed);
        assert!(result.details.contains("Execution was not successful"));
    }

    #[test]
    fn test_7_safety_refused_execution_cannot_verify_successfully() {
        let mut exec = make_valid_execution(
            r"D:\Data\doc.txt",
            vec![FileSanitizationMethod::MetadataSanitization],
            None,
        );
        exec.outcome = MockFileExecutionOutcome::RefusedSafetyBlocked;
        exec.was_successful = false;

        let result = verify_mock_execution_result(
            &exec,
            Some("MOCK-TOKEN-123"),
            &[FileSanitizationMethod::MetadataSanitization],
        )
        .expect("Verification evaluates safely");

        assert_eq!(result.verdict, VerificationVerdict::VerificationFailed);
        assert!(result.details.contains("Execution was not successful"));
    }

    #[test]
    fn test_8_zero_method_execution_cannot_verify_successfully() {
        let mut exec = make_valid_execution(r"D:\Data\doc.txt", vec![], None);
        exec.outcome = MockFileExecutionOutcome::NoExecutableMethods;
        exec.was_successful = false;

        let result = verify_mock_execution_result(&exec, Some("MOCK-TOKEN-123"), &[])
            .expect("Verification evaluates safely");

        assert_eq!(result.verdict, VerificationVerdict::VerificationFailed);
        assert!(result.details.contains("Execution was not successful"));
    }

    #[test]
    fn test_9_unsuccessful_execution_cannot_verify_successfully() {
        let mut exec = make_valid_execution(
            r"D:\Data\doc.txt",
            vec![FileSanitizationMethod::MetadataSanitization],
            Some("MOCK-TOKEN-123"),
        );
        exec.was_successful = false;

        let result = verify_mock_execution_result(
            &exec,
            Some("MOCK-TOKEN-123"),
            &[FileSanitizationMethod::MetadataSanitization],
        )
        .expect("Verification evaluates safely");

        assert_eq!(result.verdict, VerificationVerdict::VerificationFailed);
        assert!(result.details.contains("Execution was not successful"));
    }

    #[test]
    fn test_10_non_mock_result_cannot_verify_successfully() {
        let mut exec = make_valid_execution(
            r"D:\Data\doc.txt",
            vec![FileSanitizationMethod::MetadataSanitization],
            Some("MOCK-TOKEN-123"),
        );
        exec.is_simulated_mock = false;

        let result = verify_mock_execution_result(
            &exec,
            Some("MOCK-TOKEN-123"),
            &[FileSanitizationMethod::MetadataSanitization],
        )
        .expect("Verification evaluates safely");

        assert_eq!(result.verdict, VerificationVerdict::VerificationFailed);
        assert!(result.details.contains("not marked as a mock simulation"));
    }

    #[test]
    fn test_11_target_path_mismatch_produces_verification_failed() {
        let exec = make_valid_execution(
            r"D:\Data\doc_actual.txt",
            vec![FileSanitizationMethod::MetadataSanitization],
            Some("MOCK-TOKEN-123"),
        );

        let request = MockFileVerificationRequest {
            target_path: PathBuf::from(r"D:\Data\doc_expected.txt"),
            expected_methods: vec![FileSanitizationMethod::MetadataSanitization],
            expected_token: Some("MOCK-TOKEN-123".into()),
        };

        let result = verify_mock_file_erasure(&request, &exec).expect("Must succeed");
        assert_eq!(result.verdict, VerificationVerdict::VerificationFailed);
        assert!(result.details.contains("Target path mismatch"));
    }

    #[test]
    fn test_12_method_list_mismatch_produces_verification_failed() {
        let exec = make_valid_execution(
            r"D:\Data\doc.txt",
            vec![FileSanitizationMethod::MetadataSanitization],
            Some("MOCK-TOKEN-123"),
        );

        let result = verify_mock_execution_result(
            &exec,
            Some("MOCK-TOKEN-123"),
            &[
                FileSanitizationMethod::MetadataSanitization,
                FileSanitizationMethod::AlternateDataStreamSanitization,
            ],
        )
        .expect("Must succeed");

        assert_eq!(result.verdict, VerificationVerdict::VerificationFailed);
        assert!(result.details.contains("method list or ordering mismatch"));
    }

    #[test]
    fn test_13_method_ordering_mismatch_produces_verification_failed() {
        let exec = make_valid_execution(
            r"D:\Data\doc.txt",
            vec![
                FileSanitizationMethod::ContentOverwrite,
                FileSanitizationMethod::MetadataSanitization,
            ],
            Some("MOCK-TOKEN-123"),
        );

        let result = verify_mock_execution_result(
            &exec,
            Some("MOCK-TOKEN-123"),
            &[
                FileSanitizationMethod::MetadataSanitization,
                FileSanitizationMethod::ContentOverwrite,
            ],
        )
        .expect("Must succeed");

        assert_eq!(result.verdict, VerificationVerdict::VerificationFailed);
        assert!(result.details.contains("method list or ordering mismatch"));
    }

    #[test]
    fn test_14_correct_method_list_and_order_produces_verified() {
        let methods = vec![
            FileSanitizationMethod::ContentOverwrite,
            FileSanitizationMethod::MetadataSanitization,
            FileSanitizationMethod::AlternateDataStreamSanitization,
        ];
        let exec = make_valid_execution(r"D:\Data\doc.txt", methods.clone(), Some("MOCK-TOKEN-123"));

        let result = verify_mock_execution_result(&exec, Some("MOCK-TOKEN-123"), &methods)
            .expect("Must succeed");

        assert_eq!(result.verdict, VerificationVerdict::Verified);
        assert_eq!(result.verified_method_count, 3);
    }

    #[test]
    fn test_15_deterministic_identical_input_produces_identical_result() {
        let exec = make_valid_execution(
            r"D:\Data\doc.txt",
            vec![FileSanitizationMethod::MetadataSanitization],
            Some("TOKEN-1"),
        );
        let request1 = MockFileVerificationRequest {
            target_path: PathBuf::from(r"D:\Data\doc.txt"),
            expected_methods: vec![FileSanitizationMethod::MetadataSanitization],
            expected_token: Some("TOKEN-1".into()),
        };
        let request2 = request1.clone();

        let res1 = verify_mock_file_erasure(&request1, &exec).expect("Must succeed");
        let res2 = verify_mock_file_erasure(&request2, &exec).expect("Must succeed");

        assert_eq!(res1, res2);
    }

    #[test]
    fn test_16_result_clearly_identifies_itself_as_simulated_mock() {
        let exec = make_valid_execution(
            r"D:\Data\doc.txt",
            vec![FileSanitizationMethod::MetadataSanitization],
            Some("TOKEN-1"),
        );
        let request = MockFileVerificationRequest {
            target_path: PathBuf::from(r"D:\Data\doc.txt"),
            expected_methods: vec![FileSanitizationMethod::MetadataSanitization],
            expected_token: Some("TOKEN-1".into()),
        };

        let res = verify_mock_file_erasure(&request, &exec).expect("Must succeed");
        assert!(res.is_simulated_mock);
        assert!(res.details.contains("SIMULATED MOCK VERIFICATION"));
        assert!(res.details.contains("No physical storage verification performed"));
    }

    #[test]
    fn test_17_no_physical_filesystem_access() {
        let non_existent = PathBuf::from(r"Z:\Nonexistent\Virtual\Missing.bin");
        let exec = MockFileExecutionResult {
            target_path: non_existent.clone(),
            simulated_methods: vec![FileSanitizationMethod::MetadataSanitization],
            outcome: MockFileExecutionOutcome::Completed,
            was_successful: true,
            is_simulated_mock: true,
            note: "Simulation completed".into(),
            simulated_verification_token: Some("TOKEN-1".into()),
        };

        let request = MockFileVerificationRequest {
            target_path: non_existent,
            expected_methods: vec![FileSanitizationMethod::MetadataSanitization],
            expected_token: Some("TOKEN-1".into()),
        };

        let res = verify_mock_file_erasure(&request, &exec).expect("Must succeed in memory");
        assert_eq!(res.verdict, VerificationVerdict::Verified);
    }

    #[test]
    fn test_18_no_physical_storage_access_disclaimer() {
        let exec = make_valid_execution(
            r"D:\Data\doc.txt",
            vec![FileSanitizationMethod::MetadataSanitization],
            Some("TOKEN-1"),
        );
        let request = MockFileVerificationRequest {
            target_path: PathBuf::from(r"D:\Data\doc.txt"),
            expected_methods: vec![FileSanitizationMethod::MetadataSanitization],
            expected_token: Some("TOKEN-1".into()),
        };

        let res = verify_mock_file_erasure(&request, &exec).expect("Must succeed");
        assert!(!res.details.contains("physically erased"));
        assert!(!res.details.contains("NIST certified"));
        assert!(!res.details.contains("guaranteed unrecoverable"));
    }

    #[test]
    fn test_19_empty_target_handled_safely() {
        let exec = make_valid_execution(
            r"D:\Data\doc.txt",
            vec![FileSanitizationMethod::MetadataSanitization],
            Some("TOKEN-1"),
        );
        let request = MockFileVerificationRequest {
            target_path: PathBuf::from(""),
            expected_methods: vec![FileSanitizationMethod::MetadataSanitization],
            expected_token: Some("TOKEN-1".into()),
        };

        let err = verify_mock_file_erasure(&request, &exec).expect_err("Empty target path must error");
        match err {
            EraserError::IoError { message, .. } => {
                assert!(message.contains("cannot be empty"));
            }
            other => panic!("Expected IoError, got: {:?}", other),
        }
    }

    #[test]
    fn test_20_successful_verification_reports_correct_verified_method_count() {
        let methods = vec![
            FileSanitizationMethod::ContentOverwrite,
            FileSanitizationMethod::MetadataSanitization,
            FileSanitizationMethod::AlternateDataStreamSanitization,
            FileSanitizationMethod::SlackSpaceSanitization,
            FileSanitizationMethod::FreeSpaceSanitization,
        ];
        let exec = make_valid_execution(r"D:\Data\doc.txt", methods.clone(), Some("TOKEN-ALL"));

        let res = verify_mock_execution_result(&exec, Some("TOKEN-ALL"), &methods).expect("Must succeed");
        assert_eq!(res.verdict, VerificationVerdict::Verified);
        assert_eq!(res.verified_method_count, 5);
    }
}