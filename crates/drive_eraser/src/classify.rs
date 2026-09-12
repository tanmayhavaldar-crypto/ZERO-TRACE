use crate::model::{BusType, DriveClassification, MediaType};

/// Evaluates verified hardware bus and media indicators to conservatively classify a drive.
///
/// Rules:
/// - Removable media is explicitly tagged as `RemovableStorage`.
/// - An NVMe bus interface is architecturally paired with solid-state memory. However, to preserve
///   strict non-inference rules, if the underlying media is positively reported as anything other
///   than SSD or Unknown, it resolves to `Unknown`.
/// - A SATA bus interface is NEVER inferred to be an HDD or SSD without explicit media corroboration.
/// - Uncorroborated, ambiguous, or contradictory evidence defaults to `Unknown`.
pub fn classify_drive(
    bus: BusType,
    media: MediaType,
    is_removable: bool,
) -> DriveClassification {
    if is_removable {
        return DriveClassification::RemovableStorage;
    }

    match (bus, media) {
        (BusType::Nvme, MediaType::Ssd) | (BusType::Nvme, MediaType::Unknown) => {
            DriveClassification::NvmeSsd
        }
        (BusType::Sata, MediaType::Ssd) => DriveClassification::SataSsd,
        (BusType::Sata, MediaType::Hdd) => DriveClassification::Hdd,
        (BusType::Sata, MediaType::Unknown) => DriveClassification::Unknown,
        (_, MediaType::Hdd) => DriveClassification::Hdd,
        (_, MediaType::Ssd) => DriveClassification::Ssd,
        _ => DriveClassification::Unknown,
    }
}

/// Reconciles the physical media type strictly using explicit, non-destructive device property data.
///
/// - Removable status marks media as `RemovableMedia`.
/// - Explicit seek penalty queries (or rotational speed equivalents):
///   - Seek penalty == 0 (rotational rate == 1) indicates Solid State Media (`Ssd`).
///   - Seek penalty != 0 (rotational rate > 1) indicates Rotational Media (`Hdd`).
/// - If no seek penalty or rotational data is verified by the OS, media remains strictly `Unknown`.
///   We do NOT guess media type based on transport bus (e.g. SATA, USB, or SCSI).
pub fn determine_media_type_from_penalty(
    incurs_seek_penalty: Option<bool>,
    is_removable: bool,
) -> MediaType {
    if is_removable {
        return MediaType::RemovableMedia;
    }

    match incurs_seek_penalty {
        Some(false) => MediaType::Ssd,
        Some(true) => MediaType::Hdd,
        None => MediaType::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conservative_sata_handling() {
        // SATA with unknown media MUST NOT be assumed to be HDD
        assert_eq!(
            classify_drive(BusType::Sata, MediaType::Unknown, false),
            DriveClassification::Unknown
        );
        // SATA with confirmed SSD media
        assert_eq!(
            classify_drive(BusType::Sata, MediaType::Ssd, false),
            DriveClassification::SataSsd
        );
        // SATA with confirmed HDD media
        assert_eq!(
            classify_drive(BusType::Sata, MediaType::Hdd, false),
            DriveClassification::Hdd
        );
    }

    #[test]
    fn test_removable_media_classification() {
        // USB with unverified media is RemovableStorage, never guessed as SSD or HDD
        assert_eq!(
            classify_drive(BusType::Usb, MediaType::RemovableMedia, true),
            DriveClassification::RemovableStorage
        );
        assert_eq!(
            classify_drive(BusType::Usb, MediaType::Unknown, true),
            DriveClassification::RemovableStorage
        );
    }

    #[test]
    fn test_seek_penalty_resolution() {
        assert_eq!(
            determine_media_type_from_penalty(Some(false), false),
            MediaType::Ssd
        );
        assert_eq!(
            determine_media_type_from_penalty(Some(true), false),
            MediaType::Hdd
        );
        assert_eq!(
            determine_media_type_from_penalty(None, false),
            MediaType::Unknown
        );
        assert_eq!(
            determine_media_type_from_penalty(None, true),
            MediaType::RemovableMedia
        );
    }
}