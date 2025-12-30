//! Binary Object Store (BOS) Descriptor Parsing
//!
//! The BOS descriptor contains device capability descriptors that describe
//! extended capabilities of the USB device, including:
//! - USB 2.0 Extension (LPM support)
//! - SuperSpeed USB Device Capability
//! - Container ID
//! - Platform-specific capabilities
//! - SuperSpeedPlus capabilities (USB 3.1+)
//! - USB Power Delivery capabilities
//! - Billboard capabilities

use super::superspeed::{SuperSpeedCapability, SuperSpeedPlusCapability};
use super::DESCRIPTOR_TIMEOUT;
use serde::{Deserialize, Serialize};

/// Binary Object Store Descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BosDescriptor {
    /// Total length of the BOS descriptor and all capability descriptors
    pub total_length: u16,
    /// Number of device capability descriptors
    pub num_capabilities: u8,
    /// All device capability descriptors
    pub capabilities: Vec<DeviceCapability>,
}

/// Device Capability Descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceCapability {
    /// Wireless USB specific capability
    WirelessUsb(WirelessUsbCapability),
    /// USB 2.0 Extension (LPM support)
    Usb20Extension(Usb20ExtensionCapability),
    /// SuperSpeed USB Device Capability
    SuperSpeed(SuperSpeedCapability),
    /// Container ID
    ContainerId(ContainerIdCapability),
    /// Platform-specific capability
    Platform(PlatformCapability),
    /// SuperSpeedPlus USB Device Capability
    SuperSpeedPlus(SuperSpeedPlusCapability),
    /// Precision Time Measurement
    PrecisionTimeMeasurement(PrecisionTimeMeasurementCapability),
    /// Wireless USB External Hub Capability
    WirelessExtHub(WirelessExtHubCapability),
    /// Billboard capability
    Billboard(BillboardCapability),
    /// Authentication capability
    Authentication(AuthenticationCapability),
    /// Billboard Extension capability
    BillboardExtension(BillboardExtensionCapability),
    /// Configuration Summary capability
    ConfigurationSummary(ConfigurationSummaryCapability),
    /// Unknown capability type
    Unknown {
        capability_type: u8,
        data: Vec<u8>,
    },
}

/// USB 2.0 Extension Capability (LPM)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usb20ExtensionCapability {
    /// Bitmap of attributes
    pub attributes: u32,
    /// LPM (Link Power Management) supported
    pub lpm_supported: bool,
    /// BESL (Best Effort Service Latency) supported
    pub besl_supported: bool,
    /// Baseline BESL valid
    pub baseline_besl_valid: bool,
    /// Deep BESL valid
    pub deep_besl_valid: bool,
    /// Baseline BESL value
    pub baseline_besl: u8,
    /// Deep BESL value
    pub deep_besl: u8,
}

/// Wireless USB Capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WirelessUsbCapability {
    /// Bitmap of attributes
    pub attributes: u8,
    /// PHY rates supported
    pub phy_rates: u16,
    /// Power level
    pub power_level: u8,
}

/// Container ID Capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerIdCapability {
    /// Container ID (UUID)
    pub container_id: [u8; 16],
    /// Container ID as string
    pub container_id_string: String,
}

/// Platform Capability Descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformCapability {
    /// Platform capability UUID
    pub capability_uuid: [u8; 16],
    /// Capability UUID as string
    pub uuid_string: String,
    /// Platform-specific capability data
    pub capability_data: Vec<u8>,
    /// Parsed WebUSB descriptor (if applicable)
    pub webusb: Option<WebUsbPlatformCapability>,
    /// Parsed Microsoft OS 2.0 descriptor (if applicable)
    pub microsoft_os_20: Option<MicrosoftOs20Capability>,
}

/// WebUSB Platform Capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebUsbPlatformCapability {
    /// WebUSB version (BCD)
    pub version: u16,
    /// Vendor request code for reading landing page URL
    pub vendor_code: u8,
    /// Landing page URL index
    pub landing_page_index: u8,
    /// Landing page URL (if read)
    pub landing_page_url: Option<String>,
}

