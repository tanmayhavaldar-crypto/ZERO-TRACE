use crate::error::{DriveEraserError, Result};
use sha2::{Digest, Sha256};

/// Indicates the source and stability guarantee of a resolved drive identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentitySource {
    /// Extracted directly from hardware-reported serial number or hardware descriptor.
    /// High confidence of device uniqueness.
    HardwareSerial,
    /// OS-assigned physical device path (e.g., \\.\PhysicalDriveN).
    /// Stable across the current OS session, but NOT guaranteed across hardware re-enumeration or reboot.
    OsDevicePathFallback,
    /// Deterministic pseudo-fingerprint derived from static metadata (vendor, model, byte capacity).
    /// NOT globally unique: two identical drive models of identical capacity will yield the same hash.
    PropertyFingerprintFallback,
}

/// A structured drive identifier documenting both the resolved ID string and its provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDriveIdentity {
    pub id: String,
    pub source: IdentitySource,
}

/// Resolves a drive identity string following a strict hierarchy.
///
/// Priority order:
/// 1. Hardware-reported Serial Number: Preferred. Considered persistent and device-unique.
/// 2. OS Device Path (`\\.\PhysicalDriveN`): Session-stable fallback when hardware serial is absent.
///    Caveat: Not a persistent hardware identifier; enumeration indices can shift across boots.
/// 3. Static Property Hash (SHA-256 over vendor + model + byte capacity):
///    Deterministic fallback.
///    LIMITATION: Not globally unique. Two drives of the exact same make, model, and capacity
///    will produce identical hash values. Drive letters or temporary GUI mount points are strictly excluded.
pub fn resolve_drive_identity(
    serial_number: Option<&str>,
    device_path: &str,
    vendor: Option<&str>,
    model: Option<&str>,
    size_in_bytes: u64,
) -> Result<ResolvedDriveIdentity> {
    if let Some(sn) = serial_number {
        let cleaned = sn.trim();
        // Hardware serials must be non-empty and composed of valid graphic ASCII characters.
        if !cleaned.is_empty() && cleaned.chars().all(|c| c.is_ascii_graphic()) {
            return Ok(ResolvedDriveIdentity {
                id: format!("SN-{}", cleaned),
                source: IdentitySource::HardwareSerial,
            });
        }
    }

    let cleaned_path = device_path.trim();
    if !cleaned_path.is_empty() {
        return Ok(ResolvedDriveIdentity {
            id: format!("PATH-{}", cleaned_path),
            source: IdentitySource::OsDevicePathFallback,
        });
    }

    if size_in_bytes == 0 && vendor.is_none() && model.is_none() {
        return Err(DriveEraserError::IdentityResolutionFailed(
            "Insufficient device attributes to compute fallback drive identity".to_string(),
        ));
    }

    let mut hasher = Sha256::new();
    hasher.update(vendor.unwrap_or("UNKNOWN_VENDOR").trim().as_bytes());
    hasher.update(b"|");
    hasher.update(model.unwrap_or("UNKNOWN_MODEL").trim().as_bytes());
    hasher.update(b"|");
    hasher.update(size_in_bytes.to_le_bytes());

    let hash_result = hasher.finalize();
    Ok(ResolvedDriveIdentity {
        id: format!("HASH-{}", hex::encode(&hash_result[..16])),
        source: IdentitySource::PropertyFingerprintFallback,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serial_number_priority() {
        let resolved = resolve_drive_identity(
            Some("WD-WCC4N7LXYZ12"),
            r"\\.\PhysicalDrive0",
            Some("Western Digital"),
            Some("WD10JPVX"),
            1000204886016,
        )
        .unwrap();

        assert_eq!(resolved.id, "SN-WD-WCC4N7LXYZ12");
        assert_eq!(resolved.source, IdentitySource::HardwareSerial);
    }

    #[test]
    fn test_device_path_fallback_when_serial_missing() {
        let resolved = resolve_drive_identity(
            None,
            r"\\.\PhysicalDrive2",
            Some("Generic"),
            Some("Flash Disk"),
            31457280,
        )
        .unwrap();

        assert_eq!(resolved.id, r"PATH-\\.\PhysicalDrive2");
        assert_eq!(resolved.source, IdentitySource::OsDevicePathFallback);
    }

    #[test]
    fn test_deterministic_property_hash_fallback_not_globally_unique() {
        // Two separate drives with identical specs but missing serials yield identical hashes
        let res1 =
            resolve_drive_identity(None, "", Some("Samsung"), Some("SSD 980"), 500107862016)
                .unwrap();
        let res2 =
            resolve_drive_identity(None, "", Some("Samsung"), Some("SSD 980"), 500107862016)
                .unwrap();

        assert!(res1.id.starts_with("HASH-"));
        assert_eq!(res1.id, res2.id);
        assert_eq!(res1.source, IdentitySource::PropertyFingerprintFallback);
    }

    #[test]
    fn test_ignores_blank_serial_and_falls_back_to_path() {
        let resolved = resolve_drive_identity(
            Some("   "),
            r"\\.\PhysicalDrive1",
            Some("Crucial"),
            Some("CT500MX500SSD1"),
            500107862016,
        )
        .unwrap();

        assert_eq!(resolved.id, r"PATH-\\.\PhysicalDrive1");
        assert_eq!(resolved.source, IdentitySource::OsDevicePathFallback);
    }
}