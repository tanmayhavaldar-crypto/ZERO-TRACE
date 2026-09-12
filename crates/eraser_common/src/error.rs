//! Common error representation shared by `drive_eraser`, `file_eraser`,
//! and `eraser_common` itself.
//!
//! Kept deliberately small: one enum covering the categories the shared
//! foundation actually needs (validation, configuration, storage,
//! serialization, internal), plus a [`EraserCommonError::code`] method
//! for stable machine-readable identifiers usable in
//! [`crate::ipc_dto::ErrorDto`].

use thiserror::Error;

/// Common error type for shared ForenX Secure Eraser infrastructure.
#[derive(Debug, Error)]
pub enum EraserCommonError {
    /// Input failed validation (e.g. malformed hash string, bad DTO field).
    #[error("validation error: {0}")]
    Validation(String),

    /// A configuration value was missing or invalid.
    #[error("configuration error: {0}")]
    Configuration(String),

    /// A storage backend (SQLite, RocksDB) operation failed.
    #[error("storage error: {0}")]
    Storage(String),

    /// (De)serialization of a DTO or audit record failed.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// An unexpected internal error (e.g. a poisoned mutex).
    #[error("internal error: {0}")]
    Internal(String),
}

impl EraserCommonError {
    /// Stable, machine-readable identifier for this error category.
    /// Intended for [`crate::ipc_dto::ErrorDto::code`], not for humans.
    pub fn code(&self) -> &'static str {
        match self {
            EraserCommonError::Validation(_) => "validation_error",
            EraserCommonError::Configuration(_) => "configuration_error",
            EraserCommonError::Storage(_) => "storage_error",
            EraserCommonError::Serialization(_) => "serialization_error",
            EraserCommonError::Internal(_) => "internal_error",
        }
    }
}

/// Convenience alias used throughout `eraser_common` (and, later,
/// `drive_eraser`/`file_eraser`).
pub type EraserCommonResult<T> = Result<T, EraserCommonError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_error_has_expected_code_and_message() {
        let err = EraserCommonError::Validation("bad input".to_string());
        assert_eq!(err.code(), "validation_error");
        assert!(err.to_string().contains("bad input"));
    }

    #[test]
    fn storage_error_has_expected_code() {
        let err = EraserCommonError::Storage("disk full".to_string());
        assert_eq!(err.code(), "storage_error");
    }

    #[test]
    fn serialization_error_converts_from_serde_json_error() {
        let bad_json = "{ not valid json";
        let parse_err = serde_json::from_str::<serde_json::Value>(bad_json).unwrap_err();
        let err: EraserCommonError = parse_err.into();
        assert_eq!(err.code(), "serialization_error");
    }
}
