//! Step 6.1 — Mock File Erasure Engine Implementation.
//!
//! Provides a safe, deterministic, non-destructive simulation of file sanitization plans.

use super::{
    FileErasureEngine, MockFileCancellationToken, MockFileExecutionOutcome,
    MockFileExecutionRequest, MockFileExecutionResult, MockFileProgressEvent,
};
use crate::errors::{EraserError, EraserResult};
use crate::method_select::FileSanitizationMethod;
use crate::safety_validate::SafetyValidationStatus;

/// Mock engine for non-destructive simulation of file and metadata sanitization.
#[derive(Debug, Clone, Default)]
pub struct MockFileErasureEngine;

impl MockFileErasureEngine {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Generates a deterministic mock verification token based on the request parameters.
    fn generate_deterministic_token(request: &MockFileExecutionRequest) -> String {
        let seed = request.test_seed.as_deref().unwrap_or("default");
        let path_str = request.target_path.to_string_lossy();
        let method_tags: Vec<&str> = request
            .plan
            .proposed_methods
            .iter()
            .map(|m| m.method.label())
            .collect();
        let combined = format!("{}:{}:{}", path_str, method_tags.join(","), seed);

        format!("MOCK-FILE-TOKEN-{:x}", simple_hash(&combined))
    }
}

/// Simple 64-bit FNV-1a hash implementation for deterministic in-memory token generation.
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

