//! USB Descriptor Parsing Module - GOD MODE
//!
//! This module provides comprehensive USB descriptor enumeration including:
//! - Configuration descriptors
//! - Interface descriptors  
//! - Endpoint descriptors (bulk, interrupt, isochronous, control)
//! - USB 3.0+ SuperSpeed descriptors
//! - Binary Object Store (BOS) descriptors
//! - Power Delivery status
//! - Alternate mode detection

use serde::{Deserialize, Serialize};
use std::time::Duration;

pub mod alternate_modes;
pub mod bos;
pub mod configuration;
pub mod endpoint;
pub mod interface;
pub mod power_delivery;
pub mod superspeed;

// Re-exports for convenience
pub use alternate_modes::*;
pub use bos::*;
pub use configuration::*;
pub use endpoint::*;
pub use interface::*;
pub use power_delivery::*;
pub use superspeed::*;

/// Default timeout for descriptor operations
pub const DESCRIPTOR_TIMEOUT: Duration = Duration::from_secs(1);

/// USB Descriptor Types as per USB specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum DescriptorType {
    Device = 0x01,
    Configuration = 0x02,
    String = 0x03,
    Interface = 0x04,
    Endpoint = 0x05,
    DeviceQualifier = 0x06,
    OtherSpeedConfiguration = 0x07,
    InterfacePower = 0x08,
    Otg = 0x09,
    Debug = 0x0A,
    InterfaceAssociation = 0x0B,
    Bos = 0x0F,
    DeviceCapability = 0x10,
    SuperSpeedEndpointCompanion = 0x30,
    SuperSpeedPlusIsochEndpointCompanion = 0x31,
    Unknown(u8),
}

impl From<u8> for DescriptorType {
    fn from(value: u8) -> Self {
        match value {
            0x01 => DescriptorType::Device,
            0x02 => DescriptorType::Configuration,
            0x03 => DescriptorType::String,
            0x04 => DescriptorType::Interface,
            0x05 => DescriptorType::Endpoint,
            0x06 => DescriptorType::DeviceQualifier,
            0x07 => DescriptorType::OtherSpeedConfiguration,
            0x08 => DescriptorType::InterfacePower,
            0x09 => DescriptorType::Otg,
            0x0A => DescriptorType::Debug,
            0x0B => DescriptorType::InterfaceAssociation,
            0x0F => DescriptorType::Bos,
            0x10 => DescriptorType::DeviceCapability,
            0x30 => DescriptorType::SuperSpeedEndpointCompanion,
            0x31 => DescriptorType::SuperSpeedPlusIsochEndpointCompanion,
            other => DescriptorType::Unknown(other),
        }
    }
}

/// USB Device Class Codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UsbClass {
    /// Use class information in Interface Descriptors
    PerInterface,
    /// Audio
    Audio,
    /// Communications and CDC Control
    CdcControl,
    /// Human Interface Device
    Hid,
    /// Physical
    Physical,
    /// Image/Still Image Capture
    Image,
    /// Printer
    Printer,
    /// Mass Storage
    MassStorage,
    /// Hub
    Hub,
    /// CDC-Data
    CdcData,
    /// Smart Card
    SmartCard,
    /// Content Security
    ContentSecurity,
    /// Video
    Video,
    /// Personal Healthcare
    PersonalHealthcare,
    /// Audio/Video Devices
    AudioVideo,
    /// Billboard Device Class
    Billboard,
    /// USB Type-C Bridge Class
    TypeCBridge,
    /// Diagnostic Device
    Diagnostic,
    /// Wireless Controller
    WirelessController,
    /// Miscellaneous
    Miscellaneous,
    /// Application Specific
    ApplicationSpecific,
    /// Vendor Specific
    VendorSpecific,
    /// Unknown class
    Unknown(u8),
}

impl From<u8> for UsbClass {
    fn from(value: u8) -> Self {
        match value {
            0x00 => UsbClass::PerInterface,
            0x01 => UsbClass::Audio,
            0x02 => UsbClass::CdcControl,
            0x03 => UsbClass::Hid,
            0x05 => UsbClass::Physical,
            0x06 => UsbClass::Image,
            0x07 => UsbClass::Printer,
            0x08 => UsbClass::MassStorage,
            0x09 => UsbClass::Hub,
            0x0A => UsbClass::CdcData,
            0x0B => UsbClass::SmartCard,
            0x0D => UsbClass::ContentSecurity,
            0x0E => UsbClass::Video,
            0x0F => UsbClass::PersonalHealthcare,
            0x10 => UsbClass::AudioVideo,
            0x11 => UsbClass::Billboard,
            0x12 => UsbClass::TypeCBridge,
            0xDC => UsbClass::Diagnostic,
            0xE0 => UsbClass::WirelessController,
            0xEF => UsbClass::Miscellaneous,
            0xFE => UsbClass::ApplicationSpecific,
            0xFF => UsbClass::VendorSpecific,
            other => UsbClass::Unknown(other),
        }
    }
}

