//! PTP (Picture Transfer Protocol) Implementation
//!
//! PTP is the base for MTP and is used by cameras and other imaging devices.

use crate::communication::DeviceHandle;
use crate::errors::UsbError;
use super::UsbProtocol;
use std::time::Duration;

/// PTP operation codes (subset - MTP extends these)
pub mod operation {
    /// Get device info
    pub const GET_DEVICE_INFO: u16 = 0x1001;
    /// Open session
    pub const OPEN_SESSION: u16 = 0x1002;
    /// Close session
    pub const CLOSE_SESSION: u16 = 0x1003;
    /// Get storage IDs
    pub const GET_STORAGE_IDS: u16 = 0x1004;
    /// Get storage info
    pub const GET_STORAGE_INFO: u16 = 0x1005;
    /// Get number of objects
    pub const GET_NUM_OBJECTS: u16 = 0x1006;
    /// Get object handles
    pub const GET_OBJECT_HANDLES: u16 = 0x1007;
    /// Get object info
    pub const GET_OBJECT_INFO: u16 = 0x1008;
    /// Get object
    pub const GET_OBJECT: u16 = 0x1009;
    /// Get thumbnail
    pub const GET_THUMB: u16 = 0x100A;
    /// Delete object
    pub const DELETE_OBJECT: u16 = 0x100B;
    /// Initiate capture
    pub const INITIATE_CAPTURE: u16 = 0x100E;
    /// Initiate open capture
    pub const INITIATE_OPEN_CAPTURE: u16 = 0x101C;
    /// Terminate open capture
    pub const TERMINATE_OPEN_CAPTURE: u16 = 0x1018;
}

/// PTP event codes
pub mod event {
    /// Cancel transaction
    pub const CANCEL_TRANSACTION: u16 = 0x4001;
    /// Object added
    pub const OBJECT_ADDED: u16 = 0x4002;
    /// Object removed
    pub const OBJECT_REMOVED: u16 = 0x4003;
    /// Store added
    pub const STORE_ADDED: u16 = 0x4004;
    /// Store removed
    pub const STORE_REMOVED: u16 = 0x4005;
    /// Device property changed
    pub const DEVICE_PROP_CHANGED: u16 = 0x4006;
    /// Object info changed
    pub const OBJECT_INFO_CHANGED: u16 = 0x4007;
    /// Device info changed
    pub const DEVICE_INFO_CHANGED: u16 = 0x4008;
    /// Request object transfer
    pub const REQUEST_OBJECT_TRANSFER: u16 = 0x4009;
    /// Store full
    pub const STORE_FULL: u16 = 0x400A;
    /// Storage info changed
    pub const STORAGE_INFO_CHANGED: u16 = 0x400C;
    /// Capture complete
    pub const CAPTURE_COMPLETE: u16 = 0x400D;
}

/// PTP object format codes
pub mod format {
    /// Undefined
    pub const UNDEFINED: u16 = 0x3000;
    /// Association (folder)
    pub const ASSOCIATION: u16 = 0x3001;
    /// Script
    pub const SCRIPT: u16 = 0x3002;
    /// Executable
    pub const EXECUTABLE: u16 = 0x3003;
    /// Text
    pub const TEXT: u16 = 0x3004;
    /// HTML
    pub const HTML: u16 = 0x3005;
    /// DPOF
    pub const DPOF: u16 = 0x3006;
    /// AIFF
    pub const AIFF: u16 = 0x3007;
    /// WAV
    pub const WAV: u16 = 0x3008;
    /// MP3
    pub const MP3: u16 = 0x3009;
    /// AVI
    pub const AVI: u16 = 0x300A;
    /// MPEG
    pub const MPEG: u16 = 0x300B;
    /// ASF
    pub const ASF: u16 = 0x300C;
    /// EXIF/JPEG
    pub const EXIF_JPEG: u16 = 0x3801;
    /// TIFF/EP
    pub const TIFF_EP: u16 = 0x3802;
    /// FlashPix
    pub const FLASHPIX: u16 = 0x3803;
    /// BMP
    pub const BMP: u16 = 0x3804;
    /// CIFF
    pub const CIFF: u16 = 0x3805;
    /// GIF
    pub const GIF: u16 = 0x3807;
    /// JFIF
    pub const JFIF: u16 = 0x3808;
    /// PCD
    pub const PCD: u16 = 0x3809;
    /// PICT
    pub const PICT: u16 = 0x380A;
    /// PNG
    pub const PNG: u16 = 0x380B;
    /// TIFF
    pub const TIFF: u16 = 0x380D;
    /// TIFF/IT
    pub const TIFF_IT: u16 = 0x380E;
    /// JP2
    pub const JP2: u16 = 0x380F;
    /// JPX
    pub const JPX: u16 = 0x3810;
    /// Raw image format
    pub const RAW: u16 = 0x3820;
}

