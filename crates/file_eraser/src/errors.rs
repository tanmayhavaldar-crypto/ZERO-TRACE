use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum EraserError {
    #[error("Target does not exist or is inaccessible: {path:?} ({reason})")]
    TargetInaccessible { path: PathBuf, reason: String },

    #[error("I/O error at {path:?}: {message}")]
    IoError { path: PathBuf, message: String },

    #[error("Safety violation: target {path:?} is protected: {reason}")]
    SafetyViolation { path: PathBuf, reason: String },

    #[error("Reparse point query failed for {path:?}: {reason}")]
    ReparseQueryFailed { path: PathBuf, reason: String },

    #[error("Volume information query failed for {path:?}: {reason}")]
    VolumeQueryFailed { path: PathBuf, reason: String },

    #[error("Path normalization failed for {path:?}: {reason}")]
    NormalizationFailed { path: PathBuf, reason: String },

    #[error("Identity verification failed for {path:?}: expected {expected:?}, found {actual:?}")]
    IdentityMismatch {
        path: PathBuf,
        expected: String,
        actual: String,
    },
}

pub type EraserResult<T> = Result<T, EraserError>;
