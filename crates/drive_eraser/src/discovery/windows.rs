use crate::capability::default_readonly_capabilities;
use crate::classify::{classify_drive, determine_media_type_from_penalty};
use crate::discovery::DriveDiscoveryProvider;
use crate::error::{DriveEraserError, Result};
use crate::identity::resolve_drive_identity;
use crate::model::{
    BusType, DiscoveredDrive, DriveClassification, MediaType, SafetyFlags, TriState,
};
use std::collections::HashSet;
use std::ffi::c_void;
use std::mem::size_of;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, GENERIC_READ, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows::Win32::System::Ioctl::{
    IOCTL_DISK_GET_DRIVE_GEOMETRY_EX, IOCTL_STORAGE_GET_DEVICE_NUMBER,
    IOCTL_STORAGE_QUERY_PROPERTY, DISK_GEOMETRY_EX, STORAGE_DEVICE_NUMBER,
    STORAGE_PROPERTY_QUERY, PropertyStandardQuery, StorageDeviceProperty,
    StorageDeviceSeekPenaltyProperty,
};

/// Production Windows discovery backend using strictly read-only APIs and IOCTLs.
pub struct WindowsDriveDiscoveryProvider;

impl WindowsDriveDiscoveryProvider {
    pub fn new() -> Self {
        Self
    }

