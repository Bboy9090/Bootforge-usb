//! Device information and mode detection

use serde::{Deserialize, Serialize};

/// Information about a detected USB device
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceInfo {
    /// Vendor ID
    pub vendor_id: u16,

    /// Product ID
    pub product_id: u16,

    /// Manufacturer string (if available)
    pub manufacturer: Option<String>,

    /// Product name string (if available)
    pub product_name: Option<String>,

    /// Serial number (if available)
    pub serial_number: Option<String>,

    /// USB class code
    pub class: u8,

    /// USB subclass code
    pub subclass: u8,

    /// USB protocol code
    pub protocol: u8,

    /// USB version (BCD format)
    pub usb_version: u16,

    /// Device version (BCD format)
    pub device_version: u16,

    /// Bus number
    pub bus_number: u8,

    /// Device address on bus
    pub device_address: u8,

    /// Platform-specific device path
    pub platform_path: Option<String>,

    /// Detected device mode
    pub device_mode: DeviceMode,
}

/// Device operating mode detection
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeviceMode {
    /// Normal operating mode
    Normal,

    /// Device Firmware Update (DFU) mode
    Dfu,

    /// Recovery mode
    Recovery,

    /// Diagnostic mode
    Diagnostic,

    /// Unknown or undetected mode
    Unknown,
}

impl DeviceMode {
    /// Detect device mode from vendor/product IDs and class information
    pub fn detect(vendor_id: u16, product_id: u16, class: u8) -> Self {
        // Apple DFU mode detection
        if vendor_id == 0x05ac && product_id == 0x1227 {
            return DeviceMode::Dfu;
        }

        // Apple Recovery mode detection
        if vendor_id == 0x05ac && product_id == 0x1281 {
            return DeviceMode::Recovery;
        }

        // Generic DFU class detection
        if class == 0xfe {
            return DeviceMode::Dfu;
        }

        // Android fastboot/download mode
        if vendor_id == 0x18d1 && (product_id == 0xd00d || product_id == 0x4ee0) {
            return DeviceMode::Recovery;
        }

        // Samsung download mode
        if vendor_id == 0x04e8 && product_id == 0x685d {
            return DeviceMode::Recovery;
        }

        DeviceMode::Normal
    }

    /// Check if device is in a special mode (non-normal)
    pub fn is_special_mode(&self) -> bool {
        !matches!(self, DeviceMode::Normal | DeviceMode::Unknown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apple_dfu_detection() {
        let mode = DeviceMode::detect(0x05ac, 0x1227, 0);
        assert_eq!(mode, DeviceMode::Dfu);
        assert!(mode.is_special_mode());
    }

    #[test]
    fn test_apple_recovery_detection() {
        let mode = DeviceMode::detect(0x05ac, 0x1281, 0);
        assert_eq!(mode, DeviceMode::Recovery);
        assert!(mode.is_special_mode());
    }

    #[test]
    fn test_normal_mode() {
        let mode = DeviceMode::detect(0x1234, 0x5678, 0);
        assert_eq!(mode, DeviceMode::Normal);
        assert!(!mode.is_special_mode());
    }

    #[test]
    fn test_dfu_class_detection() {
        let mode = DeviceMode::detect(0x0000, 0x0000, 0xfe);
        assert_eq!(mode, DeviceMode::Dfu);
    }
}