impl UsbClass {
    /// Get human-readable name for the USB class
    pub fn name(&self) -> &'static str {
        match self {
            UsbClass::PerInterface => "Defined at Interface level",
            UsbClass::Audio => "Audio",
            UsbClass::CdcControl => "Communications (CDC Control)",
            UsbClass::Hid => "Human Interface Device (HID)",
            UsbClass::Physical => "Physical",
            UsbClass::Image => "Image/Still Image Capture",
            UsbClass::Printer => "Printer",
            UsbClass::MassStorage => "Mass Storage",
            UsbClass::Hub => "Hub",
            UsbClass::CdcData => "CDC-Data",
            UsbClass::SmartCard => "Smart Card",
            UsbClass::ContentSecurity => "Content Security",
            UsbClass::Video => "Video",
            UsbClass::PersonalHealthcare => "Personal Healthcare",
            UsbClass::AudioVideo => "Audio/Video",
            UsbClass::Billboard => "Billboard",
            UsbClass::TypeCBridge => "USB Type-C Bridge",
            UsbClass::Diagnostic => "Diagnostic Device",
            UsbClass::WirelessController => "Wireless Controller",
            UsbClass::Miscellaneous => "Miscellaneous",
            UsbClass::ApplicationSpecific => "Application Specific",
            UsbClass::VendorSpecific => "Vendor Specific",
            UsbClass::Unknown(_) => "Unknown",
        }
    }
}

/// Complete USB device descriptor information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullDeviceDescriptor {
    /// USB specification version (BCD)
    pub usb_version: u16,
    /// Device class code
    pub device_class: UsbClass,
    /// Device subclass code
    pub device_subclass: u8,
    /// Device protocol code
    pub device_protocol: u8,
    /// Maximum packet size for endpoint 0
    pub max_packet_size_ep0: u8,
    /// Vendor ID
    pub vendor_id: u16,
    /// Product ID
    pub product_id: u16,
    /// Device release number (BCD)
    pub device_version: u16,
    /// Manufacturer string
    pub manufacturer: Option<String>,
    /// Product string
    pub product: Option<String>,
    /// Serial number string
    pub serial_number: Option<String>,
    /// Number of configurations
    pub num_configurations: u8,
    /// All configuration descriptors
    pub configurations: Vec<ConfigurationDescriptor>,
    /// Binary Object Store descriptor (USB 2.1+)
    pub bos: Option<BosDescriptor>,
    /// USB speed capability
    pub speed: UsbSpeed,
}

/// USB device speed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UsbSpeed {
    /// Low Speed (1.5 Mbps)
    Low,
    /// Full Speed (12 Mbps)
    Full,
    /// High Speed (480 Mbps) - USB 2.0
    High,
    /// SuperSpeed (5 Gbps) - USB 3.0
    Super,
    /// SuperSpeed+ (10 Gbps) - USB 3.1 Gen 2
    SuperPlus,
    /// SuperSpeed+ (20 Gbps) - USB 3.2 Gen 2x2
    SuperPlus20,
    /// USB4 (40 Gbps)
    Usb4Gen3,
    /// Unknown speed
    Unknown,
}

impl UsbSpeed {
    /// Get the theoretical maximum bandwidth in Mbps
    pub fn bandwidth_mbps(&self) -> u32 {
        match self {
            UsbSpeed::Low => 1,
            UsbSpeed::Full => 12,
            UsbSpeed::High => 480,
            UsbSpeed::Super => 5000,
            UsbSpeed::SuperPlus => 10000,
            UsbSpeed::SuperPlus20 => 20000,
            UsbSpeed::Usb4Gen3 => 40000,
            UsbSpeed::Unknown => 0,
        }
    }

