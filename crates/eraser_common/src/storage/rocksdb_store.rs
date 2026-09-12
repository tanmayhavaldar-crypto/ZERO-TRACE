//! Optional RocksDB-backed key-value store for high-volume,
//! non-relational progress/sample state (e.g. per-range sanitize
//! progress markers for a large drive).
//!
//! This is intentionally **not** part of the audit log system of
//! record — see [`crate::storage::sqlite_store`] for that. The normal
//! audit flow never depends on this module. Its contents only compile
//! when the `rocksdb-store` feature is enabled; with the feature off
//! (the default), this file contributes nothing to the build.

#[cfg(feature = "rocksdb-store")]
mod enabled {
    use rocksdb::DB;

    use crate::error::{EraserCommonError, EraserCommonResult};

    /// Simple key-value wrapper around a RocksDB instance for optional,
    /// high-volume progress/sample state. Never used for audit data —
    /// audit data always goes through [`crate::storage::AuditStore`].
    pub struct RocksDbProgressStore {
        db: DB,
    }

    impl RocksDbProgressStore {
        /// Opens (creating if needed) a RocksDB instance at `path`.
        pub fn open(path: &str) -> EraserCommonResult<Self> {
            let db = DB::open_default(path)
                .map_err(|e| EraserCommonError::Storage(format!("failed to open rocksdb: {e}")))?;
            Ok(Self { db })
        }

        /// Stores `value` under `key`, overwriting any existing value.
        pub fn put(&self, key: &str, value: &[u8]) -> EraserCommonResult<()> {
            self.db
                .put(key.as_bytes(), value)
                .map_err(|e| EraserCommonError::Storage(format!("rocksdb put failed: {e}")))
        }

        /// Reads the value stored under `key`, if any.
        pub fn get(&self, key: &str) -> EraserCommonResult<Option<Vec<u8>>> {
            self.db
                .get(key.as_bytes())
                .map_err(|e| EraserCommonError::Storage(format!("rocksdb get failed: {e}")))
        }

        /// Removes the value stored under `key`, if any.
        pub fn delete(&self, key: &str) -> EraserCommonResult<()> {
            self.db
                .delete(key.as_bytes())
                .map_err(|e| EraserCommonError::Storage(format!("rocksdb delete failed: {e}")))
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn put_get_delete_round_trip() {
            let dir =
                std::env::temp_dir().join(format!("forenx-rocksdb-test-{}", std::process::id()));
            let store = RocksDbProgressStore::open(dir.to_str().unwrap()).unwrap();

            store.put("range:0-1024", b"in_progress").unwrap();
            assert_eq!(
                store.get("range:0-1024").unwrap(),
                Some(b"in_progress".to_vec())
            );

            store.delete("range:0-1024").unwrap();
            assert_eq!(store.get("range:0-1024").unwrap(), None);

            let _ = std::fs::remove_dir_all(&dir);
        }
    }
}

#[cfg(feature = "rocksdb-store")]
pub use enabled::RocksDbProgressStore;