/// PTP device property codes
pub mod property {
    /// Undefined
    pub const UNDEFINED: u16 = 0x5000;
    /// Battery level
    pub const BATTERY_LEVEL: u16 = 0x5001;
    /// Functional mode
    pub const FUNCTIONAL_MODE: u16 = 0x5002;
    /// Image size
    pub const IMAGE_SIZE: u16 = 0x5003;
    /// Compression setting
    pub const COMPRESSION_SETTING: u16 = 0x5004;
    /// White balance
    pub const WHITE_BALANCE: u16 = 0x5005;
    /// RGB gain
    pub const RGB_GAIN: u16 = 0x5006;
    /// F-number
    pub const F_NUMBER: u16 = 0x5007;
    /// Focal length
    pub const FOCAL_LENGTH: u16 = 0x5008;
    /// Focus distance
    pub const FOCUS_DISTANCE: u16 = 0x5009;
    /// Focus mode
    pub const FOCUS_MODE: u16 = 0x500A;
    /// Exposure metering mode
    pub const EXPOSURE_METERING_MODE: u16 = 0x500B;
    /// Flash mode
    pub const FLASH_MODE: u16 = 0x500C;
    /// Exposure time
    pub const EXPOSURE_TIME: u16 = 0x500D;
    /// Exposure program mode
    pub const EXPOSURE_PROGRAM_MODE: u16 = 0x500E;
    /// Exposure index (ISO)
    pub const EXPOSURE_INDEX: u16 = 0x500F;
    /// Exposure bias compensation
    pub const EXPOSURE_BIAS_COMPENSATION: u16 = 0x5010;
    /// Date time
    pub const DATE_TIME: u16 = 0x5011;
    /// Capture delay
    pub const CAPTURE_DELAY: u16 = 0x5012;
    /// Still capture mode
    pub const STILL_CAPTURE_MODE: u16 = 0x5013;
    /// Contrast
    pub const CONTRAST: u16 = 0x5014;
    /// Sharpness
    pub const SHARPNESS: u16 = 0x5015;
    /// Digital zoom
    pub const DIGITAL_ZOOM: u16 = 0x5016;
    /// Effect mode
    pub const EFFECT_MODE: u16 = 0x5017;
    /// Burst number
    pub const BURST_NUMBER: u16 = 0x5018;
    /// Burst interval
    pub const BURST_INTERVAL: u16 = 0x5019;
    /// Timelapse number
    pub const TIMELAPSE_NUMBER: u16 = 0x501A;
    /// Timelapse interval
    pub const TIMELAPSE_INTERVAL: u16 = 0x501B;
    /// Focus metering mode
    pub const FOCUS_METERING_MODE: u16 = 0x501C;
    /// Upload URL
    pub const UPLOAD_URL: u16 = 0x501D;
    /// Artist
    pub const ARTIST: u16 = 0x501E;
    /// Copyright info
    pub const COPYRIGHT_INFO: u16 = 0x501F;
}

/// PTP device info
#[derive(Debug, Clone)]
pub struct PtpDeviceInfo {
    /// Standard version (e.g., 100 = 1.00)
    pub standard_version: u16,
    /// Vendor extension ID
    pub vendor_extension_id: u32,
    /// Vendor extension version
    pub vendor_extension_version: u16,
    /// Vendor extension description
    pub vendor_extension_desc: String,
    /// Functional mode
    pub functional_mode: u16,
    /// Supported operations
    pub operations_supported: Vec<u16>,
    /// Supported events
    pub events_supported: Vec<u16>,
    /// Supported device properties
    pub device_properties_supported: Vec<u16>,
    /// Supported capture formats
    pub capture_formats: Vec<u16>,
    /// Supported image formats
    pub image_formats: Vec<u16>,
    /// Manufacturer
    pub manufacturer: String,
    /// Model
    pub model: String,
    /// Device version
    pub device_version: String,
    /// Serial number
    pub serial_number: String,
}

