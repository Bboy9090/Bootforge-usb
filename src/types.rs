use crate::model::{DriverStatus, LinkHealth};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a USB device with all available information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbDeviceInfo {
    /// USB vendor ID
    pub vendor_id: u16,
    /// USB product ID
    pub product_id: u16,
    /// Manufacturer string (if available)
    pub manufacturer: Option<String>,
    /// Product name string (if available)
    pub product: Option<String>,
    /// Serial number string (if available)
    pub serial_number: Option<String>,
    /// USB bus number
    pub bus_number: u8,
    /// USB device address on the bus
    pub device_address: u8,
    /// USB device class code
    pub class: u8,
    /// USB device subclass code
    pub subclass: u8,
    /// USB device protocol code
    pub protocol: u8,
    /// USB specification version (BCD format)
    pub usb_version: u16,
    /// Device version (BCD format)
    pub device_version: u16,
    /// Number of configurations
    pub num_configurations: u8,
    /// Platform-specific hints about the device
    pub platform_hint: PlatformHint,
    /// Type of USB bus
    pub bus_type: UsbBusType,
    /// Additional USB IDs information
    pub usb_ids: Option<UsbIds>,
    /// Driver status information
    pub driver_status: DriverStatus,
    /// Link health status
    pub link_health: LinkHealth,
    /// Device tags for classification
    pub tags: Vec<String>,
    /// USB port path/topology
    pub port_path: Option<String>,
}

impl UsbDeviceInfo {
    /// Create a new UsbDeviceInfo with default values
    pub fn new(vendor_id: u16, product_id: u16) -> Self {
        Self {
            vendor_id,
            product_id,
            manufacturer: None,
            product: None,
            serial_number: None,
            bus_number: 0,
            device_address: 0,
            class: 0,
            subclass: 0,
            protocol: 0,
            usb_version: 0,
            device_version: 0,
            num_configurations: 0,
            platform_hint: PlatformHint::default(),
            bus_type: UsbBusType::Unknown,
            usb_ids: None,
            driver_status: DriverStatus::Unknown,
            link_health: LinkHealth::Good,
            tags: Vec::new(),
            port_path: None,
        }
    }
    
    /// Add a tag to the device
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        let tag = tag.into();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
    
    /// Check if device has a specific tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t.eq_ignore_ascii_case(tag))
    }
}

/// Platform-specific information about the USB device
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlatformHint {
    /// Windows-specific device path (if available)
    #[cfg(target_os = "windows")]
    pub device_path: Option<String>,
    
    /// Windows-specific device instance path
    #[cfg(target_os = "windows")]
    pub instance_path: Option<String>,
    
    /// Windows-specific driver name
    #[cfg(target_os = "windows")]
    pub driver_name: Option<String>,
    
    /// Windows-specific hardware IDs
    #[cfg(target_os = "windows")]
    pub hardware_ids: Vec<String>,

    /// Linux-specific sysfs path (if available)
    #[cfg(target_os = "linux")]
    pub sysfs_path: Option<String>,
    
    /// Linux-specific driver name
    #[cfg(target_os = "linux")]
    pub driver: Option<String>,
    
    /// Linux-specific authorization status
    #[cfg(target_os = "linux")]
    pub authorized: Option<bool>,
    
    /// Linux-specific device node (e.g., /dev/bus/usb/001/002)
    #[cfg(target_os = "linux")]
    pub devnode: Option<String>,

    /// macOS-specific IORegistry path (if available)
    #[cfg(target_os = "macos")]
    pub ioregistry_path: Option<String>,
    
    /// macOS-specific location ID
    #[cfg(target_os = "macos")]
    pub location_id: Option<u32>,
    
    /// macOS-specific driver name
    #[cfg(target_os = "macos")]
    pub driver_name: Option<String>,

    /// Generic device path for other platforms
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    pub device_path: Option<String>,
}

/// Type of USB bus connection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UsbBusType {
    /// Standard USB connection
    Standard,
    /// USB over network or virtual USB
    Virtual,
    /// Unknown or could not be determined
    Unknown,
}

/// Extended USB device identification information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbIds {
    /// Human-readable vendor name from USB ID database
    pub vendor_name: Option<String>,
    /// Human-readable product name from USB ID database
    pub product_name: Option<String>,
    /// USB interface class name
    pub class_name: Option<String>,
}

impl fmt::Display for UsbDeviceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "USB Device [{:04x}:{:04x}] ",
            self.vendor_id, self.product_id
        )?;

        if let Some(ref manufacturer) = self.manufacturer {
            write!(f, "{} ", manufacturer)?;
        }

        if let Some(ref product) = self.product {
            write!(f, "{} ", product)?;
        }

        if let Some(ref serial) = self.serial_number {
            write!(f, "(S/N: {}) ", serial)?;
        }

        write!(
            f,
            "Bus {:03} Device {:03}",
            self.bus_number, self.device_address
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usb_device_info_creation() {
        let device = UsbDeviceInfo::new(0x1234, 0x5678);
        assert_eq!(device.vendor_id, 0x1234);
        assert_eq!(device.product_id, 0x5678);
        assert_eq!(device.bus_type, UsbBusType::Unknown);
    }

    #[test]
    fn test_usb_device_info_display() {
        let mut device = UsbDeviceInfo::new(0x1234, 0x5678);
        device.manufacturer = Some("Test Manufacturer".to_string());
        device.product = Some("Test Product".to_string());
        device.bus_number = 1;
        device.device_address = 2;

        let display = format!("{}", device);
        assert!(display.contains("1234:5678"));
        assert!(display.contains("Test Manufacturer"));
        assert!(display.contains("Test Product"));
    }
}
