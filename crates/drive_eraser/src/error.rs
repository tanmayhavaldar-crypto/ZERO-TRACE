//! Error types for ForenX Drive Eraser.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DriveEraserError {
    #[error("I/O error encountered: {0}")]
    Io(#[from] std::io::Error),

    #[error("Device handle error for path '{path}': {message}")]
    DeviceAccess { path: String, message: String },

    #[error("Query property IOCTL failed on '{path}': {message}")]
    PropertyQueryFailed { path: String, message: String },

    #[error("Drive identity resolution failed: {0}")]
    IdentityResolutionFailed(String),

    #[error("Invalid device data: {0}")]
    InvalidData(String),

    #[error("Discovery error: {0}")]
    DiscoveryError(String),

    #[error("Safety violation for drive {device_id}: {reason}")]
    SafetyViolation { device_id: String, reason: String },

    #[error("Execution error on drive {device_id}: {message}")]
    ExecutionFailed { device_id: String, message: String },

    #[error("Operation cancelled for drive {device_id}")]
    Cancelled { device_id: String },
}

pub type Result<T> = std::result::Result<T, DriveEraserError>;