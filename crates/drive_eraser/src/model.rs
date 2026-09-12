use serde::{Deserialize, Serialize};

/// Represents the physical bus protocol through which the drive is attached.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BusType {
    Unknown,
    Scsi,
    Atapi,
    Ata,
    OneThreeNineFour,
    Ssa,
    Fibre,
    Usb,
    Raid,
    Iscsi,
    Sas,
    Sata,
    Sd,
    Mmc,
    Virtual,
    FileBackedVirtual,
    Spaces,
    Nvme,
    Scm,
    Ufs,
}

impl Default for BusType {
    fn default() -> Self {
        BusType::Unknown
    }
}

/// Physical and operational media type of the underlying storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MediaType {
    Unknown,
    Hdd,
    Ssd,
    Scm,
    RemovableMedia,
}

impl Default for MediaType {
    fn default() -> Self {
        MediaType::Unknown
    }
}

/// High-level device classification derived from verified hardware properties.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DriveClassification {
    Unknown,
    Hdd,
    Ssd,
    NvmeSsd,
    SataSsd,
    RemovableStorage,
}

impl Default for DriveClassification {
    fn default() -> Self {
        DriveClassification::Unknown
    }
}

/// Verification state of drive capabilities or safety attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TriState {
    Unknown,
    Supported,
    Unsupported,
}

impl Default for TriState {
    fn default() -> Self {
        TriState::Unknown
    }
}

/// Capability representation strictly adhering to read-only non-inferred evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DriveCapabilities {
    pub ata_secure_erase: TriState,
    pub ata_enhanced_secure_erase: TriState,
    pub ata_sanitize_block: TriState,
    pub ata_sanitize_crypto: TriState,
    pub ata_sanitize_overwrite: TriState,
    pub nvme_format: TriState,
    pub nvme_format_crypto: TriState,
    pub nvme_sanitize_block: TriState,
    pub nvme_sanitize_crypto: TriState,
    pub nvme_sanitize_overwrite: TriState,
    pub scsi_sanitize: TriState,
}

/// Conservative safety flags evaluated during drive discovery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SafetyFlags {
    /// True if the drive houses the active Windows directory or system partition.
    pub is_system: TriState,
    /// True if the drive houses the active boot loader partition.
    pub is_boot: TriState,
    /// Indicates whether the media is removable (e.g., USB drive).
    pub is_removable: TriState,
    /// Indicates if the drive is marked hardware or software read-only.
    pub is_read_only: TriState,
    /// True if the safety state could not be reliably determined and must be treated cautiously.
    pub requires_conservatism: bool,
}

/// Fully populated, read-only descriptor for a discovered storage device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveredDrive {
    pub id: String,
    pub physical_index: u32,
    pub device_path: String,
    pub vendor: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub size_in_bytes: u64,
    pub sector_size: Option<u32>,
    pub bus_type: BusType,
    pub media_type: MediaType,
    pub classification: DriveClassification,
    pub capabilities: DriveCapabilities,
    pub safety: SafetyFlags,
    pub is_read_only: bool,
}