/// Microsoft OS 2.0 Platform Capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrosoftOs20Capability {
    /// Windows version minimum
    pub windows_version: u32,
    /// Total length of MS OS 2.0 descriptor set
    pub total_length: u16,
    /// Vendor request code
    pub vendor_code: u8,
    /// Alternate enumeration code
    pub alt_enum_code: u8,
}

/// Precision Time Measurement Capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrecisionTimeMeasurementCapability {
    /// Reserved field
    pub reserved: u8,
}

/// Wireless USB External Hub Capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WirelessExtHubCapability {
    /// Number of ports
    pub num_ports: u8,
}

/// Billboard Capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillboardCapability {
    /// Number of alternate modes
    pub num_alternate_modes: u8,
    /// Preferred alternate mode index
    pub preferred_alternate_mode: u8,
    /// VCONN power requirements
    pub vconn_power: VconnPower,
    /// Billboard descriptor version (BCD)
    pub version: u16,
    /// Additional failure info available
    pub additional_failure_info: bool,
    /// Alternate modes
    pub alternate_modes: Vec<BillboardAlternateMode>,
}

/// VCONN Power requirements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VconnPower {
    /// 1W
    Watts1,
    /// 1.5W
    Watts1_5,
    /// 2W
    Watts2,
    /// 3W
    Watts3,
    /// 4W
    Watts4,
    /// 5W
    Watts5,
    /// 6W
    Watts6,
    /// Reserved
    Reserved(u8),
}

/// Billboard Alternate Mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillboardAlternateMode {
    /// Alternate mode SVID (Standard or Vendor ID)
    pub svid: u16,
    /// Alternate mode index
    pub mode_index: u8,
    /// Mode state
    pub state: AlternateModeState,
}

/// Alternate Mode State
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlternateModeState {
    /// Unspecified
    Unspecified,
    /// Alternate mode not attempted
    NotAttempted,
    /// Alternate mode attempted but unsuccessful
    Unsuccessful,
    /// Alternate mode successful
    Successful,
}

/// Authentication Capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationCapability {
    /// Authentication types supported
    pub auth_types: u8,
}

/// Billboard Extension Capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillboardExtensionCapability {
    /// Billboard version
    pub version: u8,
    /// Number of SVIDs
    pub num_svids: u8,
}

/// Configuration Summary Capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationSummaryCapability {
    /// Configuration number
    pub config_number: u8,
    /// Function class
    pub function_class: u8,
    /// Function subclass
    pub function_subclass: u8,
    /// Function protocol
    pub function_protocol: u8,
}

/// Read BOS descriptor from a device
pub fn read_bos_descriptor(
    handle: &rusb::DeviceHandle<rusb::Context>,
) -> Result<BosDescriptor, crate::errors::UsbError> {
    // First, read just the BOS header to get total length
    let mut header = [0u8; 5];
    let request_type = rusb::request_type(
        rusb::Direction::In,
        rusb::RequestType::Standard,
        rusb::Recipient::Device,
    );
    
    let bytes_read = handle.read_control(
        request_type,
        0x06, // GET_DESCRIPTOR
        0x0F00, // BOS descriptor (type 0x0F, index 0)
        0,
        &mut header,
        DESCRIPTOR_TIMEOUT,
    )?;
    
    if bytes_read < 5 {
        return Err(crate::errors::UsbError::Parse(
            "BOS descriptor too short".to_string(),
        ));
    }
    
    let total_length = u16::from_le_bytes([header[2], header[3]]);
    let num_capabilities = header[4];
    
    // Now read the full BOS descriptor
    let mut buffer = vec![0u8; total_length as usize];
    let bytes_read = handle.read_control(
        request_type,
        0x06,
        0x0F00,
        0,
        &mut buffer,
        DESCRIPTOR_TIMEOUT,
    )?;
    
    if bytes_read < total_length as usize {
        buffer.truncate(bytes_read);
    }
    
    // Parse device capabilities
    let capabilities = parse_capabilities(&buffer[5..]);
    
    Ok(BosDescriptor {
        total_length,
        num_capabilities,
        capabilities,
    })
}

