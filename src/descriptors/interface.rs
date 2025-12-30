//! Interface Descriptor Parsing
//!
//! USB Interface descriptors describe a specific interface within a configuration,
//! including all endpoints and their characteristics.

use super::endpoint::EndpointDescriptor;
use super::{UsbClass, DESCRIPTOR_TIMEOUT};
use serde::{Deserialize, Serialize};

/// USB Interface Descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceDescriptor {
    /// Interface number
    pub number: u8,
    /// Alternate setting number
    pub alternate_setting: u8,
    /// Interface string descriptor
    pub description: Option<String>,
    /// Interface class code
    pub class: UsbClass,
    /// Interface subclass code
    pub subclass: u8,
    /// Interface protocol code
    pub protocol: u8,
    /// Number of endpoints (excluding endpoint 0)
    pub num_endpoints: u8,
    /// All endpoints in this interface
    pub endpoints: Vec<EndpointDescriptor>,
    /// Parsed class-specific information
    pub class_specific: Option<ClassSpecificInfo>,
    /// Raw extra bytes (class-specific descriptors)
    pub extra: Vec<u8>,
}

/// Class-specific interface information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClassSpecificInfo {
    /// HID-specific information
    Hid(HidInfo),
    /// Audio-specific information
    Audio(AudioInfo),
    /// Video-specific information
    Video(VideoInfo),
    /// CDC-specific information
    Cdc(CdcInfo),
    /// Mass Storage-specific information
    MassStorage(MassStorageInfo),
    /// Unknown class-specific data
    Unknown(Vec<u8>),
}

/// HID (Human Interface Device) specific information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HidInfo {
    /// HID specification version (BCD)
    pub hid_version: u16,
    /// Country code
    pub country_code: u8,
    /// Number of HID descriptors
    pub num_descriptors: u8,
    /// HID report descriptor length
    pub report_descriptor_length: u16,
    /// Parsed report descriptor (if available)
    pub report_descriptor: Option<Vec<u8>>,
    /// HID subclass
    pub subclass: HidSubclass,
    /// HID protocol
    pub protocol: HidProtocol,
}

/// HID Subclass
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HidSubclass {
    /// No subclass
    None,
    /// Boot interface subclass
    Boot,
    /// Unknown subclass
    Unknown(u8),
}

impl From<u8> for HidSubclass {
    fn from(value: u8) -> Self {
        match value {
            0x00 => HidSubclass::None,
            0x01 => HidSubclass::Boot,
            other => HidSubclass::Unknown(other),
        }
    }
}

/// HID Protocol (for boot subclass)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HidProtocol {
    /// No specific protocol
    None,
    /// Keyboard
    Keyboard,
    /// Mouse
    Mouse,
    /// Unknown protocol
    Unknown(u8),
}

impl From<u8> for HidProtocol {
    fn from(value: u8) -> Self {
        match value {
            0x00 => HidProtocol::None,
            0x01 => HidProtocol::Keyboard,
            0x02 => HidProtocol::Mouse,
            other => HidProtocol::Unknown(other),
        }
    }
}

/// Audio class specific information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioInfo {
    /// Audio class subclass
    pub subclass: AudioSubclass,
    /// Audio specification version (BCD)
    pub audio_version: Option<u16>,
    /// Total length of audio class-specific descriptors
    pub total_length: u16,
    /// Number of streaming interfaces
    pub num_streaming_interfaces: u8,
}

/// Audio Subclass
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioSubclass {
    /// Undefined
    Undefined,
    /// Audio Control
    AudioControl,
    /// Audio Streaming
    AudioStreaming,
    /// MIDI Streaming
    MidiStreaming,
    /// Unknown
    Unknown(u8),
}

impl From<u8> for AudioSubclass {
    fn from(value: u8) -> Self {
        match value {
            0x00 => AudioSubclass::Undefined,
            0x01 => AudioSubclass::AudioControl,
            0x02 => AudioSubclass::AudioStreaming,
            0x03 => AudioSubclass::MidiStreaming,
            other => AudioSubclass::Unknown(other),
        }
    }
}

