//! SHA-256 hashing helpers shared across the audit log, verification
//! sampling, and report-integrity use cases described in the design
//! document. This module has nothing to do with sanitization/wiping —
//! it only computes and validates digests.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{EraserCommonError, EraserCommonResult};

/// A SHA-256 digest, stored as lowercase hex (64 characters), suitable
/// for embedding directly in audit records and reports.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Sha256Hex(String);

impl Sha256Hex {
    /// Hashes `data` and returns its SHA-256 digest as lowercase hex.
    pub fn from_bytes(data: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let digest = hasher.finalize();
        Self(to_hex(&digest))
    }

    /// Parses an existing lowercase-hex SHA-256 string (e.g. one read
    /// back from storage), validating its shape (64 hex characters).
    pub fn parse(s: &str) -> EraserCommonResult<Self> {
        if s.len() != 64 || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(EraserCommonError::Validation(format!(
                "expected a 64-character hex SHA-256 string, got: {s}"
            )));
        }
        Ok(Self(s.to_ascii_lowercase()))
    }

    /// A reserved all-zero digest used as the `previous_entry_hash` of
    /// the first ("genesis") entry in an audit chain. It is not the
    /// hash of any real content.
    pub fn genesis() -> Self {
        Self("0".repeat(64))
    }

    /// Borrows the lowercase-hex representation.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Sha256Hex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_vector_empty_string() {
        // SHA-256("") — well-known test vector.
        let hash = Sha256Hex::from_bytes(b"");
        assert_eq!(
            hash.as_str(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn same_input_hashes_identically() {
        let a = Sha256Hex::from_bytes(b"forenx");
        let b = Sha256Hex::from_bytes(b"forenx");
        assert_eq!(a, b);
    }

    #[test]
    fn different_input_hashes_differently() {
        let a = Sha256Hex::from_bytes(b"forenx-a");
        let b = Sha256Hex::from_bytes(b"forenx-b");
        assert_ne!(a, b);
    }

    #[test]
    fn parse_rejects_wrong_length() {
        assert!(Sha256Hex::parse("abc").is_err());
    }

    #[test]
    fn parse_rejects_non_hex_characters() {
        let bad = "z".repeat(64);
        assert!(Sha256Hex::parse(&bad).is_err());
    }

    #[test]
    fn parse_accepts_a_value_this_module_produced() {
        let hash = Sha256Hex::from_bytes(b"round trip me");
        let parsed = Sha256Hex::parse(hash.as_str()).unwrap();
        assert_eq!(hash, parsed);
    }

    #[test]
    fn genesis_is_64_zero_chars() {
        let g = Sha256Hex::genesis();
        assert_eq!(g.as_str(), "0".repeat(64));
    }
}
