//! ForenX Secure Eraser - Step 4: File Target Resolution & Filesystem Probing.
//!
//! STRICT READ-ONLY GUARANTEE:
//! This crate performs resolution, directory crawling, filesystem feature probing,
//! safety classification, read-only capability assessment (Step 5.1), read-only
//! sanitization method selection (Step 5.2), in-memory safety validation (Step 5.3),
//! mock file sanitization simulation (Step 6.1), and mock file verification (Step 6.2).
//! It does NOT delete, overwrite, truncate, write, lock volumes, dismount volumes,
//! or invoke destructive IOCTLs.
//!
//! Epistemic certainty rule:
//! Values are marked `ProbedValue::Verified(T)` or `CapabilityState::Supported` only
//! when authoritatively confirmed by OS queries or verified evidence. Uncertain,
//! unqueried, or failed probes MUST be preserved as `ProbedValue::Unavailable` or
//! `CapabilityState::Unknown` and never guessed.

pub mod capability;
pub mod engine;
pub mod errors;
pub mod filesystem_probe;
pub mod method_select;
pub mod model;
pub mod platform;
pub mod safety;
pub mod safety_validate;
pub mod target_resolver;
pub mod traversal;
pub mod verification;

pub use capability::{
    assess_target_capabilities, CapabilityFinding, CapabilityState, FileCapabilityAssessment,
};
pub use engine::{
    FileErasureEngine, MockFileCancellationToken, MockFileErasureEngine,
    MockFileExecutionOutcome, MockFileExecutionRequest, MockFileExecutionResult,
    MockFileProgressEvent,
};
pub use errors::{EraserError, EraserResult};
pub use filesystem_probe::FilesystemProber;
pub use method_select::{
    plan_target_sanitization, FileSanitizationMethod, FileSanitizationPlan, MethodProposal,
};
pub use model::*;
pub use platform::mock::{MockEntryConfig, MockPlatformProvider};
pub use platform::PlatformProvider;
pub use safety::{is_descendant_or_equal, SafetyEngine};
pub use safety_validate::{
    validate_sanitization_plan, SafetyValidationResult, SafetyValidationStatus,
};
pub use target_resolver::TargetResolver;
pub use traversal::DirectoryCrawler;
pub use verification::{
    verify_mock_execution_result, verify_mock_file_erasure, MockFileVerificationRequest,
    MockFileVerificationResult, VerificationVerdict,
};