//! SQLite-backed implementation of [`AuditStore`] — the system of
//! record for ForenX audit data.
//!
//! Schema is intentionally minimal (one append-only table). No `UPDATE`
//! or `DELETE` statement exists anywhere in this module; a production
//! deployment should additionally enforce append-only behavior at the
//! database level (e.g. triggers or restricted grants), as noted in the
//! design document.

use std::sync::Mutex;

use rusqlite::{params, Connection, Row};

use super::AuditStore;
use crate::audit_log::{AuditEntry, AuditEntryFields};
use crate::error::{EraserCommonError, EraserCommonResult};
use crate::hashing::Sha256Hex;

/// SQLite-backed audit store. Wraps a single [`rusqlite::Connection`]
/// behind a mutex since `Connection` is not `Sync`.
pub struct SqliteAuditStore {
    conn: Mutex<Connection>,
}

impl SqliteAuditStore {
    /// Opens (creating if needed) a SQLite database file at `path` and
    /// ensures the `audit_log` table exists.
    pub fn open(path: &str) -> EraserCommonResult<Self> {
        let conn = Connection::open(path)
            .map_err(|e| EraserCommonError::Storage(format!("failed to open sqlite db: {e}")))?;
        let store = Self {
            conn: Mutex::new(conn),
        };
        store.init_schema()?;
        Ok(store)
    }

    /// Opens an in-memory SQLite database. Useful for tests and for the
    /// mock-backend-first test harness — never for real audit data.
    pub fn open_in_memory() -> EraserCommonResult<Self> {
        let conn = Connection::open_in_memory().map_err(|e| {
            EraserCommonError::Storage(format!("failed to open in-memory sqlite db: {e}"))
        })?;
        let store = Self {
            conn: Mutex::new(conn),
        };
        store.init_schema()?;
        Ok(store)
    }

    fn init_schema(&self) -> EraserCommonResult<()> {
        let conn = self.lock_conn()?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS audit_log (
                sequence            INTEGER PRIMARY KEY,
                previous_entry_hash TEXT NOT NULL,
                entry_hash          TEXT NOT NULL,
                fields_json         TEXT NOT NULL
            );",
        )
        .map_err(|e| EraserCommonError::Storage(format!("failed to init schema: {e}")))?;
        Ok(())
    }

    fn lock_conn(&self) -> EraserCommonResult<std::sync::MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|_| EraserCommonError::Internal("sqlite connection mutex poisoned".into()))
    }
}

impl AuditStore for SqliteAuditStore {
    fn append_entry(&self, entry: &AuditEntry) -> EraserCommonResult<()> {
        let fields_json = serde_json::to_string(&entry.fields)?;
        let conn = self.lock_conn()?;
        conn.execute(
            "INSERT INTO audit_log (sequence, previous_entry_hash, entry_hash, fields_json)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                entry.sequence as i64,
                entry.previous_entry_hash.as_str(),
                entry.entry_hash.as_str(),
                fields_json,
            ],
        )
        .map_err(|e| EraserCommonError::Storage(format!("failed to append entry: {e}")))?;
        Ok(())
    }

    fn latest_entry(&self) -> EraserCommonResult<Option<AuditEntry>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT sequence, previous_entry_hash, entry_hash, fields_json
                 FROM audit_log ORDER BY sequence DESC LIMIT 1",
            )
            .map_err(|e| EraserCommonError::Storage(e.to_string()))?;
        let mut rows = stmt
            .query([])
            .map_err(|e| EraserCommonError::Storage(e.to_string()))?;
        match rows
            .next()
            .map_err(|e| EraserCommonError::Storage(e.to_string()))?
        {
            Some(row) => Ok(Some(row_to_entry(row)?)),
            None => Ok(None),
        }
    }

    fn all_entries(&self) -> EraserCommonResult<Vec<AuditEntry>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT sequence, previous_entry_hash, entry_hash, fields_json
                 FROM audit_log ORDER BY sequence ASC",
            )
            .map_err(|e| EraserCommonError::Storage(e.to_string()))?;
        let mut rows = stmt
            .query([])
            .map_err(|e| EraserCommonError::Storage(e.to_string()))?;
        let mut out = Vec::new();
        while let Some(row) = rows
            .next()
            .map_err(|e| EraserCommonError::Storage(e.to_string()))?
        {
            out.push(row_to_entry(row)?);
        }
        Ok(out)
    }
}

fn row_to_entry(row: &Row) -> EraserCommonResult<AuditEntry> {
    let sequence: i64 = row
        .get(0)
        .map_err(|e| EraserCommonError::Storage(e.to_string()))?;
    let previous_entry_hash: String = row
        .get(1)
        .map_err(|e| EraserCommonError::Storage(e.to_string()))?;
    let entry_hash: String = row
        .get(2)
        .map_err(|e| EraserCommonError::Storage(e.to_string()))?;
    let fields_json: String = row
        .get(3)
        .map_err(|e| EraserCommonError::Storage(e.to_string()))?;

    let fields: AuditEntryFields = serde_json::from_str(&fields_json)?;

    Ok(AuditEntry {
        sequence: sequence as u64,
        previous_entry_hash: Sha256Hex::parse(&previous_entry_hash)?,
        entry_hash: Sha256Hex::parse(&entry_hash)?,
        fields,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit_log::AuditChainBuilder;
    use crate::time::AuditTimestamp;
    use serde_json::json;

    fn sample_fields(event: &str) -> AuditEntryFields {
        AuditEntryFields {
            event: event.to_string(),
            job_id: None,
            operator_id: None,
            details: json!({}),
            timestamp: AuditTimestamp::now(),
        }
    }

    #[test]
    fn append_and_read_back_entries() {
        let store = SqliteAuditStore::open_in_memory().unwrap();
        let mut builder = AuditChainBuilder::new();
        let e0 = builder.append(sample_fields("job_requested")).unwrap();
        let e1 = builder.append(sample_fields("job_completed")).unwrap();

        store.append_entry(&e0).unwrap();
        store.append_entry(&e1).unwrap();

        let latest = store.latest_entry().unwrap().unwrap();
        assert_eq!(latest.sequence, 1);
        assert_eq!(latest.entry_hash, e1.entry_hash);

        let all = store.all_entries().unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].entry_hash, e0.entry_hash);
        assert_eq!(all[1].entry_hash, e1.entry_hash);
    }

    #[test]
    fn empty_store_has_no_latest_entry() {
        let store = SqliteAuditStore::open_in_memory().unwrap();
        assert!(store.latest_entry().unwrap().is_none());
        assert!(store.all_entries().unwrap().is_empty());
    }
}
