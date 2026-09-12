use crate::discovery::DriveDiscoveryProvider;
use crate::error::Result;
use crate::model::{
    BusType, DiscoveredDrive, DriveCapabilities, DriveClassification, MediaType, SafetyFlags,
    TriState,
};

/// Deterministic, non-hardware mock discovery provider for testing and validation.
pub struct MockDriveDiscoveryProvider {
    drives: Vec<DiscoveredDrive>,
}

impl MockDriveDiscoveryProvider {
    pub fn new() -> Self {
        let drives = vec![
            // 1. System NVMe SSD (PhysicalDrive0, but system flag is based on verified layout, not index)
            DiscoveredDrive {
                id: "SN-S46VNF0MA12345".to_string(),
                physical_index: 0,
                device_path: r"\\.\PhysicalDrive0".to_string(),
                vendor: Some("Samsung".to_string()),
                model: Some("SSD 980 PRO 1TB".to_string()),
                serial_number: Some("S46VNF0MA12345".to_string()),
                size_in_bytes: 1_000_204_886_016,
                sector_size: Some(512),
                bus_type: BusType::Nvme,
                media_type: MediaType::Ssd,
                classification: DriveClassification::NvmeSsd,
                capabilities: DriveCapabilities {
                    ata_secure_erase: TriState::Unsupported,
                    ata_enhanced_secure_erase: TriState::Unsupported,
                    ata_sanitize_block: TriState::Unsupported,
                    ata_sanitize_crypto: TriState::Unsupported,
                    ata_sanitize_overwrite: TriState::Unsupported,
                    nvme_format: TriState::Unknown,
                    nvme_format_crypto: TriState::Unknown,
                    nvme_sanitize_block: TriState::Unknown,
                    nvme_sanitize_crypto: TriState::Unknown,
                    nvme_sanitize_overwrite: TriState::Unknown,
                    scsi_sanitize: TriState::Unsupported,
                },
                safety: SafetyFlags {
                    is_system: TriState::Supported, // Verified system drive
                    is_boot: TriState::Supported,   // Verified boot loader drive
                    is_removable: TriState::Unsupported,
                    is_read_only: TriState::Unsupported,
                    requires_conservatism: true,
                },
                is_read_only: false,
            },
            // 2. Secondary Internal SATA SSD (PhysicalDrive1)
            DiscoveredDrive {
                id: "SN-CT500MX500SSD1-1823".to_string(),
                physical_index: 1,
                device_path: r"\\.\PhysicalDrive1".to_string(),
                vendor: Some("Crucial".to_string()),
                model: Some("CT500MX500SSD1".to_string()),
                serial_number: Some("1823E1657A90".to_string()),
                size_in_bytes: 500_107_862_016,
                sector_size: Some(512),
                bus_type: BusType::Sata,
                media_type: MediaType::Ssd,
                classification: DriveClassification::SataSsd,
                capabilities: DriveCapabilities {
                    ata_secure_erase: TriState::Unknown,
                    ata_enhanced_secure_erase: TriState::Unknown,
                    ata_sanitize_block: TriState::Unknown,
                    ata_sanitize_crypto: TriState::Unknown,
                    ata_sanitize_overwrite: TriState::Unknown,
                    nvme_format: TriState::Unsupported,
                    nvme_format_crypto: TriState::Unsupported,
                    nvme_sanitize_block: TriState::Unsupported,
                    nvme_sanitize_crypto: TriState::Unsupported,
                    nvme_sanitize_overwrite: TriState::Unsupported,
                    scsi_sanitize: TriState::Unsupported,
                },
                safety: SafetyFlags {
                    is_system: TriState::Unsupported,
                    is_boot: TriState::Unsupported,
                    is_removable: TriState::Unsupported,
                    is_read_only: TriState::Unsupported,
                    requires_conservatism: false,
                },
                is_read_only: false,
            },
            // 3. Secondary Internal SATA HDD (PhysicalDrive2)
            DiscoveredDrive {
                id: "SN-WD-WCC4N7L99182".to_string(),
                physical_index: 2,
                device_path: r"\\.\PhysicalDrive2".to_string(),
                vendor: Some("Western Digital".to_string()),
                model: Some("WD20EZAZ".to_string()),
                serial_number: Some("WD-WCC4N7L99182".to_string()),
                size_in_bytes: 2_000_398_934_016,
                sector_size: Some(4096),
                bus_type: BusType::Sata,
                media_type: MediaType::Hdd,
                classification: DriveClassification::Hdd,
                capabilities: DriveCapabilities {
                    ata_secure_erase: TriState::Unknown,
                    ata_enhanced_secure_erase: TriState::Unknown,
                    ata_sanitize_block: TriState::Unknown,
                    ata_sanitize_crypto: TriState::Unknown,
                    ata_sanitize_overwrite: TriState::Unknown,
                    nvme_format: TriState::Unsupported,
                    nvme_format_crypto: TriState::Unsupported,
                    nvme_sanitize_block: TriState::Unsupported,
                    nvme_sanitize_crypto: TriState::Unsupported,
                    nvme_sanitize_overwrite: TriState::Unsupported,
                    scsi_sanitize: TriState::Unsupported,
                },
                safety: SafetyFlags {
                    is_system: TriState::Unsupported,
                    is_boot: TriState::Unsupported,
                    is_removable: TriState::Unsupported,
                    is_read_only: TriState::Unsupported,
                    requires_conservatism: false,
                },
                is_read_only: false,
            },
            // 4. Removable USB Flash Drive (PhysicalDrive3)
            DiscoveredDrive {
                id: "SN-07082518420083".to_string(),
                physical_index: 3,
                device_path: r"\\.\PhysicalDrive3".to_string(),
                vendor: Some("SanDisk".to_string()),
                model: Some("Ultra USB 3.0".to_string()),
                serial_number: Some("07082518420083".to_string()),
                size_in_bytes: 32_000_000_000,
                sector_size: Some(512),
                bus_type: BusType::Usb,
                media_type: MediaType::RemovableMedia,
                classification: DriveClassification::RemovableStorage,
                capabilities: DriveCapabilities {
                    ata_secure_erase: TriState::Unsupported,
                    ata_enhanced_secure_erase: TriState::Unsupported,
                    ata_sanitize_block: TriState::Unsupported,
                    ata_sanitize_crypto: TriState::Unsupported,
                    ata_sanitize_overwrite: TriState::Unsupported,
                    nvme_format: TriState::Unsupported,
                    nvme_format_crypto: TriState::Unsupported,
                    nvme_sanitize_block: TriState::Unsupported,
                    nvme_sanitize_crypto: TriState::Unsupported,
                    nvme_sanitize_overwrite: TriState::Unsupported,
                    scsi_sanitize: TriState::Unsupported,
                },
                safety: SafetyFlags {
                    is_system: TriState::Unsupported,
                    is_boot: TriState::Unsupported,
                    is_removable: TriState::Supported,
                    is_read_only: TriState::Unsupported,
                    requires_conservatism: false,
                },
                is_read_only: false,
            },
        ];

        Self { drives }
    }
}