impl FileErasureEngine for MockFileErasureEngine {
    fn execute(
        &self,
        request: &MockFileExecutionRequest,
        cancellation: &MockFileCancellationToken,
        progress_callback: &mut dyn FnMut(MockFileProgressEvent),
    ) -> EraserResult<MockFileExecutionResult> {
        // Validate target path input
        if request.target_path.as_os_str().is_empty() {
            return Err(EraserError::IoError {
                path: request.target_path.clone(),
                message: "Target file path cannot be empty for erasure simulation".into(),
            });
        }

        // 1. Safety Precondition Check: Blocked validation refuses execution immediately.
        if request.safety_validation.status == SafetyValidationStatus::Blocked {
            let reason_summary = request.safety_validation.reasons.join("; ");
            return Ok(MockFileExecutionResult {
                target_path: request.target_path.clone(),
                simulated_methods: Vec::new(),
                outcome: MockFileExecutionOutcome::RefusedSafetyBlocked,
                was_successful: false,
                is_simulated_mock: true,
                note: format!(
                    "SIMULATED MOCK RUN REFUSED: Step 5.3 safety validation was Blocked. Reasons: {reason_summary}. NO REAL FILE OPERATIONS PERFORMED."
                ),
                simulated_verification_token: None,
            });
        }

        // 2. Check for empty plan with zero executable methods.
        if request.plan.proposed_methods.is_empty() {
            return Ok(MockFileExecutionResult {
                target_path: request.target_path.clone(),
                simulated_methods: Vec::new(),
                outcome: MockFileExecutionOutcome::NoExecutableMethods,
                was_successful: false,
                is_simulated_mock: true,
                note: "SIMULATED MOCK RUN: Plan contains zero executable methods. No sanitization simulation was performed. NO REAL FILE OPERATIONS PERFORMED."
                    .into(),
                simulated_verification_token: None,
            });
        }

        let total_methods = request.plan.proposed_methods.len();
        let mut executed_methods = Vec::with_capacity(total_methods);

        // Initial 0% progress
        progress_callback(MockFileProgressEvent {
            target_path: request.target_path.clone(),
            current_method: None,
            percentage_complete: 0,
            status_message: "Initializing mock sanitization sequence".into(),
        });

        for (index, proposal) in request.plan.proposed_methods.iter().enumerate() {
            if cancellation.is_cancelled() {
                return Ok(MockFileExecutionResult {
                    target_path: request.target_path.clone(),
                    simulated_methods: executed_methods,
                    outcome: MockFileExecutionOutcome::Cancelled,
                    was_successful: false,
                    is_simulated_mock: true,
                    note: "SIMULATED MOCK RUN: Operation cancelled before completion. NO REAL FILE OPERATIONS PERFORMED."
                        .into(),
                    simulated_verification_token: None,
                });
            }

            let method = proposal.method;
            executed_methods.push(method);

            let status_msg = match method {
                FileSanitizationMethod::ContentOverwrite => {
                    "Simulating file content allocation overwrite pass"
                }
                FileSanitizationMethod::MetadataSanitization => {
                    "Simulating metadata timestamp and attribute sanitization"
                }
                FileSanitizationMethod::AlternateDataStreamSanitization => {
                    "Simulating alternate data stream clearing and truncation"
                }
                FileSanitizationMethod::SlackSpaceSanitization => {
                    "Simulating terminal cluster slack space byte clearing"
                }
                FileSanitizationMethod::FreeSpaceSanitization => {
                    "Simulating volume unallocated cluster pass"
                }
            };

            // Intermediate progress calculation
            let pct = (((index + 1) as f32 / total_methods as f32) * 100.0).min(99.0) as u8;

            progress_callback(MockFileProgressEvent {
                target_path: request.target_path.clone(),
                current_method: Some(method),
                percentage_complete: pct,
                status_message: status_msg.into(),
            });
        }

        // Final 100% completion check
        if cancellation.is_cancelled() {
            return Ok(MockFileExecutionResult {
                target_path: request.target_path.clone(),
                simulated_methods: executed_methods,
                outcome: MockFileExecutionOutcome::Cancelled,
                was_successful: false,
                is_simulated_mock: true,
                note: "SIMULATED MOCK RUN: Operation cancelled at finalization. NO REAL FILE OPERATIONS PERFORMED."
                    .into(),
                simulated_verification_token: None,
            });
        }

        progress_callback(MockFileProgressEvent {
            target_path: request.target_path.clone(),
            current_method: None,
            percentage_complete: 100,
            status_message: "Simulated file sanitization complete".into(),
        });

        let token = Self::generate_deterministic_token(request);

        Ok(MockFileExecutionResult {
            target_path: request.target_path.clone(),
            simulated_methods: executed_methods,
            outcome: MockFileExecutionOutcome::Completed,
            was_successful: true,
            is_simulated_mock: true,
            note: "SIMULATED MOCK RUN: Simulated file sanitization completed successfully. NO REAL FILE OPERATIONS PERFORMED."
                .into(),
            simulated_verification_token: Some(token),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::CapabilityState;
    use crate::engine::{
        MockFileCancellationToken, MockFileExecutionOutcome, MockFileExecutionRequest,
    };
    use crate::method_select::{
        FileSanitizationMethod, FileSanitizationPlan, MethodProposal,
    };
    use crate::model::SafetyClassification;
    use crate::safety_validate::{SafetyValidationResult, SafetyValidationStatus};
    use std::path::PathBuf;

    fn make_test_plan(methods: Vec<FileSanitizationMethod>) -> FileSanitizationPlan {
        let proposals = methods
            .into_iter()
            .map(|m| MethodProposal {
                method: m,
                label: m.label().to_string(),
                rationale: "Rationale for test".into(),
                capability_state: CapabilityState::Supported,
            })
            .collect();

        FileSanitizationPlan {
            target_path: PathBuf::from(r"D:\SafeZone\sample_file.txt"),
            safety: SafetyClassification::SafeToAnalyze,
            proposed_methods: proposals,
            summary: "Plan summary".into(),
        }
    }

    fn make_allowed_safety(method_count: usize) -> SafetyValidationResult {
        SafetyValidationResult {
            status: SafetyValidationStatus::Allowed,
            target_path: PathBuf::from(r"D:\SafeZone\sample_file.txt"),
            reasons: vec!["Plan passed safety checks".into()],
            validated_method_count: method_count,
            has_executable_methods: method_count > 0,
        }
    }

    fn make_blocked_safety() -> SafetyValidationResult {
        SafetyValidationResult {
            status: SafetyValidationStatus::Blocked,
            target_path: PathBuf::from(r"D:\SafeZone\sample_file.txt"),
            reasons: vec!["Target path is protected".into()],
            validated_method_count: 0,
            has_executable_methods: false,
        }
    }

    #[test]
    fn test_1_successful_simulation() {
        let engine = MockFileErasureEngine::new();
        let plan = make_test_plan(vec![
            FileSanitizationMethod::ContentOverwrite,
            FileSanitizationMethod::MetadataSanitization,
        ]);
        let safety = make_allowed_safety(2);
        let cancel = MockFileCancellationToken::new();

        let request = MockFileExecutionRequest {
            target_path: PathBuf::from(r"D:\SafeZone\sample_file.txt"),
            plan,
            safety_validation: safety,
            test_seed: Some("test_seed_1".into()),
        };

        let mut events = Vec::new();
        let result = engine
            .execute(&request, &cancel, &mut |e| events.push(e))
            .expect("Execution simulation must succeed");

        assert_eq!(result.outcome, MockFileExecutionOutcome::Completed);
        assert!(result.was_successful);
        assert!(result.is_simulated_mock);
        assert_eq!(result.simulated_methods.len(), 2);
        assert!(result.simulated_verification_token.is_some());
    }

    #[test]
    fn test_2_safety_blocked_execution_is_refused() {
        let engine = MockFileErasureEngine::new();
        let plan = make_test_plan(vec![FileSanitizationMethod::ContentOverwrite]);
        let safety = make_blocked_safety();
        let cancel = MockFileCancellationToken::new();

        let request = MockFileExecutionRequest {
            target_path: PathBuf::from(r"D:\SafeZone\sample_file.txt"),
            plan,
            safety_validation: safety,
            test_seed: None,
        };

        let mut events = Vec::new();
        let result = engine
            .execute(&request, &cancel, &mut |e| events.push(e))
            .expect("Execution must return a refusal outcome rather than an Err");

        assert_eq!(result.outcome, MockFileExecutionOutcome::RefusedSafetyBlocked);
        assert!(!result.was_successful);
        assert!(result.is_simulated_mock);
        assert!(result.simulated_verification_token.is_none());
        assert!(result.note.contains("SIMULATED MOCK RUN REFUSED"));
        assert!(events.is_empty());
    }

    #[test]
    fn test_3_zero_executable_methods_handled_safely() {
        let engine = MockFileErasureEngine::new();
        let plan = make_test_plan(vec![]);
        let safety = make_allowed_safety(0);
        let cancel = MockFileCancellationToken::new();

        let request = MockFileExecutionRequest {
            target_path: PathBuf::from(r"D:\SafeZone\sample_file.txt"),
            plan,
            safety_validation: safety,
            test_seed: None,
        };

        let mut events = Vec::new();
        let result = engine
            .execute(&request, &cancel, &mut |e| events.push(e))
            .expect("Zero methods execution must succeed safely");

        assert_eq!(result.outcome, MockFileExecutionOutcome::NoExecutableMethods);
        assert!(!result.was_successful);
        assert!(result.is_simulated_mock);
        assert!(result.note.contains("contains zero executable methods"));
        assert!(events.is_empty());
    }

    #[test]
    fn test_4_deterministic_verification_token() {
        let engine = MockFileErasureEngine::new();
        let plan = make_test_plan(vec![FileSanitizationMethod::MetadataSanitization]);
        let safety = make_allowed_safety(1);
        let cancel = MockFileCancellationToken::new();

        let request1 = MockFileExecutionRequest {
            target_path: PathBuf::from(r"D:\SafeZone\sample_file.txt"),
            plan: plan.clone(),
            safety_validation: safety.clone(),
            test_seed: Some("stable_seed".into()),
        };
        let request2 = request1.clone();

        let res1 = engine
            .execute(&request1, &cancel, &mut |_| {})
            .expect("Run 1 must succeed");
        let res2 = engine
            .execute(&request2, &cancel, &mut |_| {})
            .expect("Run 2 must succeed");

        assert_eq!(res1, res2);
        assert_eq!(
            res1.simulated_verification_token,
            res2.simulated_verification_token
        );
    }

    #[test]
    fn test_5_progress_reaches_100_percent() {
        let engine = MockFileErasureEngine::new();
        let plan = make_test_plan(vec![
            FileSanitizationMethod::ContentOverwrite,
            FileSanitizationMethod::MetadataSanitization,
        ]);
        let safety = make_allowed_safety(2);
        let cancel = MockFileCancellationToken::new();

        let request = MockFileExecutionRequest {
            target_path: PathBuf::from(r"D:\SafeZone\sample_file.txt"),
            plan,
            safety_validation: safety,
            test_seed: None,
        };

        let mut percentages = Vec::new();
        let result = engine
            .execute(&request, &cancel, &mut |e| percentages.push(e.percentage_complete))
            .expect("Execution must succeed");

        assert_eq!(result.outcome, MockFileExecutionOutcome::Completed);
        assert_eq!(percentages.first(), Some(&0));
        assert_eq!(percentages.last(), Some(&100));
    }

    #[test]
    fn test_6_cancellation_stops_simulation() {
        let engine = MockFileErasureEngine::new();
        let plan = make_test_plan(vec![
            FileSanitizationMethod::ContentOverwrite,
            FileSanitizationMethod::MetadataSanitization,
        ]);
        let safety = make_allowed_safety(2);
        let cancel = MockFileCancellationToken::new();
        cancel.cancel(); // Pre-cancel before execution

        let request = MockFileExecutionRequest {
            target_path: PathBuf::from(r"D:\SafeZone\sample_file.txt"),
            plan,
            safety_validation: safety,
            test_seed: None,
        };

        let result = engine
            .execute(&request, &cancel, &mut |_| {})
            .expect("Cancelled run must return Ok with Cancelled outcome");

        assert_eq!(result.outcome, MockFileExecutionOutcome::Cancelled);
        assert!(!result.was_successful);
        assert!(result.is_simulated_mock);
        assert!(result.simulated_verification_token.is_none());
    }

    #[test]
    fn test_7_each_supported_sanitization_method_is_recognized() {
        let engine = MockFileErasureEngine::new();
        let methods = [
            FileSanitizationMethod::ContentOverwrite,
            FileSanitizationMethod::MetadataSanitization,
            FileSanitizationMethod::AlternateDataStreamSanitization,
            FileSanitizationMethod::SlackSpaceSanitization,
            FileSanitizationMethod::FreeSpaceSanitization,
        ];

        for method in methods {
            let plan = make_test_plan(vec![method]);
            let safety = make_allowed_safety(1);
            let cancel = MockFileCancellationToken::new();

            let request = MockFileExecutionRequest {
                target_path: PathBuf::from(r"D:\SafeZone\sample_file.txt"),
                plan,
                safety_validation: safety,
                test_seed: None,
            };

            let mut events = Vec::new();
            let res = engine
                .execute(&request, &cancel, &mut |e| events.push(e))
                .expect("Execution must succeed");

            assert_eq!(res.outcome, MockFileExecutionOutcome::Completed);
            assert_eq!(res.simulated_methods, vec![method]);
            assert!(events.iter().any(|e| e.current_method == Some(method)));
        }
    }

    #[test]
    fn test_8_simulated_result_clearly_identifies_as_mock() {
        let engine = MockFileErasureEngine::new();
        let plan = make_test_plan(vec![FileSanitizationMethod::ContentOverwrite]);
        let safety = make_allowed_safety(1);
        let cancel = MockFileCancellationToken::new();

        let request = MockFileExecutionRequest {
            target_path: PathBuf::from(r"D:\SafeZone\sample_file.txt"),
            plan,
            safety_validation: safety,
            test_seed: None,
        };

        let result = engine
            .execute(&request, &cancel, &mut |_| {})
            .expect("Execution must succeed");

        assert!(result.is_simulated_mock);
        assert!(result.note.contains("SIMULATED MOCK RUN"));
        assert!(result.note.contains("NO REAL FILE OPERATIONS PERFORMED"));
    }

    #[test]
    fn test_9_no_physical_filesystem_access_occurs() {
        let engine = MockFileErasureEngine::new();
        // Virtual nonexistent path
        let non_existent = PathBuf::from(r"Z:\NonExistent\VirtualDisk\missing.dat");
        let plan = FileSanitizationPlan {
            target_path: non_existent.clone(),
            safety: SafetyClassification::SafeToAnalyze,
            proposed_methods: vec![MethodProposal {
                method: FileSanitizationMethod::MetadataSanitization,
                label: "Metadata Sanitization".into(),
                rationale: "Testing pure in-memory".into(),
                capability_state: CapabilityState::Supported,
            }],
            summary: "Plan summary".into(),
        };
        let safety = SafetyValidationResult {
            status: SafetyValidationStatus::Allowed,
            target_path: non_existent.clone(),
            reasons: vec!["Allowed".into()],
            validated_method_count: 1,
            has_executable_methods: true,
        };
        let cancel = MockFileCancellationToken::new();

        let request = MockFileExecutionRequest {
            target_path: non_existent,
            plan,
            safety_validation: safety,
            test_seed: None,
        };

        // Must succeed in memory without attempting to open or stat the nonexistent file
        let result = engine
            .execute(&request, &cancel, &mut |_| {})
            .expect("In-memory engine must succeed without filesystem handles");

        assert_eq!(result.outcome, MockFileExecutionOutcome::Completed);
    }

    #[test]
    fn test_10_deterministic_execution_result() {
        let engine = MockFileErasureEngine::new();
        let plan = make_test_plan(vec![
            FileSanitizationMethod::ContentOverwrite,
            FileSanitizationMethod::MetadataSanitization,
        ]);
        let safety = make_allowed_safety(2);
        let cancel = MockFileCancellationToken::new();

        let request1 = MockFileExecutionRequest {
            target_path: PathBuf::from(r"D:\SafeZone\sample_file.txt"),
            plan: plan.clone(),
            safety_validation: safety.clone(),
            test_seed: Some("fixed_seed".into()),
        };
        let request2 = request1.clone();

        let res1 = engine.execute(&request1, &cancel, &mut |_| {}).unwrap();
        let res2 = engine.execute(&request2, &cancel, &mut |_| {}).unwrap();

        assert_eq!(res1, res2);
    }

    #[test]
    fn test_11_multiple_methods_execute_in_defined_plan_order() {
        let engine = MockFileErasureEngine::new();
        let ordered_methods = vec![
            FileSanitizationMethod::ContentOverwrite,
            FileSanitizationMethod::MetadataSanitization,
            FileSanitizationMethod::AlternateDataStreamSanitization,
            FileSanitizationMethod::SlackSpaceSanitization,
            FileSanitizationMethod::FreeSpaceSanitization,
        ];
        let plan = make_test_plan(ordered_methods.clone());
        let safety = make_allowed_safety(5);
        let cancel = MockFileCancellationToken::new();

        let request = MockFileExecutionRequest {
            target_path: PathBuf::from(r"D:\SafeZone\sample_file.txt"),
            plan,
            safety_validation: safety,
            test_seed: None,
        };

        let result = engine
            .execute(&request, &cancel, &mut |_| {})
            .expect("Execution must succeed");

        assert_eq!(result.simulated_methods, ordered_methods);
    }

    #[test]
    fn test_12_empty_target_path_handled_safely() {
        let engine = MockFileErasureEngine::new();
        let plan = make_test_plan(vec![FileSanitizationMethod::ContentOverwrite]);
        let safety = make_allowed_safety(1);
        let cancel = MockFileCancellationToken::new();

        let request = MockFileExecutionRequest {
            target_path: PathBuf::from(""), // Empty path
            plan,
            safety_validation: safety,
            test_seed: None,
        };

        let err = engine
            .execute(&request, &cancel, &mut |_| {})
            .expect_err("Empty target path must produce an error");

        match err {
            EraserError::IoError { message, .. } => {
                assert!(message.contains("cannot be empty"));
            }
            other => panic!("Expected IoError, got: {:?}", other),
        }
    }
}