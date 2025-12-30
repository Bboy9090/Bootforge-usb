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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

/// Extended device record with full descriptor information (God Mode)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedDeviceRecord {
    /// Base device record
    pub base: UsbDeviceRecord,
    /// Device speed capability
    pub speed: DeviceSpeed,
    /// Full configuration descriptors
    pub configurations: Vec<ConfigurationInfo>,
    /// BOS capabilities (USB 2.1+)
    pub capabilities: DeviceCapabilities,
    /// Power delivery status
    pub power: PowerInfo,
    /// Alternate mode capabilities (USB Type-C)
    pub alternate_modes: AlternateModeInfo,
}

/// Device speed information
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DeviceSpeed {
    /// Low Speed (1.5 Mbps)
    Low,
    /// Full Speed (12 Mbps)
    Full,
    /// High Speed (480 Mbps)
    High,
    /// SuperSpeed (5 Gbps)
    Super,
    /// SuperSpeed+ (10 Gbps)
    SuperPlus,
    /// SuperSpeed+ (20 Gbps)
    SuperPlus20,
    /// USB4 (40 Gbps)
    Usb4,
    /// Unknown
    Unknown,
}

impl DeviceSpeed {
    /// Get the maximum bandwidth in Mbps
    pub fn bandwidth_mbps(&self) -> u32 {
        match self {
            DeviceSpeed::Low => 1,
            DeviceSpeed::Full => 12,
            DeviceSpeed::High => 480,
            DeviceSpeed::Super => 5000,
            DeviceSpeed::SuperPlus => 10000,
            DeviceSpeed::SuperPlus20 => 20000,
            DeviceSpeed::Usb4 => 40000,
            DeviceSpeed::Unknown => 0,
        }
    }
}

/// Configuration information with interfaces and endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationInfo {
    /// Configuration number
    pub number: u8,
    /// Configuration description
    pub description: Option<String>,
    /// Maximum power in milliamps
    pub max_power_ma: u16,
    /// Self-powered
    pub self_powered: bool,
    /// Remote wakeup capable
    pub remote_wakeup: bool,
    /// Interfaces in this configuration
    pub interfaces: Vec<InterfaceInfo>,
}

/// Interface information with endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceInfo {
    /// Interface number
    pub number: u8,
    /// Alternate setting
    pub alternate_setting: u8,
    /// Interface class
    pub class: u8,
    /// Interface subclass
    pub subclass: u8,
    /// Interface protocol
    pub protocol: u8,
    /// Interface description
    pub description: Option<String>,
    /// Class name (human readable)
    pub class_name: Option<String>,
    /// Endpoints
    pub endpoints: Vec<EndpointInfo>,
}

/// Endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointInfo {
    /// Endpoint address
    pub address: u8,
    /// Endpoint number (0-15)
    pub number: u8,
    /// Direction (In/Out)
    pub direction: EndpointDir,
    /// Transfer type
    pub transfer_type: EndpointTransferType,
    /// Maximum packet size
    pub max_packet_size: u16,
    /// Polling interval (for interrupt/isochronous)
    pub interval_ms: f64,
}

/// Endpoint direction
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EndpointDir {
    In,
    Out,
}

/// Endpoint transfer type
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EndpointTransferType {
    Control,
    Isochronous,
    Bulk,
    Interrupt,
}

/// Device capabilities from BOS descriptor
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    /// USB 2.0 LPM (Link Power Management) supported
    pub lpm_supported: bool,
    /// SuperSpeed capable
    pub superspeed: bool,
    /// SuperSpeedPlus capable
    pub superspeed_plus: bool,
    /// USB4 capable
    pub usb4: bool,
    /// Container ID (if available)
    pub container_id: Option<String>,
    /// WebUSB capable
    pub webusb: bool,
    /// Microsoft OS 2.0 descriptors
    pub microsoft_os_20: bool,
}

/// Power delivery information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PowerInfo {
    /// USB-PD supported
    pub pd_supported: bool,
    /// Current voltage (mV)
    pub voltage_mv: Option<u32>,
    /// Current (mA)
    pub current_ma: Option<u32>,
    /// Power role
    pub power_role: Option<String>,
    /// Maximum power (mW)
    pub max_power_mw: Option<u32>,
}

/// Alternate mode information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AlternateModeInfo {
    /// DisplayPort alternate mode supported
    pub displayport: bool,
    /// Thunderbolt alternate mode supported
    pub thunderbolt: bool,
    /// Maximum DisplayPort resolution (if supported)
    pub dp_max_resolution: Option<String>,
    /// Thunderbolt version (if supported)
    pub tb_version: Option<String>,
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

impl std::fmt::Display for UsbDeviceRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.descriptor.product.as_deref().unwrap_or("Unknown Device");
        let mfg = self.descriptor.manufacturer.as_deref().unwrap_or("");
        write!(f, "{} {} [{:04X}:{:04X}]", mfg, name, self.id.vid, self.id.pid)
    }
}
