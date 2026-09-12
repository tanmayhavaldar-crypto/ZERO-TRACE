//! ForenX Secure Drive Eraser crate.

pub mod capability;
pub mod classify;
pub mod discovery;
pub mod engine;
pub mod error;
pub mod identity;
pub mod method_select;
pub mod model;
pub mod safety;
pub mod verification;

pub use capability::{
    default_readonly_capabilities, evaluate_capabilities_from_evidence, evaluate_safe_capabilities,
    AtaCapabilityEvidence, CapabilityEvidence, NvmeCapabilityEvidence, ScsiCapabilityEvidence,
};
pub use classify::classify_drive;
pub use engine::{
    DriveErasureEngine, MockCancellationToken, MockDriveErasureEngine, MockExecutionOutcome,
    MockExecutionRequest, MockExecutionResult, MockProgressEvent,
};
pub use error::{DriveEraserError, Result};
pub use identity::resolve_drive_identity;
pub use method_select::{
    select_sanitization_methods, MethodCandidate, MethodProposal, SanitizationMethod,
};
pub use model::{
    BusType, DiscoveredDrive, DriveCapabilities, DriveClassification, MediaType, SafetyFlags,
    TriState,
};
pub use safety::{
    validate_discovered_drive, validate_drive_safety, BlockReasonCode, SafetyAssessment,
    SafetyFinding, SafetyVerdict,
};
pub use verification::{
    verify_mock_erasure, verify_mock_execution_result, MockVerificationRequest,
    MockVerificationResult, VerificationVerdict,
};