/// Video class specific information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    /// Video subclass
    pub subclass: VideoSubclass,
    /// UVC specification version (BCD)
    pub uvc_version: Option<u16>,
    /// Video formats supported
    pub formats: Vec<String>,
}

/// Video Subclass
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VideoSubclass {
    /// Undefined
    Undefined,
    /// Video Control
    VideoControl,
    /// Video Streaming
    VideoStreaming,
    /// Video Interface Collection
    VideoInterfaceCollection,
    /// Unknown
    Unknown(u8),
}

impl From<u8> for VideoSubclass {
    fn from(value: u8) -> Self {
        match value {
            0x00 => VideoSubclass::Undefined,
            0x01 => VideoSubclass::VideoControl,
            0x02 => VideoSubclass::VideoStreaming,
            0x03 => VideoSubclass::VideoInterfaceCollection,
            other => VideoSubclass::Unknown(other),
        }
    }
}

/// CDC (Communications Device Class) specific information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdcInfo {
    /// CDC subclass
    pub subclass: CdcSubclass,
    /// CDC protocol
    pub protocol: CdcProtocol,
    /// Functional descriptors
    pub functional_descriptors: Vec<CdcFunctionalDescriptor>,
}

/// CDC Subclass
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CdcSubclass {
    /// Direct Line Control Model
    DirectLine,
    /// Abstract Control Model (modem)
    AbstractControl,
    /// Telephone Control Model
    TelephoneControl,
    /// Multi-Channel Control Model
    MultiChannel,
    /// CAPI Control Model
    CapiControl,
    /// Ethernet Networking Control Model
    EthernetControl,
    /// ATM Networking Control Model
    AtmControl,
    /// Wireless Handset Control Model
    WirelessHandset,
    /// Device Management
    DeviceManagement,
    /// Mobile Direct Line Model
    MobileDirectLine,
    /// OBEX
    Obex,
    /// Ethernet Emulation Model
    EthernetEmulation,
    /// Network Control Model
    NetworkControl,
    /// Unknown
    Unknown(u8),
}

impl From<u8> for CdcSubclass {
    fn from(value: u8) -> Self {
        match value {
            0x01 => CdcSubclass::DirectLine,
            0x02 => CdcSubclass::AbstractControl,
            0x03 => CdcSubclass::TelephoneControl,
            0x04 => CdcSubclass::MultiChannel,
            0x05 => CdcSubclass::CapiControl,
            0x06 => CdcSubclass::EthernetControl,
            0x07 => CdcSubclass::AtmControl,
            0x08 => CdcSubclass::WirelessHandset,
            0x09 => CdcSubclass::DeviceManagement,
            0x0A => CdcSubclass::MobileDirectLine,
            0x0B => CdcSubclass::Obex,
            0x0C => CdcSubclass::EthernetEmulation,
            0x0D => CdcSubclass::NetworkControl,
            other => CdcSubclass::Unknown(other),
        }
    }
}

/// CDC Protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CdcProtocol {
    /// No class specific protocol required
    None,
    /// AT Commands: V.250 etc
    AtV250,
    /// AT Commands defined by PCCA-101
    AtPcca101,
    /// AT Commands defined by PCCA-101 & Annex O
    AtPcca101AnnexO,
    /// AT Commands defined by GSM 07.07
    AtGsm0707,
    /// AT Commands defined by 3GPP 27.007
    At3gpp27007,
    /// AT Commands defined by TIA for CDMA
    AtTiaCdma,
    /// Ethernet Emulation Model
    EthernetEmulationModel,
    /// External Protocol
    External,
    /// Vendor-specific
    VendorSpecific,
    /// Unknown
    Unknown(u8),
}

impl From<u8> for CdcProtocol {
    fn from(value: u8) -> Self {
        match value {
            0x00 => CdcProtocol::None,
            0x01 => CdcProtocol::AtV250,
            0x02 => CdcProtocol::AtPcca101,
            0x03 => CdcProtocol::AtPcca101AnnexO,
            0x04 => CdcProtocol::AtGsm0707,
            0x05 => CdcProtocol::At3gpp27007,
            0x06 => CdcProtocol::AtTiaCdma,
            0x07 => CdcProtocol::EthernetEmulationModel,
            0xFE => CdcProtocol::External,
            0xFF => CdcProtocol::VendorSpecific,
            other => CdcProtocol::Unknown(other),
        }
    }
}

