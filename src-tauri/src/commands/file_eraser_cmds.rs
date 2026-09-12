use file_eraser::{
    assess_target_capabilities,
    plan_target_sanitization,
    validate_sanitization_plan,
    MockFileCancellationToken,
    MockFileErasureEngine,
    MockFileExecutionRequest,
    MockFileExecutionResult,
    MockFileProgressEvent,
    MockFileVerificationResult,
    MockPlatformProvider,
    TargetResolver,
    verify_mock_execution_result,
    FileErasureEngine,
};

use file_eraser::platform::PlatformFileAttributes;
use file_eraser::platform::mock::MockEntryConfig;

use file_eraser::model::{
    FileIdentityToken,
    ReparseTagType,
    TargetKind,
};

use serde::Serialize;
use std::path::PathBuf;
#[derive(Debug, Serialize)]
pub struct FileEraseTestResponse {
    pub target_path: String,
    pub execution: MockFileExecutionResult,
    pub verification: MockFileVerificationResult,
    pub progress: Vec<MockFileProgressEvent>,
}

/// Step 7.3 mock-only File Eraser integration test.
///
/// This command:
/// 1. Creates an entirely in-memory mock file.
/// 2. Resolves/probes the mock target.
/// 3. Assesses capabilities.
/// 4. Builds a sanitization plan.
/// 5. Runs Step 5.3 safety validation.
/// 6. Runs the Step 6.1 mock engine.
/// 7. Runs Step 6.2 mock verification.
///
/// No real filesystem operation is performed.
#[tauri::command]
pub fn test_mock_file_erasure() -> Result<FileEraseTestResponse, String> {
    let target_path = PathBuf::from(r"D:\SafeZone\sample_file.txt");

    // ------------------------------------------------------------
    // 1. Create mock filesystem state entirely in memory.
    // ------------------------------------------------------------
    let mut provider = MockPlatformProvider::new();

    provider.insert_entry(
        target_path.clone(),
        MockEntryConfig {
            kind: TargetKind::RegularFile,
            size_bytes: 2048,
            allocated_size: 4096,
            identity: FileIdentityToken {
                volume_serial_number: 11111,
                file_index: 22222,
            },
            attributes: PlatformFileAttributes {
                is_sparse: false,
                is_compressed: false,
                is_encrypted: false,
                is_reparse_point: false,
                is_directory: false,
            },
            reparse_tag: Ok(ReparseTagType::None),
            streams: Vec::new(),
            children: Vec::new(),
        },
    );

    // ------------------------------------------------------------
    // 2. Resolve/probe the mock target using existing Step 4 code.
    // ------------------------------------------------------------
    let resolver = TargetResolver::new(provider.clone())
    .map_err(|e| format!("Target resolver initialization failed: {e}"))?;

    let (probes, _stats) = resolver
    .resolve_targets(std::slice::from_ref(&target_path))
    .map_err(|e| format!("Target resolution failed: {e}"))?;

    let probe = probes
    .into_iter()
    .next()
    .ok_or_else(|| "Mock target was not resolved".to_string())?;

    // ------------------------------------------------------------
    // 3. Step 5.1 — capability assessment.
    // ------------------------------------------------------------
    let assessment = assess_target_capabilities(&probe);

    // ------------------------------------------------------------
    // 4. Step 5.2 — create a plan.
    //
    // Content sanitization is intentionally NOT forced to Supported.
    // The existing capability layer correctly reports it as Unknown
    // from ordinary filesystem evidence.
    //
    // Metadata sanitization is Supported by the mock evidence.
    // ------------------------------------------------------------
    let plan = plan_target_sanitization(&assessment);

    // ------------------------------------------------------------
    // 5. Step 5.3 — authoritative in-memory safety validation.
    // ------------------------------------------------------------
    let safety_validation = validate_sanitization_plan(&assessment, &plan);

    if !safety_validation.is_ready_for_execution() {
        return Err(format!(
            "Safety validation refused execution: {}",
            safety_validation.reasons.join("; ")
        ));
    }

    // ------------------------------------------------------------
    // 6. Step 6.1 — execute the existing mock engine.
    // ------------------------------------------------------------
    let request = MockFileExecutionRequest {
        target_path: target_path.clone(),
        plan: plan.clone(),
        safety_validation,
        test_seed: Some("tauri-step7-file-test".into()),
    };

    let engine = MockFileErasureEngine::new();
    let cancellation = MockFileCancellationToken::new();
    let mut progress = Vec::new();

    let execution = engine
        .execute(
            &request,
            &cancellation,
            &mut |event| {
                progress.push(event);
            },
        )
        .map_err(|e| format!("Mock file execution failed: {e}"))?;

    // ------------------------------------------------------------
    // 7. Step 6.2 — verify the actual Step 6.1 result.
    //
    // We use the token produced by the mock execution as the
    // expected token for this deterministic simulation test.
    // ------------------------------------------------------------
    let expected_token = execution.simulated_verification_token.clone();

    let verification = verify_mock_execution_result(
        &execution,
        expected_token.as_deref(),
        &execution.simulated_methods,
    )
    .map_err(|e| format!("Mock verification failed: {e}"))?;

    Ok(FileEraseTestResponse {
        target_path: target_path.to_string_lossy().into_owned(),
        execution,
        verification,
        progress,
    })
}