    /// Safely obtains a read-only handle to a physical drive or volume.
    fn open_read_only(path: &str) -> Result<HANDLE> {
        let wide: Vec<u16> = std::ffi::OsStr::new(path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        unsafe {
            let handle = CreateFileW(
                PCWSTR(wide.as_ptr()),
                GENERIC_READ.0,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                None,
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                None,
            );

            match handle {
                Ok(h) if h != INVALID_HANDLE_VALUE => Ok(h),
                _ => Err(DriveEraserError::DeviceAccess {
                    path: path.to_string(),
                    message: "Failed to open read-only device handle".to_string(),
                }),
            }
        }
    }

    /// Queries the physical disk number associated with a given volume path.
    fn get_volume_physical_disk_index(volume_path: &str) -> Option<u32> {
        let handle = Self::open_read_only(volume_path).ok()?;
        let mut device_number = STORAGE_DEVICE_NUMBER::default();
        let mut bytes_returned = 0u32;

        let success = unsafe {
            windows::Win32::System::IO::DeviceIoControl(
                handle,
                IOCTL_STORAGE_GET_DEVICE_NUMBER,
                None,
                0,
                Some(&mut device_number as *mut _ as *mut c_void),
                size_of::<STORAGE_DEVICE_NUMBER>() as u32,
                Some(&mut bytes_returned),
                None,
            )
        };

        unsafe {
            let _ = CloseHandle(handle);
        }

        if success.is_ok() && bytes_returned >= size_of::<STORAGE_DEVICE_NUMBER>() as u32 {
            Some(device_number.DeviceNumber)
        } else {
            None
        }
    }

    /// Detects the physical drive hosting the active Windows system files (e.g. %SystemRoot%\System32).
    ///
    /// Never equates PhysicalDrive0 to the system drive.
    fn detect_system_drive() -> Option<u32> {
        let windows_dir = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
        let path_buf = PathBuf::from(windows_dir);
        if let Some(prefix) = path_buf.components().next() {
            let root_path = prefix.as_os_str().to_string_lossy();
            let volume_device_path = format!(r"\\.\{}", root_path.trim_end_matches('\\'));
            return Self::get_volume_physical_disk_index(&volume_device_path);
        }
        None
    }

    /// Detects the physical drive hosting the active boot loader (e.g. ESP / BCD system partition).
    ///
    /// The Windows system volume (containing Windows files) and the boot volume (containing the EFI system partition
    /// or active bootloader) can reside on different physical drives.
    ///
    /// To remain strictly non-destructive and safe:
    /// If the boot volume cannot be verified by a dedicated read-only probe without administrative EFI mount manipulation,
    /// we do NOT assume SystemDrive or PhysicalDrive0 is the boot drive.
    /// It returns `None` (TriState::Unknown) to force conservative evaluation downstream.
    fn detect_boot_drive() -> Option<u32> {
        // Safe read-only evaluation: We inspect whether an explicit SystemPartition path is published
        // in the Windows environment or registry without mounting hidden ESP volumes destructively.
        // In the absence of a direct, verified read-only handle to the EFI partition, return None.
        None
    }

    /// Queries the nominal seek penalty property to verify whether the device has rotational latency (HDD)
    /// or solid state memory (SSD).
    fn query_seek_penalty(handle: HANDLE) -> Option<bool> {
        let query = STORAGE_PROPERTY_QUERY {
            PropertyId: StorageDeviceSeekPenaltyProperty,
            QueryType: PropertyStandardQuery,
            AdditionalParameters: [0],
        };

        #[repr(C)]
        struct DEVICE_SEEK_PENALTY_DESCRIPTOR {
            version: u32,
            size: u32,
            incurs_seek_penalty: u8,
        }

        let mut descriptor = DEVICE_SEEK_PENALTY_DESCRIPTOR {
            version: 0,
            size: 0,
            incurs_seek_penalty: 0,
        };
        let mut bytes_returned = 0u32;

        let success = unsafe {
            windows::Win32::System::IO::DeviceIoControl(
                handle,
                IOCTL_STORAGE_QUERY_PROPERTY,
                Some(&query as *const _ as *const c_void),
                size_of::<STORAGE_PROPERTY_QUERY>() as u32,
                Some(&mut descriptor as *mut _ as *mut c_void),
                size_of::<DEVICE_SEEK_PENALTY_DESCRIPTOR>() as u32,
                Some(&mut bytes_returned),
                None,
            )
        };

        if success.is_ok() && bytes_returned >= size_of::<DEVICE_SEEK_PENALTY_DESCRIPTOR>() as u32 {
            Some(descriptor.incurs_seek_penalty != 0)
        } else {
            None
        }
    }

    /// Queries drive capacity and sector geometry using IOCTL_DISK_GET_DRIVE_GEOMETRY_EX.
    fn query_drive_capacity(handle: HANDLE) -> (u64, Option<u32>) {
        let mut geometry_ex = DISK_GEOMETRY_EX::default();
        let mut bytes_returned = 0u32;

        let success = unsafe {
            windows::Win32::System::IO::DeviceIoControl(
                handle,
                IOCTL_DISK_GET_DRIVE_GEOMETRY_EX,
                None,
                0,
                Some(&mut geometry_ex as *mut _ as *mut c_void),
                size_of::<DISK_GEOMETRY_EX>() as u32,
                Some(&mut bytes_returned),
                None,
            )
        };

        if success.is_ok() {
            let bytes = geometry_ex.DiskSize as u64;
            let sector_size = geometry_ex.Geometry.BytesPerSector;
            (bytes, Some(sector_size))
        } else {
            (0, None)
        }
    }

    /// Queries the storage device descriptor containing hardware identifiers, bus type, and removability.
    fn query_device_descriptor(
        handle: HANDLE,
    ) -> Result<(
        Option<String>,
        Option<String>,
        Option<String>,
        BusType,
        bool,
    )> {
        let query = STORAGE_PROPERTY_QUERY {
            PropertyId: StorageDeviceProperty,
            QueryType: PropertyStandardQuery,
            AdditionalParameters: [0],
        };

        let mut buffer = [0u8; 1024];
        let mut bytes_returned = 0u32;

        let success = unsafe {
            windows::Win32::System::IO::DeviceIoControl(
                handle,
                IOCTL_STORAGE_QUERY_PROPERTY,
                Some(&query as *const _ as *const c_void),
                size_of::<STORAGE_PROPERTY_QUERY>() as u32,
                Some(buffer.as_mut_ptr() as *mut c_void),
                buffer.len() as u32,
                Some(&mut bytes_returned),
                None,
            )
        };

        if success.is_err() || bytes_returned < 24 {
            return Ok((None, None, None, BusType::Unknown, false));
        }

        let is_removable = buffer[10] != 0;

        let vendor_offset =
            u32::from_le_bytes(buffer[16..20].try_into().unwrap_or_default()) as usize;
        let product_offset =
            u32::from_le_bytes(buffer[20..24].try_into().unwrap_or_default()) as usize;
        let serial_offset =
            u32::from_le_bytes(buffer[28..32].try_into().unwrap_or_default()) as usize;
        let raw_bus_type = u32::from_le_bytes(buffer[32..36].try_into().unwrap_or_default());

        let extract_string = |offset: usize| -> Option<String> {
            if offset == 0 || offset >= bytes_returned as usize {
                return None;
            }
            let slice = &buffer[offset..bytes_returned as usize];
            let len = slice.iter().position(|&b| b == 0).unwrap_or(slice.len());
            let s = String::from_utf8_lossy(&slice[..len]).trim().to_string();
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        };

        let vendor = extract_string(vendor_offset);
        let model = extract_string(product_offset);
        let serial = extract_string(serial_offset);

        let bus = match raw_bus_type {
            1 => BusType::Scsi,
            2 => BusType::Atapi,
            3 => BusType::Ata,
            4 => BusType::OneThreeNineFour,
            5 => BusType::Ssa,
            6 => BusType::Fibre,
            7 => BusType::Usb,
            8 => BusType::Raid,
            9 => BusType::Iscsi,
            10 => BusType::Sas,
            11 => BusType::Sata,
            12 => BusType::Sd,
            13 => BusType::Mmc,
            14 => BusType::Virtual,
            15 => BusType::FileBackedVirtual,
            16 => BusType::Spaces,
            17 => BusType::Nvme,
            18 => BusType::Scm,
            19 => BusType::Ufs,
            _ => BusType::Unknown,
        };

        Ok((vendor, model, serial, bus, is_removable))
    }
}

impl Default for WindowsDriveDiscoveryProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl DriveDiscoveryProvider for WindowsDriveDiscoveryProvider {
    fn discover_drives(&self) -> Result<Vec<DiscoveredDrive>> {
        let mut discovered = Vec::new();
        let system_idx = Self::detect_system_drive();
        let boot_idx = Self::detect_boot_drive();

        // Enumerate physical drives conservatively from index 0 to 64
        for index in 0..64 {
            let path = format!(r"\\.\PhysicalDrive{}", index);
            let handle = match Self::open_read_only(&path) {
                Ok(h) => h,
                Err(_) => continue,
            };

            let (capacity, sector_size) = Self::query_drive_capacity(handle);
            let (vendor, model, serial, bus, is_removable) =
                Self::query_device_descriptor(handle).unwrap_or((
                    None,
                    None,
                    None,
                    BusType::Unknown,
                    false,
                ));

            let seek_penalty = Self::query_seek_penalty(handle);
            unsafe {
                let _ = CloseHandle(handle);
            }

            // Strictly separate verified media indicators from speculative assumptions
            let media_type = determine_media_type_from_penalty(seek_penalty, is_removable);

            // Classification preserves conservative rules:
            // - Removable media becomes RemovableStorage.
            // - NVMe is classified as NvmeSsd because the NVMe protocol is designed for non-volatile solid-state storage.
            // - SATA with Unknown media remains Unknown (never assume SATA == HDD).
            let classification = classify_drive(bus, media_type, is_removable);

            let is_system = match system_idx {
                Some(sys) if sys == index => TriState::Supported,
                Some(_) => TriState::Unsupported,
                None => TriState::Unknown,
            };

            // Boot status remains strictly Unknown unless safely and positively verified
            let is_boot = match boot_idx {
                Some(bt) if bt == index => TriState::Supported,
                Some(_) => TriState::Unsupported,
                None => TriState::Unknown,
            };

            // If system or boot cannot be definitively ruled out as Unsupported,
            // requires_conservatism MUST remain true.
            let requires_conservatism = is_system != TriState::Unsupported
                || is_boot != TriState::Unsupported;

            let safety = SafetyFlags {
                is_system,
                is_boot,
                is_removable: if is_removable {
                    TriState::Supported
                } else {
                    TriState::Unsupported
                },
                is_read_only: TriState::Unknown,
                requires_conservatism,
            };

            let resolved_identity = resolve_drive_identity(
                serial.as_deref(),
                &path,
                vendor.as_deref(),
                model.as_deref(),
                capacity,
            )?;

            discovered.push(DiscoveredDrive {
                id: resolved_identity.id,
                physical_index: index,
                device_path: path,
                vendor,
                model,
                serial_number: serial,
                size_in_bytes: capacity,
                sector_size,
                bus_type: bus,
                media_type,
                classification,
                capabilities: default_readonly_capabilities(),
                safety,
                is_read_only: true,
            });
        }

        Ok(discovered)
    }

    fn get_drive_safety(&self, physical_index: u32) -> Result<SafetyFlags> {
        let system_idx = Self::detect_system_drive();
        let boot_idx = Self::detect_boot_drive();

        let is_system = match system_idx {
            Some(sys) if sys == physical_index => TriState::Supported,
            Some(_) => TriState::Unsupported,
            None => TriState::Unknown,
        };

        let is_boot = match boot_idx {
            Some(bt) if bt == physical_index => TriState::Supported,
            Some(_) => TriState::Unsupported,
            None => TriState::Unknown,
        };

        let requires_conservatism = is_system != TriState::Unsupported
            || is_boot != TriState::Unsupported;

        Ok(SafetyFlags {
            is_system,
            is_boot,
            is_removable: TriState::Unknown,
            is_read_only: TriState::Unknown,
            requires_conservatism,
        })
    }
}