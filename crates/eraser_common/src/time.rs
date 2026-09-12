//! UTC timestamp representation used by audit records and job status
//! DTOs.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A UTC timestamp, serialized as RFC 3339 (e.g. `2026-09-08T07:12:34Z`).
///
/// A thin wrapper around `chrono::DateTime<Utc>` so the rest of the
/// crate has one clear, explicit "this is a UTC audit timestamp" type
/// rather than passing `DateTime<Utc>` around directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AuditTimestamp(DateTime<Utc>);

impl AuditTimestamp {
    /// Captures the current UTC time.
    pub fn now() -> Self {
        Self(Utc::now())
    }

    /// Wraps an existing UTC `DateTime`.
    pub fn from_utc(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }

    /// Returns the underlying `chrono::DateTime<Utc>`.
    pub fn as_datetime(&self) -> DateTime<Utc> {
        self.0
    }

    /// Formats as an RFC 3339 string.
    pub fn to_rfc3339(&self) -> String {
        self.0.to_rfc3339()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_json() {
        let ts = AuditTimestamp::now();
        let json = serde_json::to_string(&ts).expect("serialize");
        let back: AuditTimestamp = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(ts, back);
    }

    #[test]
    fn to_rfc3339_is_nonempty_and_parseable() {
        let ts = AuditTimestamp::now();
        let s = ts.to_rfc3339();
        assert!(!s.is_empty());
        assert!(DateTime::parse_from_rfc3339(&s).is_ok());
    }
}
