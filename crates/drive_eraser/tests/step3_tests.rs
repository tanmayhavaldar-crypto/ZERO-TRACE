use drive_eraser::capability::evaluate_safe_capabilities;
use drive_eraser::classify::{classify_drive, determine_media_type_from_penalty};
use drive_eraser::discovery::mock::MockDriveDiscoveryProvider;
use drive_eraser::discovery::DriveDiscoveryProvider;
use drive_eraser::identity::{resolve_drive_identity, IdentitySource};
use drive_eraser::model::{BusType, DriveClassification, MediaType, TriState};

#[test]
fn test_identity_deterministic_and_no_drive_letters() {
    let res_a = resolve_drive_identity(
        Some("WD-SN750-123456"),
        r"\\.\PhysicalDrive1",
        Some("Western Digital"),
        Some("WDS500G3X0C"),
        500107862016,
    )
    .unwrap();

    let res_b = resolve_drive_identity(
        Some("WD-SN750-123456"),
        r"\\.\PhysicalDrive1",
        Some("Western Digital"),
        Some("WDS500G3X0C"),
        500107862016,
    )
    .unwrap();

    assert_eq!(res_a.id, res_b.id);
    assert_eq!(res_a.id, "SN-WD-SN750-123456");
    assert_eq!(res_a.source, IdentitySource::HardwareSerial);
}

#[test]
fn test_identity_fallback_behavior_and_limitations() {
    // Missing serial falls back to OS device path
    let res_path = resolve_drive_identity(
        None,
        r"\\.\PhysicalDrive5",
        Some("Generic"),
        Some("Storage"),
        64000000000,
    )
    .unwrap();
    assert_eq!(res_path.id, r"PATH-\\.\PhysicalDrive5");
    assert_eq!(res_path.source, IdentitySource::OsDevicePathFallback);

    // Missing serial and empty path falls back to property fingerprint
    let res_hash1 = resolve_drive_identity(
        None,
        "",
        Some("Micron"),
        Some("1100 SATA SSD"),
        256060514304,
    )
    .unwrap();
    let res_hash2 = resolve_drive_identity(
        None,
        "",
        Some("Micron"),
        Some("1100 SATA SSD"),
        256060514304,
    )
    .unwrap();

    assert_eq!(res_hash1.id, res_hash2.id);
    assert_eq!(res_hash1.source, IdentitySource::PropertyFingerprintFallback);
    assert!(res_hash1.id.starts_with("HASH-"));
}

#[test]
fn test_classification_conservatism_no_sata_guessing() {
    // SATA with Unknown media MUST remain Unknown, never assumed to be HDD
    assert_eq!(
        classify_drive(BusType::Sata, MediaType::Unknown, false),
        DriveClassification::Unknown
    );

    // SATA with verified SSD media
    assert_eq!(
        classify_drive(BusType::Sata, MediaType::Ssd, false),
        DriveClassification::SataSsd
    );

    // SATA with verified HDD media
    assert_eq!(
        classify_drive(BusType::Sata, MediaType::Hdd, false),
        DriveClassification::Hdd
    );

    // NVMe with SSD or Unknown media classifies as NvmeSsd
    assert_eq!(
        classify_drive(BusType::Nvme, MediaType::Ssd, false),
        DriveClassification::NvmeSsd
    );
    assert_eq!(
        classify_drive(BusType::Nvme, MediaType::Unknown, false),
        DriveClassification::NvmeSsd
    );

    // Removable media classification takes precedence
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
fn test_media_type_from_seek_penalty_conservatism() {
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

#[test]
fn test_capabilities_remain_strictly_unknown_without_evidence() {
    let caps = evaluate_safe_capabilities(None, None);

    assert_eq!(caps.ata_secure_erase, TriState::Unknown);
    assert_eq!(caps.ata_sanitize_block, TriState::Unknown);
    assert_eq!(caps.nvme_format, TriState::Unknown);
    assert_eq!(caps.nvme_sanitize_crypto, TriState::Unknown);
    assert_eq!(caps.scsi_sanitize, TriState::Unknown);
}

#[test]
fn test_mock_discovery_and_safety_validation() {
    let provider = MockDriveDiscoveryProvider::new();
    let drives = provider.discover_drives().unwrap();

    assert_eq!(drives.len(), 4);

    // Drive 0: Explicitly configured as system drive in mock
    let drive0 = &drives[0];
    assert_eq!(drive0.physical_index, 0);
    assert_eq!(drive0.safety.is_system, TriState::Supported);

    // Drive 1: Non-system internal disk
    let drive1 = &drives[1];
    assert_eq!(drive1.physical_index, 1);
    assert_eq!(drive1.safety.is_system, TriState::Unsupported);
    assert_eq!(drive1.safety.is_boot, TriState::Unsupported);

    // Unknown drive queries enforce requires_conservatism
    let unknown_safety = provider.get_drive_safety(99).unwrap();
    assert_eq!(unknown_safety.is_system, TriState::Unknown);
    assert_eq!(unknown_safety.is_boot, TriState::Unknown);
    assert!(unknown_safety.requires_conservatism);
}