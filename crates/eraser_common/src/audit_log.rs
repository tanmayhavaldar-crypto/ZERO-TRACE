//! Append-only, tamper-evident audit record foundation.
//!
//! **Tamper-evident, not tamper-proof.** Verifying a chain can prove
//! whether stored entries have been altered since their hashes were
//! computed, and whether the sequence has been reordered or spliced.
//! It cannot, by itself, *prevent* someone with write access to the
//! underlying storage from altering it — that protection has to come
//! from storage-level controls (append-only tables/permissions, see
//! [`crate::storage`]) and/or external anchoring, both handled outside
//! this shared crate.
//!
//! Hash-chain formula:
//! `entry_hash = SHA256(previous_entry_hash || canonical_json(fields))`

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::EraserCommonResult;
use crate::hashing::Sha256Hex;
use crate::time::AuditTimestamp;

/// The canonical, hashed content of one audit entry.
///
/// Deliberately a fixed-shape struct (not a free-form map) so
/// serialization order is stable/deterministic across runs — this is
/// what makes `entry_hash` reproducible. Nested structured data belongs
/// in `details`, which relies on `serde_json::Value`'s default
/// (sorted-key) map representation for the same reason — this crate
/// does not enable serde_json's `preserve_order` feature.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditEntryFields {
    /// The event being recorded, e.g. `"job_requested"`, `"gate_blocked"`,
    /// `"engine_completed"`. Free text at this layer — `drive_eraser`
    /// and `file_eraser` define their own event vocabularies later.
    pub event: String,
    /// Job this entry relates to, if any.
    pub job_id: Option<String>,
    /// Operator (and/or authorizer) responsible for the action, if any.
    pub operator_id: Option<String>,
    /// Structured, event-specific payload. Use `Value::Null` when
    /// there's nothing to attach.
    pub details: Value,
    /// When the underlying event occurred (not when it was hashed or
    /// stored).
    pub timestamp: AuditTimestamp,
}

/// One entry in a hash-chained audit log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Zero-based position in the chain.
    pub sequence: u64,
    /// Hash of the previous entry (or [`Sha256Hex::genesis`] for the
    /// first entry in a chain).
    pub previous_entry_hash: Sha256Hex,
    /// `SHA256(previous_entry_hash || canonical_json(fields))`.
    pub entry_hash: Sha256Hex,
    /// The entry's canonical content.
    pub fields: AuditEntryFields,
}

/// Serializes `fields` deterministically. The same [`AuditEntryFields`]
/// value always produces the same string, which is what makes
/// [`compute_entry_hash`] reproducible for later verification.
pub fn canonical_json(fields: &AuditEntryFields) -> EraserCommonResult<String> {
    Ok(serde_json::to_string(fields)?)
}

/// Computes `SHA256(previous_entry_hash || canonical_json(fields))`.
pub fn compute_entry_hash(
    previous_entry_hash: &Sha256Hex,
    fields: &AuditEntryFields,
) -> EraserCommonResult<Sha256Hex> {
    let canonical = canonical_json(fields)?;
    let mut buf = Vec::with_capacity(previous_entry_hash.as_str().len() + canonical.len());
    buf.extend_from_slice(previous_entry_hash.as_str().as_bytes());
    buf.extend_from_slice(canonical.as_bytes());
    Ok(Sha256Hex::from_bytes(&buf))
}

impl AuditEntry {
    /// Builds the first entry in a new chain (`previous_entry_hash` is
    /// the reserved genesis marker, not a real prior hash).
    pub fn genesis(fields: AuditEntryFields) -> EraserCommonResult<Self> {
        let previous_entry_hash = Sha256Hex::genesis();
        let entry_hash = compute_entry_hash(&previous_entry_hash, &fields)?;
        Ok(Self {
            sequence: 0,
            previous_entry_hash,
            entry_hash,
            fields,
        })
    }

    /// Builds the next entry, chained after `previous`.
    pub fn next(previous: &AuditEntry, fields: AuditEntryFields) -> EraserCommonResult<Self> {
        let previous_entry_hash = previous.entry_hash.clone();
        let entry_hash = compute_entry_hash(&previous_entry_hash, &fields)?;
        Ok(Self {
            sequence: previous.sequence + 1,
            previous_entry_hash,
            entry_hash,
            fields,
        })
    }

    /// Recomputes this entry's hash from its own stored fields and
    /// `previous_entry_hash`, and checks it matches `entry_hash`.
    ///
    /// Detects whether this entry's stored data was altered after the
    /// hash was originally computed. See the module-level note on
    /// "tamper-evident" vs "tamper-proof".
    pub fn verify_self(&self) -> EraserCommonResult<bool> {
        let recomputed = compute_entry_hash(&self.previous_entry_hash, &self.fields)?;
        Ok(recomputed == self.entry_hash)
    }
}