/// Parse device capability descriptors
fn parse_capabilities(data: &[u8]) -> Vec<DeviceCapability> {
    let mut capabilities = Vec::new();
    let mut offset = 0;
    
    while offset + 3 < data.len() {
        let length = data[offset] as usize;
        let desc_type = data[offset + 1];
        
        if length < 3 || offset + length > data.len() {
            break;
        }
        
        // Device Capability descriptor type is 0x10
        if desc_type == 0x10 {
            let capability_type = data[offset + 2];
            let cap_data = &data[offset + 3..offset + length];
            
            let capability = parse_capability(capability_type, cap_data);
            capabilities.push(capability);
        }
        
        offset += length;
    }
    
    capabilities
}

/// Parse a single device capability
fn parse_capability(capability_type: u8, data: &[u8]) -> DeviceCapability {
    match capability_type {
        0x01 => parse_wireless_usb_capability(data),
        0x02 => parse_usb20_extension(data),
        0x03 => parse_superspeed_capability(data),
        0x04 => parse_container_id(data),
        0x05 => parse_platform_capability(data),
        0x06 => parse_power_delivery_capability(data),
        0x07 => DeviceCapability::Unknown { capability_type, data: data.to_vec() }, // Battery info
        0x08 => DeviceCapability::Unknown { capability_type, data: data.to_vec() }, // PD consumer port
        0x09 => DeviceCapability::Unknown { capability_type, data: data.to_vec() }, // PD provider port
        0x0A => parse_superspeed_plus_capability(data),
        0x0B => parse_precision_time_measurement(data),
        0x0C => parse_wireless_ext_hub(data),
        0x0D => parse_billboard_capability(data),
        0x0E => parse_authentication_capability(data),
        0x0F => parse_billboard_extension(data),
        0x10 => parse_configuration_summary(data),
        _ => DeviceCapability::Unknown {
            capability_type,
            data: data.to_vec(),
        },
    }
}

fn parse_wireless_usb_capability(data: &[u8]) -> DeviceCapability {
    if data.len() >= 4 {
        DeviceCapability::WirelessUsb(WirelessUsbCapability {
            attributes: data[0],
            phy_rates: u16::from_le_bytes([data[1], data[2]]),
            power_level: data[3],
        })
    } else {
        DeviceCapability::Unknown { capability_type: 0x01, data: data.to_vec() }
    }
}

fn parse_usb20_extension(data: &[u8]) -> DeviceCapability {
    if data.len() >= 4 {
        let attributes = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        DeviceCapability::Usb20Extension(Usb20ExtensionCapability {
            attributes,
            lpm_supported: (attributes & 0x02) != 0,
            besl_supported: (attributes & 0x04) != 0,
            baseline_besl_valid: (attributes & 0x08) != 0,
            deep_besl_valid: (attributes & 0x10) != 0,
            baseline_besl: ((attributes >> 8) & 0x0F) as u8,
            deep_besl: ((attributes >> 12) & 0x0F) as u8,
        })
    } else {
        DeviceCapability::Unknown { capability_type: 0x02, data: data.to_vec() }
    }
}

fn parse_superspeed_capability(data: &[u8]) -> DeviceCapability {
    if data.len() >= 7 {
        DeviceCapability::SuperSpeed(SuperSpeedCapability {
            attributes: data[0],
            lpm_supported: (data[0] & 0x02) != 0,
            speed_supported: u16::from_le_bytes([data[1], data[2]]),
            functionality_support: data[3],
            u1_dev_exit_lat: data[4],
            u2_dev_exit_lat: u16::from_le_bytes([data[5], data[6]]),
        })
    } else {
        DeviceCapability::Unknown { capability_type: 0x03, data: data.to_vec() }
    }
}

fn parse_container_id(data: &[u8]) -> DeviceCapability {
    if data.len() >= 17 {
        let mut container_id = [0u8; 16];
        container_id.copy_from_slice(&data[1..17]);
        
        let container_id_string = format!(
            "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            u32::from_le_bytes([container_id[0], container_id[1], container_id[2], container_id[3]]),
            u16::from_le_bytes([container_id[4], container_id[5]]),
            u16::from_le_bytes([container_id[6], container_id[7]]),
            container_id[8], container_id[9],
            container_id[10], container_id[11], container_id[12], container_id[13], container_id[14], container_id[15]
        );
        
        DeviceCapability::ContainerId(ContainerIdCapability {
            container_id,
            container_id_string,
        })
    } else {
        DeviceCapability::Unknown { capability_type: 0x04, data: data.to_vec() }
    }
}