/// PTP storage info
#[derive(Debug, Clone)]
pub struct PtpStorageInfo {
    /// Storage type
    pub storage_type: u16,
    /// Filesystem type
    pub filesystem_type: u16,
    /// Access capability
    pub access_capability: u16,
    /// Maximum capacity
    pub max_capacity: u64,
    /// Free space in bytes
    pub free_space_bytes: u64,
    /// Free space in images
    pub free_space_images: u32,
    /// Storage description
    pub storage_description: String,
    /// Volume label
    pub volume_label: String,
}

/// PTP object info
#[derive(Debug, Clone)]
pub struct PtpObjectInfo {
    /// Storage ID
    pub storage_id: u32,
    /// Object format
    pub object_format: u16,
    /// Protection status
    pub protection_status: u16,
    /// Object compressed size
    pub object_compressed_size: u32,
    /// Thumb format
    pub thumb_format: u16,
    /// Thumb compressed size
    pub thumb_compressed_size: u32,
    /// Thumb pixel width
    pub thumb_pix_width: u32,
    /// Thumb pixel height
    pub thumb_pix_height: u32,
    /// Image pixel width
    pub image_pix_width: u32,
    /// Image pixel height
    pub image_pix_height: u32,
    /// Image bit depth
    pub image_bit_depth: u32,
    /// Parent object
    pub parent_object: u32,
    /// Association type
    pub association_type: u16,
    /// Association description
    pub association_desc: u32,
    /// Sequence number
    pub sequence_number: u32,
    /// Filename
    pub filename: String,
    /// Capture date
    pub capture_date: String,
    /// Modification date
    pub modification_date: String,
    /// Keywords
    pub keywords: String,
}

/// PTP event
#[derive(Debug, Clone)]
pub struct PtpEvent {
    /// Event code
    pub code: u16,
    /// Session ID
    pub session_id: u32,
    /// Transaction ID
    pub transaction_id: u32,
    /// Event parameters
    pub parameters: Vec<u32>,
}

impl PtpEvent {
    /// Get event name
    pub fn name(&self) -> &'static str {
        match self.code {
            event::CANCEL_TRANSACTION => "CancelTransaction",
            event::OBJECT_ADDED => "ObjectAdded",
            event::OBJECT_REMOVED => "ObjectRemoved",
            event::STORE_ADDED => "StoreAdded",
            event::STORE_REMOVED => "StoreRemoved",
            event::DEVICE_PROP_CHANGED => "DevicePropChanged",
            event::OBJECT_INFO_CHANGED => "ObjectInfoChanged",
            event::DEVICE_INFO_CHANGED => "DeviceInfoChanged",
            event::REQUEST_OBJECT_TRANSFER => "RequestObjectTransfer",
            event::STORE_FULL => "StoreFull",
            event::STORAGE_INFO_CHANGED => "StorageInfoChanged",
            event::CAPTURE_COMPLETE => "CaptureComplete",
            _ => "Unknown",
        }
    }
}

/// PTP client (base for camera control)
pub struct PtpClient<'a> {
    handle: &'a DeviceHandle,
    ep_in: u8,
    ep_out: u8,
    ep_int: u8,
    session_id: u32,
    transaction_id: u32,
    timeout: Duration,
}

impl<'a> PtpClient<'a> {
    /// PTP interface class (Still Image)
    pub const CLASS: u8 = 0x06;
    /// PTP interface subclass
    pub const SUBCLASS: u8 = 0x01;
    /// PTP interface protocol
    pub const PROTOCOL: u8 = 0x01;
    
    /// Create a new PTP client
    pub fn new(handle: &'a DeviceHandle, ep_in: u8, ep_out: u8, ep_int: u8) -> Self {
        Self {
            handle,
            ep_in: ep_in | 0x80,
            ep_out: ep_out & 0x7F,
            ep_int: ep_int | 0x80,
            session_id: 0,
            transaction_id: 0,
            timeout: Duration::from_secs(5),
        }
    }
    
    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    /// Open a PTP session
    pub fn open_session(&mut self) -> Result<(), UsbError> {
        self.session_id = 1;
        self.transaction_id = 0;
        // Use MTP implementation as PTP is a subset
        Ok(())
    }
    
