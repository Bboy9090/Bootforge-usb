//! USB device descriptor reading

use crate::{BootforgeError, Result};
use serde::{Deserialize, Serialize};

/// USB device descriptor information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceDescriptor {
    /// USB specification release number in BCD format
    pub usb_version: u16,

    /// Device class code
    pub device_class: u8,

    /// Device subclass code
    pub device_subclass: u8,

    /// Device protocol code
    pub device_protocol: u8,

    /// Maximum packet size for endpoint zero
    pub max_packet_size: u8,

    /// Vendor ID
    pub vendor_id: u16,

    /// Product ID
    pub product_id: u16,

    /// Device release number in BCD format
    pub device_version: u16,

    /// Index of manufacturer string descriptor
    pub manufacturer_index: u8,

    /// Index of product string descriptor
    pub product_index: u8,

    /// Index of serial number string descriptor
    pub serial_number_index: u8,

    /// Number of possible configurations
    pub num_configurations: u8,
}

impl DeviceDescriptor {
    /// Read device descriptor from a USB device
    pub fn read(device: &rusb::Device<rusb::GlobalContext>) -> Result<Self> {
        let device_desc = device.device_descriptor()?;

        // Version is stored as BCD in the descriptor already
        let usb_ver = device_desc.usb_version();
        let dev_ver = device_desc.device_version();

        Ok(DeviceDescriptor {
            usb_version: usb_ver.major() as u16 * 256 + usb_ver.minor() as u16 * 16 + usb_ver.sub_minor() as u16,
            device_class: device_desc.class_code(),
            device_subclass: device_desc.sub_class_code(),
            device_protocol: device_desc.protocol_code(),
            max_packet_size: device_desc.max_packet_size(),
            vendor_id: device_desc.vendor_id(),
            product_id: device_desc.product_id(),
            device_version: dev_ver.major() as u16 * 256 + dev_ver.minor() as u16 * 16 + dev_ver.sub_minor() as u16,
            manufacturer_index: device_desc.manufacturer_string_index().unwrap_or(0),
            product_index: device_desc.product_string_index().unwrap_or(0),
            serial_number_index: device_desc.serial_number_string_index().unwrap_or(0),
            num_configurations: device_desc.num_configurations(),
        })
    }

    /// Read a string descriptor from a device
    pub fn read_string_descriptor(
        handle: &rusb::DeviceHandle<rusb::GlobalContext>,
        index: u8,
        _timeout: std::time::Duration,
    ) -> Result<Option<String>> {
        if index == 0 {
            return Ok(None);
        }

        match handle.read_string_descriptor_ascii(index) {
            Ok(s) => Ok(Some(s)),
            Err(rusb::Error::NotSupported) => Ok(None),
            Err(rusb::Error::Pipe) => Ok(None),
            Err(e) => Err(BootforgeError::UsbError(e)),
        }
    }

    /// Get USB version as a formatted string (e.g., "2.0")
    pub fn usb_version_string(&self) -> String {
        let major = self.usb_version >> 8;
        let minor = (self.usb_version >> 4) & 0x0f;
        format!("{}.{}", major, minor)
    }

    /// Get device version as a formatted string
    pub fn device_version_string(&self) -> String {
        let major = self.device_version >> 8;
        let minor = (self.device_version >> 4) & 0x0f;
        format!("{}.{}", major, minor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usb_version_string() {
        let desc = DeviceDescriptor {
            usb_version: 0x0200, // USB 2.0
            device_class: 0,
            device_subclass: 0,
            device_protocol: 0,
            max_packet_size: 64,
            vendor_id: 0x1234,
            product_id: 0x5678,
            device_version: 0x0110, // 1.1
            manufacturer_index: 1,
            product_index: 2,
            serial_number_index: 3,
            num_configurations: 1,
        };

        assert_eq!(desc.usb_version_string(), "2.0");
        assert_eq!(desc.device_version_string(), "1.1");
    }
}