/// CDC Functional Descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdcFunctionalDescriptor {
    /// Descriptor subtype
    pub subtype: u8,
    /// Descriptor data
    pub data: Vec<u8>,
}

/// Mass Storage specific information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MassStorageInfo {
    /// Mass storage subclass
    pub subclass: MassStorageSubclass,
    /// Mass storage protocol
    pub protocol: MassStorageProtocol,
}

/// Mass Storage Subclass
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MassStorageSubclass {
    /// SCSI command set not reported
    ScsiNotReported,
    /// RBC
    Rbc,
    /// MMC-5 (ATAPI)
    Atapi,
    /// Obsolete
    Obsolete,
    /// UFI
    Ufi,
    /// Obsolete
    Obsolete2,
    /// SCSI transparent command set
    ScsiTransparent,
    /// LSD FS
    LsdFs,
    /// IEEE 1667
    Ieee1667,
    /// Unknown
    Unknown(u8),
}

impl From<u8> for MassStorageSubclass {
    fn from(value: u8) -> Self {
        match value {
            0x00 => MassStorageSubclass::ScsiNotReported,
            0x01 => MassStorageSubclass::Rbc,
            0x02 => MassStorageSubclass::Atapi,
            0x03 => MassStorageSubclass::Obsolete,
            0x04 => MassStorageSubclass::Ufi,
            0x05 => MassStorageSubclass::Obsolete2,
            0x06 => MassStorageSubclass::ScsiTransparent,
            0x07 => MassStorageSubclass::LsdFs,
            0x08 => MassStorageSubclass::Ieee1667,
            other => MassStorageSubclass::Unknown(other),
        }
    }
}

/// Mass Storage Protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MassStorageProtocol {
    /// CBI with command completion interrupt
    CbiWithInterrupt,
    /// CBI without command completion interrupt
    CbiNoInterrupt,
    /// Obsolete
    Obsolete,
    /// Bulk-Only Transport (BOT)
    BulkOnly,
    /// UAS (USB Attached SCSI)
    Uas,
    /// Vendor Specific
    VendorSpecific,
    /// Unknown
    Unknown(u8),
}

impl From<u8> for MassStorageProtocol {
    fn from(value: u8) -> Self {
        match value {
            0x00 => MassStorageProtocol::CbiWithInterrupt,
            0x01 => MassStorageProtocol::CbiNoInterrupt,
            0x02 => MassStorageProtocol::Obsolete,
            0x50 => MassStorageProtocol::BulkOnly,
            0x62 => MassStorageProtocol::Uas,
            0xFF => MassStorageProtocol::VendorSpecific,
            other => MassStorageProtocol::Unknown(other),
        }
    }
}

/// Parse an interface descriptor from rusb
pub fn parse_interface(
    interface: &rusb::InterfaceDescriptor,
    handle: Option<&rusb::DeviceHandle<rusb::Context>>,
) -> InterfaceDescriptor {
    // Try to get interface string
    let description = handle.and_then(|h| {
        let langs = h.read_languages(DESCRIPTOR_TIMEOUT).ok()?;
        let lang = langs.first()?;
        h.read_string_descriptor(*lang, interface.description_string_index()?, DESCRIPTOR_TIMEOUT)
            .ok()
    });
    
    // Parse all endpoints
    let mut endpoints = Vec::new();
    for endpoint in interface.endpoint_descriptors() {
        let parsed = super::endpoint::parse_endpoint(&endpoint);
        endpoints.push(parsed);
    }
    
    let class = UsbClass::from(interface.class_code());
    let subclass = interface.sub_class_code();
    let protocol = interface.protocol_code();
    
    // Parse class-specific information
    let class_specific = parse_class_specific(class, subclass, protocol, interface.extra());
    
    InterfaceDescriptor {
        number: interface.interface_number(),
        alternate_setting: interface.setting_number(),
        description,
        class,
        subclass,
        protocol,
        num_endpoints: interface.num_endpoints(),
        endpoints,
        class_specific,
        extra: interface.extra().to_vec(),
    }
}