fn parse_platform_capability(data: &[u8]) -> DeviceCapability {
    if data.len() >= 17 {
        let mut capability_uuid = [0u8; 16];
        capability_uuid.copy_from_slice(&data[1..17]);
        
        let uuid_string = format!(
            "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            u32::from_le_bytes([capability_uuid[0], capability_uuid[1], capability_uuid[2], capability_uuid[3]]),
            u16::from_le_bytes([capability_uuid[4], capability_uuid[5]]),
            u16::from_le_bytes([capability_uuid[6], capability_uuid[7]]),
            capability_uuid[8], capability_uuid[9],
            capability_uuid[10], capability_uuid[11], capability_uuid[12], capability_uuid[13], capability_uuid[14], capability_uuid[15]
        );
        
        let capability_data = data[17..].to_vec();
        
        // Check for WebUSB UUID: 3408b638-09a9-47a0-8bfd-a0768815b665
        let webusb_uuid: [u8; 16] = [0x38, 0xB6, 0x08, 0x34, 0xA9, 0x09, 0xA0, 0x47, 0x8B, 0xFD, 0xA0, 0x76, 0x88, 0x15, 0xB6, 0x65];
        let webusb = if capability_uuid == webusb_uuid && capability_data.len() >= 3 {
            Some(WebUsbPlatformCapability {
                version: u16::from_le_bytes([capability_data[0], capability_data[1]]),
                vendor_code: capability_data[2],
                landing_page_index: if capability_data.len() > 3 { capability_data[3] } else { 0 },
                landing_page_url: None,
            })
        } else {
            None
        };
        
        // Check for Microsoft OS 2.0 UUID: D8DD60DF-4589-4CC7-9CD2-659D9E648A9F
        let ms_os_uuid: [u8; 16] = [0xDF, 0x60, 0xDD, 0xD8, 0x89, 0x45, 0xC7, 0x4C, 0x9C, 0xD2, 0x65, 0x9D, 0x9E, 0x64, 0x8A, 0x9F];
        let microsoft_os_20 = if capability_uuid == ms_os_uuid && capability_data.len() >= 8 {
            Some(MicrosoftOs20Capability {
                windows_version: u32::from_le_bytes([capability_data[0], capability_data[1], capability_data[2], capability_data[3]]),
                total_length: u16::from_le_bytes([capability_data[4], capability_data[5]]),
                vendor_code: capability_data[6],
                alt_enum_code: capability_data[7],
            })
        } else {
            None
        };
        
        DeviceCapability::Platform(PlatformCapability {
            capability_uuid,
            uuid_string,
            capability_data,
            webusb,
            microsoft_os_20,
        })
    } else {
        DeviceCapability::Unknown { capability_type: 0x05, data: data.to_vec() }
    }
}

fn parse_power_delivery_capability(_data: &[u8]) -> DeviceCapability {
    // Power Delivery capability is handled in power_delivery module
    DeviceCapability::Unknown { capability_type: 0x06, data: _data.to_vec() }
}

fn parse_superspeed_plus_capability(data: &[u8]) -> DeviceCapability {
    if data.len() >= 8 {
        DeviceCapability::SuperSpeedPlus(SuperSpeedPlusCapability {
            reserved: data[0],
            attributes: u32::from_le_bytes([data[1], data[2], data[3], data[4]]),
            functionality_support: u16::from_le_bytes([data[5], data[6]]),
            reserved2: u16::from_le_bytes([data[7], if data.len() > 8 { data[8] } else { 0 }]),
            sublink_speed_attributes: if data.len() > 9 {
                data[9..].chunks(4).map(|chunk| {
                    if chunk.len() >= 4 {
                        u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
                    } else {
                        0
                    }
                }).collect()
            } else {
                Vec::new()
            },
        })
    } else {
        DeviceCapability::Unknown { capability_type: 0x0A, data: data.to_vec() }
    }
}