impl Default for MockDriveDiscoveryProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl DriveDiscoveryProvider for MockDriveDiscoveryProvider {
    fn discover_drives(&self) -> Result<Vec<DiscoveredDrive>> {
        Ok(self.drives.clone())
    }

    fn get_drive_safety(&self, physical_index: u32) -> Result<SafetyFlags> {
        let drive = self
            .drives
            .iter()
            .find(|d| d.physical_index == physical_index);

        match drive {
            Some(d) => Ok(d.safety.clone()),
            None => Ok(SafetyFlags {
                is_system: TriState::Unknown,
                is_boot: TriState::Unknown,
                is_removable: TriState::Unknown,
                is_read_only: TriState::Unknown,
                requires_conservatism: true,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_discovery_is_deterministic() {
        let provider1 = MockDriveDiscoveryProvider::new();
        let provider2 = MockDriveDiscoveryProvider::new();

        let drives1 = provider1.discover_drives().unwrap();
        let drives2 = provider2.discover_drives().unwrap();

        assert_eq!(drives1, drives2);
        assert_eq!(drives1.len(), 4);
    }

    #[test]
    fn test_mock_safety_consistency() {
        let provider = MockDriveDiscoveryProvider::new();
        let drives = provider.discover_drives().unwrap();

        let system_drive = drives.iter().find(|d| d.physical_index == 0).unwrap();
        assert_eq!(system_drive.safety.is_system, TriState::Supported);
        assert_eq!(system_drive.safety.is_boot, TriState::Supported);
        assert!(system_drive.safety.requires_conservatism);

        let removable_drive = drives.iter().find(|d| d.physical_index == 3).unwrap();
        assert_eq!(removable_drive.safety.is_removable, TriState::Supported);
        assert_eq!(
            removable_drive.classification,
            DriveClassification::RemovableStorage
        );
    }
}