//! Shared IPC / data-transfer DTO foundations used by both `drive_eraser`
//! and `file_eraser` once they're wired up behind Tauri commands.
//!
//! These types intentionally carry no destructive-implementation
//! details — no device paths, no file paths, no method-specific
//! parameters, no erasure logic. Just the common status/result/request
//! shapes both modules will need.

use serde::{Deserialize, Serialize};

use crate::time::AuditTimestamp;

/// Schema version stamped on DTOs so a future consumer (including the
/// main ForenX app during integration) can detect drift between crate
/// versions.
pub const SCHEMA_VERSION: u16 = 1;

/// Identifies the operator (and, where applicable, a second signer)
/// driving a request. Populated by the Layer 2 RBAC dual sign-off gate
/// in the real application; passed through here as plain identifiers
/// only — this crate does not implement RBAC itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperatorContext {
    pub operator_id: String,
    pub authorizer_id: Option<String>,
}

/// Opaque job identifier shared by both modules' async operations.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId(pub String);

/// Opaque short-lived ticket identifier used by the two-step
/// request/confirm safety flow described in the design document.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TicketId(pub String);

/// Coarse lifecycle state of a job. Deliberately generic — neither
/// module-specific nor engine-specific.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    Pending,
    AwaitingConfirmation,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Final outcome classification for a completed/terminated job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationOutcome {
    Success,
    PartialSuccess,
    Failed,
    Cancelled,
}

/// Point-in-time status snapshot for a job, suitable for polling or as
/// a Tauri event payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JobStatusDto {
    pub schema_version: u16,
    pub job_id: JobId,
    pub state: JobState,
    pub percent_complete: Option<u8>,
    pub message: Option<String>,
    pub updated_at: AuditTimestamp,
}

impl JobStatusDto {
    /// Builds a fresh status snapshot stamped with the current time and
    /// crate's schema version.
    pub fn new(job_id: JobId, state: JobState) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            job_id,
            state,
            percent_complete: None,
            message: None,
            updated_at: AuditTimestamp::now(),
        }
    }
}

/// Structured error shape for IPC responses — never a bare string, so
/// the frontend can branch on `code` and still show `message` to the
/// operator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorDto {
    pub code: String,
    pub message: String,
}

impl From<&crate::error::EraserCommonError> for ErrorDto {
    fn from(err: &crate::error::EraserCommonError) -> Self {
        Self {
            code: err.code().to_string(),
            message: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn job_status_round_trips_through_json() {
        let status = JobStatusDto::new(JobId("job-1".to_string()), JobState::Running);
        let json = serde_json::to_string(&status).unwrap();
        let back: JobStatusDto = serde_json::from_str(&json).unwrap();
        assert_eq!(status, back);
    }

    #[test]
    fn job_state_serializes_as_snake_case() {
        let json = serde_json::to_string(&JobState::AwaitingConfirmation).unwrap();
        assert_eq!(json, "\"awaiting_confirmation\"");
    }

    #[test]
    fn operation_outcome_serializes_as_snake_case() {
        let json = serde_json::to_string(&OperationOutcome::PartialSuccess).unwrap();
        assert_eq!(json, "\"partial_success\"");
    }

    #[test]
    fn error_dto_built_from_common_error() {
        let err = crate::error::EraserCommonError::Validation("bad field".to_string());
        let dto: ErrorDto = (&err).into();
        assert_eq!(dto.code, "validation_error");
        assert!(dto.message.contains("bad field"));
    }
}
