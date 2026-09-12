//! Storage foundation shared by `drive_eraser` and `file_eraser`.
//!
//! SQLite ([`sqlite_store`]) is the system of record for the append-only
//! audit log. RocksDB ([`rocksdb_store`]) is optional, feature-gated
//! (`rocksdb-store`), and intended only for high-volume progress/sample
//! state — the normal audit flow never depends on it.

pub mod rocksdb_store;
pub mod sqlite_store;

use crate::audit_log::AuditEntry;
use crate::error::EraserCommonResult;

/// Storage abstraction for the append-only, tamper-evident audit log.
///
/// Implementations MUST be append-only in spirit: no method on this
/// trait allows updating or deleting an existing entry. (Real deployments
/// should additionally enforce this at the database level — see the
/// design document's note on triggers/restricted grants.)
pub trait AuditStore {
    /// Appends a new entry to the store.
    fn append_entry(&self, entry: &AuditEntry) -> EraserCommonResult<()>;

    /// Returns the most recently appended entry, if any.
    fn latest_entry(&self) -> EraserCommonResult<Option<AuditEntry>>;

    /// Returns all entries in sequence order. Intended for verification
    /// and reporting, not for hot-path use on very large logs.
    fn all_entries(&self) -> EraserCommonResult<Vec<AuditEntry>>;
}
