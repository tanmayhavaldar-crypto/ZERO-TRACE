//! Step 6.1 — Mock Drive Erasure Engine Implementation.
//!
//! Provides a safe, deterministic, non-destructive simulation of drive erasure operations.

use super::{
    DriveErasureEngine, MockCancellationToken, MockExecutionOutcome, MockExecutionRequest,
    MockExecutionResult, MockProgressEvent,
};
use crate::error::{DriveEraserError, Result};
use crate::safety::SafetyVerdict;

/// Mock engine for non-destructive simulation of drive erasure operations.
#[derive(Debug, Clone, Default)]
pub struct MockDriveErasureEngine;

impl MockDriveErasureEngine {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    fn generate_deterministic_token(request: &MockExecutionRequest) -> String {
        let seed = request.test_seed.as_deref().unwrap_or("default");
        let method_name = request.selected_method.label();
        let drive = &request.target_device_id;
        format!(
            "MOCK-TOKEN-{}-{:x}",
            drive,
            simple_hash(&format!("{}-{}-{}", drive, method_name, seed))
        )
    }
}

/// Simple non-cryptographic deterministic hash for token generation in unit tests.
const fn simple_hash(input: &str) -> u64 {
    let bytes = input.as_bytes();
    let mut hash: u64 = 0xcbf29ce484222325;
    let mut i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(0x100000001b3);
        i += 1;
    }
    hash
}

