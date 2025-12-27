use serde::{Deserialize, Serialize};

/// Unique identifier for a USB device based on Vendor ID and Product ID.
/// 
/// The UsbId represents the device type but does not uniquely identify
/// a specific device instance. For stable device tracking across reconnections,
/// combine this with serial number or port path (see device identity resolution
/// in GLOSSARY.md).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct UsbId {
    pub vid: u16,
    pub pid: u16,
}

impl UsbId {
    pub fn new(vid: u16, pid: u16) -> Self {
        Self { vid, pid }
    }

    pub fn as_hex_string(&self) -> String {
        format!("{:04X}:{:04X}", self.vid, self.pid)
    }
}

/// Physical and logical location information for a USB device.
/// 
/// This information helps with device identity resolution and reconnection tracking:
/// - `bus` and `address`: Temporary identifiers that change on reconnect
/// - `port_path`: Physical topology (stable if device stays in same port)
/// 
/// For stable identification, prefer serial numbers when available, or use
/// port_path for position-dependent tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbLocation {
    pub bus: Option<u8>,
    pub address: Option<u8>,
    pub port_path: Option<String>,
}

/// USB device string descriptors and classification information.
/// 
/// String descriptors are read during Stage 2 of the detection pipeline
/// and may be unavailable due to permissions. Applications should handle
/// missing strings gracefully.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbDescriptorSummary {
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    /// Serial number - preferred for stable device identification
    pub serial_number: Option<String>,
    pub device_class: Option<u8>,
    pub device_subclass: Option<u8>,
    pub device_protocol: Option<u8>,
    pub usb_version: Option<String>,
}

/// Operating system driver binding status for a USB device.
/// 
/// Driver status affects device accessibility and operation capabilities.
/// This information is populated during Stage 3 (Platform Enrichment) and
/// may impact whether device operations require elevated permissions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DriverStatus {
    /// Cannot determine driver state (permissions or platform limitation)
    Unknown,
    /// Driver successfully bound with identified name
    Bound { name: String },
    /// No driver currently bound to device
    Missing,
    /// Driver binding blocked by system or policy
    Blocked { reason: String },
    /// Multiple drivers bound (unusual but possible)
    Multiple { drivers: Vec<String> },
}

/// Physical connection health and stability indicators.
/// 
/// Link health helps detect problematic connections that may cause
/// operation failures. Applications should handle unhealthy devices
/// with appropriate retry logic or user warnings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LinkHealth {
    /// Normal operation, connection stable
    Good,
    /// Intermittent connectivity issues detected
    Unstable { reason: String },
    /// Possible insufficient power delivery
    PowerIssueHint { reason: String },
    /// Device repeatedly resetting (serious issue)
    ResetLoop,
    /// Device no longer accessible on bus
    Disconnected,
}

/// Complete record of a confirmed USB device.
/// 
/// A UsbDeviceRecord represents a fully-identified device that has passed through
/// the complete detection pipeline (transport scanning, descriptor reading, and
/// platform enrichment). This is a "confirmed device" ready for application use.
/// 
/// For device identity resolution across reconnections, use the following strategy:
/// 1. Prefer `descriptor.serial_number` if available (most stable)
/// 2. Use `location.port_path` for position-dependent tracking
/// 3. Fallback to combination of `id` + `location` fields
/// 
/// See GLOSSARY.md for detailed information on device lifecycle and identity resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbDeviceRecord {
    /// Device type identifier (VID/PID)
    pub id: UsbId,
    /// Physical/logical location on USB bus
    pub location: UsbLocation,
    /// String descriptors and USB characteristics
    pub descriptor: UsbDescriptorSummary,
    /// Operating system driver binding status
    pub driver: DriverStatus,
    /// Connection quality and stability
    pub health: LinkHealth,
    /// Protocol and classification tags (ADB, Fastboot, etc.)
    pub tags: Vec<String>,
    /// Platform-specific raw data (optional)
    pub raw_data: Option<String>,
}

impl UsbDeviceRecord {
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t.eq_ignore_ascii_case(tag))
    }

    pub fn add_tag(&mut self, tag: impl Into<String>) {
        let tag = tag.into();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
}