    /// Get human-readable speed name
    pub fn name(&self) -> &'static str {
        match self {
            UsbSpeed::Low => "Low Speed (USB 1.0)",
            UsbSpeed::Full => "Full Speed (USB 1.1)",
            UsbSpeed::High => "High Speed (USB 2.0)",
            UsbSpeed::Super => "SuperSpeed (USB 3.0)",
            UsbSpeed::SuperPlus => "SuperSpeed+ 10Gbps (USB 3.1)",
            UsbSpeed::SuperPlus20 => "SuperSpeed+ 20Gbps (USB 3.2)",
            UsbSpeed::Usb4Gen3 => "USB4 40Gbps",
            UsbSpeed::Unknown => "Unknown",
        }
    }
}

/// Parse all descriptors for a USB device
pub fn parse_device_descriptors(
    device: &rusb::Device<rusb::Context>,
) -> Result<FullDeviceDescriptor, crate::errors::UsbError> {
    let device_desc = device.device_descriptor()?;
    let handle = device.open().ok();
    
    let timeout = DESCRIPTOR_TIMEOUT;
    
    // Try to read string descriptors
    let (manufacturer, product, serial_number) = if let Some(ref h) = handle {
        let langs = h.read_languages(timeout).unwrap_or_default();
        let lang = langs.first().copied();
        
        let manufacturer = lang.and_then(|l| {
            h.read_manufacturer_string(l, &device_desc, timeout).ok()
        });
        let product = lang.and_then(|l| {
            h.read_product_string(l, &device_desc, timeout).ok()
        });
        let serial = lang.and_then(|l| {
            h.read_serial_number_string(l, &device_desc, timeout).ok()
        });
        
        (manufacturer, product, serial)
    } else {
        (None, None, None)
    };
    
    // Parse all configurations
    let mut configurations = Vec::new();
    for config_idx in 0..device_desc.num_configurations() {
        if let Ok(config) = device.config_descriptor(config_idx) {
            let config_desc = configuration::parse_configuration(&config, handle.as_ref());
            configurations.push(config_desc);
        }
    }
    
    // Try to read BOS descriptor (USB 2.1+)
    let bos = if device_desc.usb_version() >= rusb::Version(2, 1, 0) {
        handle.as_ref().and_then(|h| bos::read_bos_descriptor(h).ok())
    } else {
        None
    };
    
    // Determine device speed
    let speed = match device.speed() {
        rusb::Speed::Low => UsbSpeed::Low,
        rusb::Speed::Full => UsbSpeed::Full,
        rusb::Speed::High => UsbSpeed::High,
        rusb::Speed::Super => UsbSpeed::Super,
        rusb::Speed::SuperPlus => UsbSpeed::SuperPlus,
        _ => UsbSpeed::Unknown,
    };
    
    Ok(FullDeviceDescriptor {
        usb_version: (device_desc.usb_version().major() as u16) << 8
            | (device_desc.usb_version().minor() as u16) << 4
            | device_desc.usb_version().sub_minor() as u16,
        device_class: UsbClass::from(device_desc.class_code()),
        device_subclass: device_desc.sub_class_code(),
        device_protocol: device_desc.protocol_code(),
        max_packet_size_ep0: device_desc.max_packet_size(),
        vendor_id: device_desc.vendor_id(),
        product_id: device_desc.product_id(),
        device_version: (device_desc.device_version().major() as u16) << 8
            | (device_desc.device_version().minor() as u16) << 4
            | device_desc.device_version().sub_minor() as u16,
        manufacturer,
        product,
        serial_number,
        num_configurations: device_desc.num_configurations(),
        configurations,
        bos,
        speed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usb_class_names() {
        assert_eq!(UsbClass::Hid.name(), "Human Interface Device (HID)");
        assert_eq!(UsbClass::MassStorage.name(), "Mass Storage");
        assert_eq!(UsbClass::Hub.name(), "Hub");
    }

    #[test]
    fn test_usb_speed_bandwidth() {
        assert_eq!(UsbSpeed::High.bandwidth_mbps(), 480);
        assert_eq!(UsbSpeed::Super.bandwidth_mbps(), 5000);
        assert_eq!(UsbSpeed::SuperPlus.bandwidth_mbps(), 10000);
    }

    #[test]
    fn test_descriptor_type_conversion() {
        assert_eq!(DescriptorType::from(0x01), DescriptorType::Device);
        assert_eq!(DescriptorType::from(0x04), DescriptorType::Interface);
        assert_eq!(DescriptorType::from(0x05), DescriptorType::Endpoint);
    }
}