impl DriveErasureEngine for MockDriveErasureEngine {
    fn execute(
        &self,
        request: &MockExecutionRequest,
        cancellation: &MockCancellationToken,
        progress_callback: &mut dyn FnMut(MockProgressEvent),
    ) -> Result<MockExecutionResult> {
        // Enforce safety precondition: A blocked drive cannot be simulated as executed.
        if request.safety_assessment.verdict == SafetyVerdict::Blocked
            || !request.safety_assessment.is_allowed()
        {
            let reasons: Vec<String> = request
                .safety_assessment
                .blocking_reasons
                .iter()
                .map(|r| r.description.clone())
                .collect();
            return Err(DriveEraserError::SafetyViolation {
                device_id: request.target_device_id.clone(),
                reason: format!(
                    "Safety validation refused execution. Reasons: {}",
                    reasons.join("; ")
                ),
            });
        }

        // Validate target identifier
        if request.target_device_id.trim().is_empty() {
            return Err(DriveEraserError::ExecutionFailed {
                device_id: request.target_device_id.clone(),
                message: "Target device identifier cannot be empty".into(),
            });
        }

        let stages: [(u8, &str); 5] = [
            (0, "Initializing mock sanitization sequence"),
            (25, "Simulating controller command dispatch"),
            (50, "Simulating media sanitization in progress"),
            (75, "Simulating post-operation controller verification"),
            (100, "Simulated sanitization complete"),
        ];

        for (pct, stage) in stages {
            if cancellation.is_cancelled() {
                return Ok(MockExecutionResult {
                    target_device_id: request.target_device_id.clone(),
                    method: request.selected_method,
                    outcome: MockExecutionOutcome::Cancelled,
                    was_successful: false,
                    is_simulated_mock: true,
                    note: "SIMULATED MOCK RUN: Operation was cancelled before completion. No storage modified."
                        .into(),
                    simulated_verification_token: None,
                });
            }

            progress_callback(MockProgressEvent {
                target_device_id: request.target_device_id.clone(),
                percentage_complete: pct,
                stage_description: stage.to_string(),
            });
        }

        let token = Self::generate_deterministic_token(request);

        Ok(MockExecutionResult {
            target_device_id: request.target_device_id.clone(),
            method: request.selected_method,
            outcome: MockExecutionOutcome::Completed,
            was_successful: true,
            is_simulated_mock: true,
            note: "SIMULATED MOCK RUN: Simulated sanitization finished successfully. No physical disk commands or writes were performed."
                .into(),
            simulated_verification_token: Some(token),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{MockCancellationToken, MockExecutionOutcome, MockExecutionRequest};
    use crate::method_select::SanitizationMethod;
    use crate::safety::{BlockReasonCode, SafetyAssessment, SafetyFinding, SafetyVerdict};

    fn make_allowed_safety() -> SafetyAssessment {
        SafetyAssessment {
            verdict: SafetyVerdict::Allowed,
            blocking_reasons: Vec::new(),
        }
    }

    fn make_blocked_safety() -> SafetyAssessment {
        SafetyAssessment {
            verdict: SafetyVerdict::Blocked,
            blocking_reasons: vec![SafetyFinding {
                code: BlockReasonCode::SystemDrive,
                description: "Drive hosts the active Windows system volume".into(),
            }],
        }
    }

    #[test]
    fn test_1_execute_supported_method_success() {
        let engine = MockDriveErasureEngine::new();
        let request = MockExecutionRequest {
            target_device_id: "MockDrive_Disk1".into(),
            physical_drive_index: Some(1),
            selected_method: SanitizationMethod::NvmeSanitizeCrypto,
            safety_assessment: make_allowed_safety(),
            test_seed: Some("seed123".into()),
        };
        let cancel = MockCancellationToken::new();
        let mut events = Vec::new();

        let result = engine
            .execute(&request, &cancel, &mut |e| events.push(e))
            .expect("Execution must succeed");

        assert_eq!(result.outcome, MockExecutionOutcome::Completed);
        assert!(result.was_successful);
        assert!(result.is_simulated_mock);
        assert_eq!(result.target_device_id, "MockDrive_Disk1");
        assert_eq!(result.method, SanitizationMethod::NvmeSanitizeCrypto);
        assert!(result.simulated_verification_token.is_some());
    }

    #[test]
    fn test_2_result_clearly_identifies_operation_as_mock_simulated() {
        let engine = MockDriveErasureEngine::new();
        let request = MockExecutionRequest {
            target_device_id: "MockDrive_Disk2".into(),
            physical_drive_index: Some(2),
            selected_method: SanitizationMethod::AtaSanitizeBlock,
            safety_assessment: make_allowed_safety(),
            test_seed: None,
        };
        let cancel = MockCancellationToken::new();

        let result = engine
            .execute(&request, &cancel, &mut |_| {})
            .expect("Execution must succeed");

        assert!(result.is_simulated_mock);
        assert!(result.note.contains("SIMULATED MOCK RUN"));
        assert!(result.note.contains("No physical disk commands or writes were performed"));
    }

    #[test]
    fn test_3_progress_reaches_100_percent_on_success() {
        let engine = MockDriveErasureEngine::new();
        let request = MockExecutionRequest {
            target_device_id: "MockDrive_Disk3".into(),
            physical_drive_index: Some(3),
            selected_method: SanitizationMethod::AtaSecureErase,
            safety_assessment: make_allowed_safety(),
            test_seed: None,
        };
        let cancel = MockCancellationToken::new();
        let mut percentages = Vec::new();

        let result = engine
            .execute(&request, &cancel, &mut |e| percentages.push(e.percentage_complete))
            .expect("Execution must succeed");

        assert_eq!(result.outcome, MockExecutionOutcome::Completed);
        assert_eq!(percentages, vec![0, 25, 50, 75, 100]);
    }

    #[test]
    fn test_4_multiple_sanitization_methods_simulated() {
        let engine = MockDriveErasureEngine::new();
        let methods = [
            SanitizationMethod::NvmeSanitizeCrypto,
            SanitizationMethod::NvmeSanitizeBlock,
            SanitizationMethod::AtaSanitizeCrypto,
            SanitizationMethod::AtaSanitizeBlock,
            SanitizationMethod::AtaSanitizeOverwrite,
            SanitizationMethod::AtaEnhancedSecureErase,
            SanitizationMethod::AtaSecureErase,
            SanitizationMethod::ScsiSanitize,
        ];

        for method in methods {
            let request = MockExecutionRequest {
                target_device_id: "MockDrive_Multi".into(),
                physical_drive_index: Some(4),
                selected_method: method,
                safety_assessment: make_allowed_safety(),
                test_seed: Some("test_seed".into()),
            };
            let cancel = MockCancellationToken::new();
            let result = engine
                .execute(&request, &cancel, &mut |_| {})
                .expect("Execution of all methods must succeed");

            assert_eq!(result.method, method);
            assert_eq!(result.outcome, MockExecutionOutcome::Completed);
        }
    }

    #[test]
    fn test_5_cancellation_produces_cancelled_result() {
        let engine = MockDriveErasureEngine::new();
        let request = MockExecutionRequest {
            target_device_id: "MockDrive_Cancel".into(),
            physical_drive_index: Some(5),
            selected_method: SanitizationMethod::NvmeSanitizeBlock,
            safety_assessment: make_allowed_safety(),
            test_seed: None,
        };
        let cancel = MockCancellationToken::new();
        cancel.cancel();

        let mut events = Vec::new();
        let result = engine
            .execute(&request, &cancel, &mut |e| events.push(e))
            .expect("Cancelled run returns valid cancelled outcome");

        assert_eq!(result.outcome, MockExecutionOutcome::Cancelled);
        assert!(!result.was_successful);
        assert!(result.is_simulated_mock);
        assert_eq!(result.simulated_verification_token, None);
        assert!(events.is_empty());
    }

    #[test]
    fn test_6_cancelled_operation_not_reported_as_successful() {
        let engine = MockDriveErasureEngine::new();
        let request = MockExecutionRequest {
            target_device_id: "MockDrive_CancelFlag".into(),
            physical_drive_index: Some(6),
            selected_method: SanitizationMethod::AtaSecureErase,
            safety_assessment: make_allowed_safety(),
            test_seed: None,
        };
        let cancel = MockCancellationToken::new();
        cancel.cancel();

        let result = engine
            .execute(&request, &cancel, &mut |_| {})
            .expect("Returns result");

        assert!(!result.was_successful);
        assert_ne!(result.outcome, MockExecutionOutcome::Completed);
    }

    #[test]
    fn test_7_safety_blocked_request_cannot_report_success() {
        let engine = MockDriveErasureEngine::new();
        let request = MockExecutionRequest {
            target_device_id: "MockBlockedDrive".into(),
            physical_drive_index: Some(0),
            selected_method: SanitizationMethod::NvmeSanitizeCrypto,
            safety_assessment: make_blocked_safety(),
            test_seed: None,
        };
        let cancel = MockCancellationToken::new();

        let err = engine
            .execute(&request, &cancel, &mut |_| {})
            .expect_err("Safety-blocked request must be refused");

        match err {
            DriveEraserError::SafetyViolation { device_id, reason } => {
                assert_eq!(device_id, "MockBlockedDrive");
                assert!(reason.contains("Safety validation refused execution"));
            }
            other => panic!("Expected SafetyViolation error, got: {:?}", other),
        }
    }

    #[test]
    fn test_8_deterministic_result_for_identical_input() {
        let engine = MockDriveErasureEngine::new();
        let request1 = MockExecutionRequest {
            target_device_id: "MockDrive_Det".into(),
            physical_drive_index: Some(7),
            selected_method: SanitizationMethod::NvmeSanitizeCrypto,
            safety_assessment: make_allowed_safety(),
            test_seed: Some("static_seed".into()),
        };
        let request2 = request1.clone();
        let cancel = MockCancellationToken::new();

        let res1 = engine
            .execute(&request1, &cancel, &mut |_| {})
            .expect("Execution 1 succeeds");
        let res2 = engine
            .execute(&request2, &cancel, &mut |_| {})
            .expect("Execution 2 succeeds");

        assert_eq!(res1, res2);
        assert_eq!(
            res1.simulated_verification_token,
            res2.simulated_verification_token
        );
    }

    #[test]
    fn test_9_no_physical_storage_access_performed() {
        let engine = MockDriveErasureEngine::new();
        let request = MockExecutionRequest {
            target_device_id: "MockNonexistentDrive999".into(),
            physical_drive_index: Some(999),
            selected_method: SanitizationMethod::ScsiSanitize,
            safety_assessment: make_allowed_safety(),
            test_seed: None,
        };
        let cancel = MockCancellationToken::new();

        let result = engine
            .execute(&request, &cancel, &mut |_| {})
            .expect("Mock operates strictly in memory without OS storage handles");

        assert!(result.is_simulated_mock);
        assert_eq!(result.outcome, MockExecutionOutcome::Completed);
    }

    #[test]
    fn test_10_invalid_input_handled_safely() {
        let engine = MockDriveErasureEngine::new();
        let request = MockExecutionRequest {
            target_device_id: "   ".into(),
            physical_drive_index: None,
            selected_method: SanitizationMethod::NvmeSanitizeCrypto,
            safety_assessment: make_allowed_safety(),
            test_seed: None,
        };
        let cancel = MockCancellationToken::new();

        let err = engine
            .execute(&request, &cancel, &mut |_| {})
            .expect_err("Empty target identifier must be rejected");

        match err {
            DriveEraserError::ExecutionFailed { message, .. } => {
                assert!(message.contains("cannot be empty"));
            }
            other => panic!("Expected ExecutionFailed, got: {:?}", other),
        }
    }
}