/// Parse class-specific interface information
fn parse_class_specific(
    class: UsbClass,
    subclass: u8,
    protocol: u8,
    extra: &[u8],
) -> Option<ClassSpecificInfo> {
    match class {
        UsbClass::Hid => {
            // Parse HID descriptor from extra bytes
            if extra.len() >= 6 {
                let hid_version = u16::from_le_bytes([extra[2], extra[3]]);
                let country_code = extra[4];
                let num_descriptors = extra[5];
                let report_descriptor_length = if extra.len() >= 9 {
                    u16::from_le_bytes([extra[7], extra[8]])
                } else {
                    0
                };
                
                Some(ClassSpecificInfo::Hid(HidInfo {
                    hid_version,
                    country_code,
                    num_descriptors,
                    report_descriptor_length,
                    report_descriptor: None, // Would need control transfer to read
                    subclass: HidSubclass::from(subclass),
                    protocol: HidProtocol::from(protocol),
                }))
            } else {
                Some(ClassSpecificInfo::Hid(HidInfo {
                    hid_version: 0,
                    country_code: 0,
                    num_descriptors: 0,
                    report_descriptor_length: 0,
                    report_descriptor: None,
                    subclass: HidSubclass::from(subclass),
                    protocol: HidProtocol::from(protocol),
                }))
            }
        }
        UsbClass::Audio => {
            Some(ClassSpecificInfo::Audio(AudioInfo {
                subclass: AudioSubclass::from(subclass),
                audio_version: None,
                total_length: extra.len() as u16,
                num_streaming_interfaces: 0,
            }))
        }
        UsbClass::Video => {
            Some(ClassSpecificInfo::Video(VideoInfo {
                subclass: VideoSubclass::from(subclass),
                uvc_version: None,
                formats: Vec::new(),
            }))
        }
        UsbClass::CdcControl | UsbClass::CdcData => {
            Some(ClassSpecificInfo::Cdc(CdcInfo {
                subclass: CdcSubclass::from(subclass),
                protocol: CdcProtocol::from(protocol),
                functional_descriptors: parse_cdc_functional_descriptors(extra),
            }))
        }
        UsbClass::MassStorage => {
            Some(ClassSpecificInfo::MassStorage(MassStorageInfo {
                subclass: MassStorageSubclass::from(subclass),
                protocol: MassStorageProtocol::from(protocol),
            }))
        }
        _ if !extra.is_empty() => {
            Some(ClassSpecificInfo::Unknown(extra.to_vec()))
        }
        _ => None,
    }
}

/// Parse CDC functional descriptors
fn parse_cdc_functional_descriptors(extra: &[u8]) -> Vec<CdcFunctionalDescriptor> {
    let mut descriptors = Vec::new();
    let mut offset = 0;
    
    while offset + 2 < extra.len() {
        let length = extra[offset] as usize;
        if length < 3 || offset + length > extra.len() {
            break;
        }
        
        let desc_type = extra[offset + 1];
        if desc_type == 0x24 {
            // CS_INTERFACE descriptor
            let subtype = extra[offset + 2];
            let data = extra[offset + 3..offset + length].to_vec();
            descriptors.push(CdcFunctionalDescriptor { subtype, data });
        }
        
        offset += length;
    }
    
    descriptors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hid_subclass() {
        assert_eq!(HidSubclass::from(0x00), HidSubclass::None);
        assert_eq!(HidSubclass::from(0x01), HidSubclass::Boot);
    }

    #[test]
    fn test_hid_protocol() {
        assert_eq!(HidProtocol::from(0x01), HidProtocol::Keyboard);
        assert_eq!(HidProtocol::from(0x02), HidProtocol::Mouse);
    }

    #[test]
    fn test_mass_storage_protocol() {
        assert_eq!(MassStorageProtocol::from(0x50), MassStorageProtocol::BulkOnly);
        assert_eq!(MassStorageProtocol::from(0x62), MassStorageProtocol::Uas);
    }
}
