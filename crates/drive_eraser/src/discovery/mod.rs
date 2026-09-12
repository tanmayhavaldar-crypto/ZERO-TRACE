pub mod mock;

#[cfg(windows)]
pub mod windows;

use crate::error::Result;
use crate::model::DiscoveredDrive;

/// Trait defining the read-only discovery engine interface.
pub trait DriveDiscoveryProvider: Send + Sync {
    /// Discovers all accessible physical storage drives in the current system.
    fn discover_drives(&self) -> Result<Vec<DiscoveredDrive>>;

    /// Queries the safety status of a specific drive by its physical index.
    fn get_drive_safety(&self, physical_index: u32) -> Result<crate::model::SafetyFlags>;
}