fn parse_precision_time_measurement(data: &[u8]) -> DeviceCapability {
    DeviceCapability::PrecisionTimeMeasurement(PrecisionTimeMeasurementCapability {
        reserved: if !data.is_empty() { data[0] } else { 0 },
    })
}

fn parse_wireless_ext_hub(data: &[u8]) -> DeviceCapability {
    DeviceCapability::WirelessExtHub(WirelessExtHubCapability {
        num_ports: if !data.is_empty() { data[0] } else { 0 },
    })
}

fn parse_billboard_capability(data: &[u8]) -> DeviceCapability {
    if data.len() >= 7 {
        let num_alternate_modes = data[0];
        let preferred_alternate_mode = data[1];
        let vconn_power_raw = (data[2] >> 1) & 0x07;
        let vconn_power = match vconn_power_raw {
            0 => VconnPower::Watts1,
            1 => VconnPower::Watts1_5,
            2 => VconnPower::Watts2,
            3 => VconnPower::Watts3,
            4 => VconnPower::Watts4,
            5 => VconnPower::Watts5,
            6 => VconnPower::Watts6,
            n => VconnPower::Reserved(n),
        };
        let version = u16::from_le_bytes([data[4], data[5]]);
        let additional_failure_info = (data[6] & 0x01) != 0;
        
        // Parse alternate modes (4 bytes each starting at offset 7)
        let mut alternate_modes = Vec::new();
        let mut offset = 7;
        for i in 0..num_alternate_modes {
            if offset + 4 <= data.len() {
                let svid = u16::from_le_bytes([data[offset], data[offset + 1]]);
                let state_raw = (data[offset + 2] >> ((i % 4) * 2)) & 0x03;
                let state = match state_raw {
                    0 => AlternateModeState::Unspecified,
                    1 => AlternateModeState::NotAttempted,
                    2 => AlternateModeState::Unsuccessful,
                    3 => AlternateModeState::Successful,
                    _ => AlternateModeState::Unspecified,
                };
                alternate_modes.push(BillboardAlternateMode {
                    svid,
                    mode_index: i,
                    state,
                });
                offset += 4;
            }
        }
        
        DeviceCapability::Billboard(BillboardCapability {
            num_alternate_modes,
            preferred_alternate_mode,
            vconn_power,
            version,
            additional_failure_info,
            alternate_modes,
        })
    } else {
        DeviceCapability::Unknown { capability_type: 0x0D, data: data.to_vec() }
    }
}

fn parse_authentication_capability(data: &[u8]) -> DeviceCapability {
    DeviceCapability::Authentication(AuthenticationCapability {
        auth_types: if !data.is_empty() { data[0] } else { 0 },
    })
}

fn parse_billboard_extension(data: &[u8]) -> DeviceCapability {
    DeviceCapability::BillboardExtension(BillboardExtensionCapability {
        version: if !data.is_empty() { data[0] } else { 0 },
        num_svids: if data.len() > 1 { data[1] } else { 0 },
    })
}

fn parse_configuration_summary(data: &[u8]) -> DeviceCapability {
    if data.len() >= 4 {
        DeviceCapability::ConfigurationSummary(ConfigurationSummaryCapability {
            config_number: data[0],
            function_class: data[1],
            function_subclass: data[2],
            function_protocol: data[3],
        })
    } else {
        DeviceCapability::Unknown { capability_type: 0x10, data: data.to_vec() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usb20_extension_parsing() {
        // LPM supported
        let data = [0x02, 0x00, 0x00, 0x00];
        if let DeviceCapability::Usb20Extension(cap) = parse_usb20_extension(&data) {
            assert!(cap.lpm_supported);
            assert!(!cap.besl_supported);
        } else {
            panic!("Expected USB 2.0 Extension capability");
        }
    }

    #[test]
    fn test_vconn_power() {
        assert!(matches!(VconnPower::Watts1, VconnPower::Watts1));
        assert!(matches!(VconnPower::Watts3, VconnPower::Watts3));
    }
}