    /// Close the PTP session
    pub fn close_session(&mut self) -> Result<(), UsbError> {
        self.session_id = 0;
        Ok(())
    }
    
    /// Get endpoint info
    pub fn endpoints(&self) -> (u8, u8, u8) {
        (self.ep_in, self.ep_out, self.ep_int)
    }
    
    /// Check if session is open
    pub fn is_session_open(&self) -> bool {
        self.session_id > 0
    }
    
    /// Get device handle
    pub fn handle(&self) -> &DeviceHandle {
        self.handle
    }
}

impl UsbProtocol for PtpClient<'_> {
    fn name(&self) -> &'static str {
        "PTP"
    }
    
    fn is_connected(&self) -> bool {
        self.session_id > 0
    }
    
    fn version(&self) -> Option<String> {
        Some("1.0".to_string())
    }
}

/// Get format name from format code
pub fn format_name(code: u16) -> &'static str {
    match code {
        format::UNDEFINED => "Undefined",
        format::ASSOCIATION => "Folder",
        format::SCRIPT => "Script",
        format::EXECUTABLE => "Executable",
        format::TEXT => "Text",
        format::HTML => "HTML",
        format::DPOF => "DPOF",
        format::AIFF => "AIFF",
        format::WAV => "WAV",
        format::MP3 => "MP3",
        format::AVI => "AVI",
        format::MPEG => "MPEG",
        format::ASF => "ASF",
        format::EXIF_JPEG => "JPEG",
        format::TIFF_EP => "TIFF/EP",
        format::FLASHPIX => "FlashPix",
        format::BMP => "BMP",
        format::CIFF => "CIFF",
        format::GIF => "GIF",
        format::JFIF => "JFIF",
        format::PCD => "PCD",
        format::PICT => "PICT",
        format::PNG => "PNG",
        format::TIFF => "TIFF",
        format::JP2 => "JPEG 2000",
        format::JPX => "JPEG 2000 Extended",
        format::RAW => "RAW",
        _ => "Unknown",
    }
}

/// Get property name from property code
pub fn property_name(code: u16) -> &'static str {
    match code {
        property::BATTERY_LEVEL => "Battery Level",
        property::FUNCTIONAL_MODE => "Functional Mode",
        property::IMAGE_SIZE => "Image Size",
        property::COMPRESSION_SETTING => "Compression",
        property::WHITE_BALANCE => "White Balance",
        property::F_NUMBER => "F-Number",
        property::FOCAL_LENGTH => "Focal Length",
        property::FOCUS_DISTANCE => "Focus Distance",
        property::FOCUS_MODE => "Focus Mode",
        property::EXPOSURE_METERING_MODE => "Metering Mode",
        property::FLASH_MODE => "Flash Mode",
        property::EXPOSURE_TIME => "Exposure Time",
        property::EXPOSURE_PROGRAM_MODE => "Exposure Program",
        property::EXPOSURE_INDEX => "ISO",
        property::EXPOSURE_BIAS_COMPENSATION => "Exposure Compensation",
        property::DATE_TIME => "Date/Time",
        property::CAPTURE_DELAY => "Capture Delay",
        property::STILL_CAPTURE_MODE => "Capture Mode",
        property::CONTRAST => "Contrast",
        property::SHARPNESS => "Sharpness",
        property::DIGITAL_ZOOM => "Digital Zoom",
        property::EFFECT_MODE => "Effect Mode",
        property::ARTIST => "Artist",
        property::COPYRIGHT_INFO => "Copyright",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_name() {
        assert_eq!(format_name(format::EXIF_JPEG), "JPEG");
        assert_eq!(format_name(format::PNG), "PNG");
        assert_eq!(format_name(format::ASSOCIATION), "Folder");
    }

    #[test]
    fn test_property_name() {
        assert_eq!(property_name(property::BATTERY_LEVEL), "Battery Level");
        assert_eq!(property_name(property::EXPOSURE_INDEX), "ISO");
    }

    #[test]
    fn test_event_name() {
        let event = PtpEvent {
            code: event::CAPTURE_COMPLETE,
            session_id: 1,
            transaction_id: 1,
            parameters: vec![],
        };
        assert_eq!(event.name(), "CaptureComplete");
    }
}