/// Verifies an ordered slice of entries: each entry's own hash is
/// correct, and each entry correctly links to the one before it
/// (matching `previous_entry_hash` and a contiguous `sequence`).
///
/// An empty slice is trivially valid. This does not check that
/// `entries[0]` is a true chain genesis — callers verifying a full
/// chain from the start should additionally check
/// `entries[0].previous_entry_hash == Sha256Hex::genesis()`.
pub fn verify_chain(entries: &[AuditEntry]) -> EraserCommonResult<bool> {
    for (i, entry) in entries.iter().enumerate() {
        if !entry.verify_self()? {
            return Ok(false);
        }
        if i > 0 {
            let previous = &entries[i - 1];
            if entry.previous_entry_hash != previous.entry_hash {
                return Ok(false);
            }
            if entry.sequence != previous.sequence + 1 {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

/// Convenience builder for constructing a chain of entries in memory,
/// e.g. before handing them to an [`crate::storage::AuditStore`].
#[derive(Debug, Default)]
pub struct AuditChainBuilder {
    last_entry: Option<AuditEntry>,
}

impl AuditChainBuilder {
    pub fn new() -> Self {
        Self { last_entry: None }
    }

    /// Appends a new entry to the in-memory chain and returns it.
    pub fn append(&mut self, fields: AuditEntryFields) -> EraserCommonResult<AuditEntry> {
        let entry = match &self.last_entry {
            None => AuditEntry::genesis(fields)?,
            Some(previous) => AuditEntry::next(previous, fields)?,
        };
        self.last_entry = Some(entry.clone());
        Ok(entry)
    }

    /// The most recently appended entry, if any.
    pub fn latest(&self) -> Option<&AuditEntry> {
        self.last_entry.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_fields(event: &str) -> AuditEntryFields {
        AuditEntryFields {
            event: event.to_string(),
            job_id: Some("job-123".to_string()),
            operator_id: Some("operator-1".to_string()),
            details: json!({ "note": "test" }),
            timestamp: AuditTimestamp::now(),
        }
    }

    #[test]
    fn canonical_json_is_deterministic() {
        let fields = sample_fields("job_requested");
        let a = canonical_json(&fields).unwrap();
        let b = canonical_json(&fields).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn entry_hash_is_deterministic_for_same_inputs() {
        let previous = Sha256Hex::genesis();
        let fields = sample_fields("job_requested");
        let h1 = compute_entry_hash(&previous, &fields).unwrap();
        let h2 = compute_entry_hash(&previous, &fields).unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn entry_hash_changes_if_fields_change() {
        let previous = Sha256Hex::genesis();
        let fields_a = sample_fields("job_requested");
        let mut fields_b = fields_a.clone();
        fields_b.event = "job_cancelled".to_string();

        let h1 = compute_entry_hash(&previous, &fields_a).unwrap();
        let h2 = compute_entry_hash(&previous, &fields_b).unwrap();
        assert_ne!(h1, h2);
    }

    #[test]
    fn entry_hash_changes_if_previous_hash_changes() {
        let fields = sample_fields("job_requested");
        let h1 = compute_entry_hash(&Sha256Hex::genesis(), &fields).unwrap();
        let other_previous = Sha256Hex::from_bytes(b"not genesis");
        let h2 = compute_entry_hash(&other_previous, &fields).unwrap();
        assert_ne!(h1, h2);
    }

    #[test]
    fn chain_of_entries_verifies_ok() {
        let mut builder = AuditChainBuilder::new();
        let e0 = builder.append(sample_fields("job_requested")).unwrap();
        let e1 = builder.append(sample_fields("gate_passed")).unwrap();
        let e2 = builder.append(sample_fields("job_completed")).unwrap();

        assert_eq!(e0.sequence, 0);
        assert_eq!(e1.previous_entry_hash, e0.entry_hash);
        assert_eq!(e2.previous_entry_hash, e1.entry_hash);

        let chain = vec![e0, e1, e2];
        assert!(verify_chain(&chain).unwrap());
    }

    #[test]
    fn tampered_entry_fails_verification() {
        let mut builder = AuditChainBuilder::new();
        let e0 = builder.append(sample_fields("job_requested")).unwrap();
        let mut e1 = builder.append(sample_fields("gate_passed")).unwrap();

        // Simulate tampering: mutate stored fields without recomputing
        // entry_hash, exactly what an altered row in storage would look
        // like.
        e1.fields.event = "gate_blocked".to_string();

        let chain = vec![e0, e1];
        assert!(!verify_chain(&chain).unwrap());
    }

    #[test]
    fn broken_link_fails_verification() {
        let mut builder = AuditChainBuilder::new();
        let e0 = builder.append(sample_fields("job_requested")).unwrap();
        let mut e1 = builder.append(sample_fields("gate_passed")).unwrap();

        // Simulate splicing: point e1 at the wrong previous hash, keeping
        // e1 internally self-consistent (entry_hash recomputed to match
        // its own fields+previous_entry_hash) — this should still be
        // caught by the linkage check against e0.
        e1.previous_entry_hash = Sha256Hex::genesis();
        e1.entry_hash = compute_entry_hash(&e1.previous_entry_hash, &e1.fields).unwrap();

        let chain = vec![e0, e1];
        assert!(!verify_chain(&chain).unwrap());
    }

    #[test]
    fn empty_chain_verifies_trivially() {
        let chain: Vec<AuditEntry> = vec![];
        assert!(verify_chain(&chain).unwrap());
    }
}
