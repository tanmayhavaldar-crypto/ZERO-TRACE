use drive_eraser::engine::{
    DriveErasureEngine, MockCancellationToken, MockExecutionRequest,
};
use drive_eraser::method_select::SanitizationMethod;
use drive_eraser::model::{DriveClassification, SafetyFlags, TriState};
use drive_eraser::safety::validate_drive_safety;
use drive_eraser::verification::verify_mock_execution_result;
use drive_eraser::MockDriveErasureEngine;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

/// Input used only for the Step 7.2 mock integration.
///
/// These values represent a simulated drive state.
/// They are NOT authoritative hardware discovery data.
#[derive(Debug, Clone, Deserialize)]
pub struct MockDriveErasureInput {
    pub target_device_id: String,
    pub physical_drive_index: Option<u32>,
    pub classification: DriveClassification,
    pub is_system: TriState,
    pub is_boot: TriState,
    pub is_removable: TriState,
    pub is_read_only: TriState,
    pub requires_conservatism: bool,
    pub selected_method: SanitizationMethod,
    pub test_seed: Option<String>,
}

/// Result returned by the Tauri command.
#[derive(Debug, Clone, Serialize)]
pub struct MockDriveErasureResponse {
    pub execution: drive_eraser::engine::MockExecutionResult,
    pub verification: drive_eraser::verification::MockVerificationResult,
}

/// Step 7.2 mock drive erasure command.
///
/// This command performs NO physical storage operation.
#[tauri::command]
pub fn run_mock_drive_erasure(
    app: AppHandle,
    input: MockDriveErasureInput,
) -> Result<MockDriveErasureResponse, String> {
    // 1. Build the safety flags from the mock input.
    let safety_flags = SafetyFlags {
        is_system: input.is_system,
        is_boot: input.is_boot,
        is_removable: input.is_removable,
        is_read_only: input.is_read_only,
        requires_conservatism: input.requires_conservatism,
    };

    // 2. Run the existing Step 5.3 safety validation.
    let safety_assessment =
        validate_drive_safety(Some(&input.classification), Some(&safety_flags));

    // 3. Build the existing Step 6.1 mock execution request.
    let request = MockExecutionRequest {
        target_device_id: input.target_device_id.clone(),
        physical_drive_index: input.physical_drive_index,
        selected_method: input.selected_method,
        safety_assessment,
        test_seed: input.test_seed,
    };

    // 4. Execute using the existing mock engine.
    let engine = MockDriveErasureEngine::new();
    let cancellation = MockCancellationToken::new();

    let mut progress_callback = |event| {
        let _ = app.emit("erasure://progress", event);
    };

    let execution_result = engine
        .execute(
            &request,
            &cancellation,
            &mut progress_callback,
        )
        .map_err(|error| error.to_string())?;

    // 5. Verify the existing Step 6.2 mock execution result.
    //
    // This is mock verification only. It does NOT prove
    // physical disk sanitization.
    let expected_token = execution_result
        .simulated_verification_token
        .as_deref();

    let verification_result =
        verify_mock_execution_result(&execution_result, expected_token)
            .map_err(|error| error.to_string())?;

    // 6. Return execution + verification results.
    Ok(MockDriveErasureResponse {
        execution: execution_result,
        verification: verification_result,
    })
}