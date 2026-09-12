//! ForenX Secure Eraser — shared/common crate.
//!
//! Provides reusable, non-destructive infrastructure consumed by both
//! `drive_eraser` and `file_eraser`:
//! - [`error`] — common error type
//! - [`time`] — UTC timestamp representation
//! - [`hashing`] — SHA-256 helpers
//! - [`ipc_dto`] — shared IPC / data-transfer DTOs
//! - [`audit_log`] — append-only, tamper-evident audit record foundation
//! - [`storage`] — SQLite (system of record) + optional RocksDB storage
//!
//! STATUS: Step 2 (shared foundation) of the ForenX Secure Eraser
//! implementation. No drive/file discovery, classification, capability
//! detection, method selection, or erasure engine logic lives here or
//! anywhere else in this workspace yet — `drive_eraser` and
//! `file_eraser` remain unimplemented placeholders as of this step, and
//! nothing in this crate performs any destructive operation.

pub mod audit_log;
pub mod error;
pub mod hashing;
pub mod ipc_dto;
pub mod storage;
pub mod time;

pub use error::{EraserCommonError, EraserCommonResult};
pub use hashing::Sha256Hex;
pub use time::AuditTimestamp;
