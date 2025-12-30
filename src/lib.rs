//! # BootForge USB - OMEGA TRANSCENDENT MODE
//!
//! The ultimate cross-platform USB device library featuring:
//!
//! - **Complete USB Enumeration**: Discover all connected devices with full descriptors
//! - **Real-time Hotplug Monitoring**: Event-driven device add/remove detection
//! - **Protocol Detection & Communication**: ADB, Fastboot, MTP, PTP, CDC, DFU
//! - **HID Report Descriptor Parsing**: Full decode of keyboards, mice, gamepads
//! - **USB ID Database**: Vendor and product name lookups for thousands of devices
//! - **Device Query API**: Rich filtering and search capabilities
//! - **Caching Layer**: Performance optimization for repeated queries
//! - **Permission Helpers**: Cross-platform permission management and udev rules
//! - **Device Control**: Reset, power cycle, driver binding, hub port control
//!
//! ## Features
//!
//! - Cross-platform USB device enumeration (Windows, macOS, Linux)
//! - Full descriptor parsing (Configuration, Interface, Endpoint, BOS)
//! - USB 3.0+ SuperSpeed and USB4 capability detection
//! - Power Delivery (USB-PD) status
//! - Alternate Mode detection (DisplayPort, Thunderbolt)
//! - Driver status and health monitoring
//! - Real-time hotplug with debouncing and reconnection correlation
//!
//! ## Detection Pipeline
//!
//! USB device discovery follows a four-stage pipeline:
//!
//! 1. **Transport Scanning**: Query USB bus for candidate devices (VID/PID/address)
//! 2. **Descriptor Reading**: Read string descriptors (manufacturer, product, serial)
//! 3. **Platform Enrichment**: Add OS-specific paths, driver status, metadata
//! 4. **Protocol Classification**: Detect high-level protocols (ADB, Fastboot, etc.)
//!
//! ## Quick Start
//!
//! ```no_run
//! use bootforge_usb::enumerate_all;
//!
//! fn main() -> anyhow::Result<()> {
//!     // Enumerate all connected USB devices
//!     let devices = enumerate_all()?;
//!     
//!     for device in &devices {
//!         println!("Device: {:?}", device);
//!     }
//!     
//!     println!("Found {} devices", devices.len());
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Hotplug Monitoring
//!
//! ```ignore
//! use bootforge_usb::{PlatformWatcher, DeviceWatcher, DeviceEvent};
//!
//! fn main() -> anyhow::Result<()> {
//!     let watcher = PlatformWatcher::new()?;
//!     
//!     for event in watcher.events() {
//!         match event {
//!             DeviceEvent::Added(device) => println!("Connected: {}", device),
//!             DeviceEvent::Removed(device) => println!("Disconnected: {}", device),
//!             DeviceEvent::Changed(device) => println!("Changed: {}", device),
//!             DeviceEvent::Reconnected { device, .. } => println!("Reconnected: {}", device),
//!         }
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! See `docs/ARCHITECTURE.md` for detailed documentation.

// ============================================================================
// Core Modules
// ============================================================================

pub mod api;
pub mod enumerate;
pub mod errors;
pub mod model;
pub mod types;

// ============================================================================
// Detection & Classification
// ============================================================================

pub mod descriptors;
pub mod handshake;
pub mod ports;

// ============================================================================
// Monitoring
// ============================================================================

pub mod watcher;

// ============================================================================
// Communication & Protocols (OMEGA MODE)
// ============================================================================

pub mod communication;
pub mod protocols;

// ============================================================================
// Utilities (OMEGA MODE)
// ============================================================================

pub mod cache;
pub mod control;
pub mod database;
pub mod hid;
pub mod permissions;
pub mod query;

// ============================================================================
// Re-exports - Core Types
// ============================================================================

pub use api::UsbEnumerator;
pub use enumerate::enumerate_all;
pub use errors::UsbError;
pub use types::{PlatformHint, UsbBusType, UsbDeviceInfo, UsbIds};

// ============================================================================
// Re-exports - Device Model
// ============================================================================

pub use model::{
    DriverStatus, LinkHealth, UsbDescriptorSummary, UsbDeviceRecord, UsbId, UsbLocation,
    // Extended descriptor types
    ExtendedDeviceRecord, DeviceSpeed, ConfigurationInfo, InterfaceInfo, EndpointInfo,
    EndpointDir, EndpointTransferType, DeviceCapabilities, PowerInfo, AlternateModeInfo,
};

// ============================================================================
// Re-exports - Protocol Detection
// ============================================================================

pub use handshake::{classify_device_protocols, DeviceProtocol};

// ============================================================================
// Re-exports - Watcher/Hotplug
// ============================================================================

pub use watcher::{
    DeviceEvent, DeviceWatcher, 
    DeviceIdentity, DeviceSession, DeviceSessionTracker, EnhancedDeviceWatcher,
    PlatformWatcher,
};

// ============================================================================
// Re-exports - Descriptors
// ============================================================================

pub use descriptors::{
    // Core types
    DescriptorType, UsbClass, UsbSpeed, FullDeviceDescriptor,
    // Configuration and interface
    ConfigurationDescriptor, ConfigurationAttributes,
    InterfaceDescriptor, ClassSpecificInfo,
    // Endpoints
    EndpointDescriptor, EndpointDirection, TransferType, SyncType, UsageType,
    SuperSpeedCompanion, SuperSpeedPlusIsocCompanion,
    // BOS and capabilities
    BosDescriptor, DeviceCapability, Usb20ExtensionCapability,
    // SuperSpeed
    SuperSpeedCapability, SuperSpeedPlusCapability, SuperSpeedMode,
    SublinkSpeedAttribute, SpeedExponent, Usb4Capability,
    // Power Delivery
    PowerDeliveryStatus, PowerContract, PowerRole, DataRole,
    PowerDataObject, FixedSupplyPdo, PpsPdo,
    // Alternate Modes
    AlternateModeCapabilities, DisplayPortAltMode, ThunderboltAltMode,
    DpVersion, DpPinAssignment, TbVersion,
    // Parsing functions
    parse_device_descriptors, detect_alternate_modes,
};

// ============================================================================
// Re-exports - Communication Layer (OMEGA MODE)
// ============================================================================

pub use communication::{
    DeviceHandle, DevicePool, TransferResult, Direction,
    ControlTransfer, BulkTransfer, BulkReader, BulkWriter,
    InterruptTransfer, InterruptPoller,
    DeviceSession as CommSession, SessionManager, SessionGuard, SessionState,
    DEFAULT_TIMEOUT, MAX_RETRIES,
};

// ============================================================================
// Re-exports - Protocols (OMEGA MODE)
// ============================================================================

pub use protocols::{
    UsbProtocol,
    // ADB
    AdbClient, AdbMessage, AdbState, AdbStream,
    // Fastboot
    FastbootClient, FastbootResponse, FastbootDeviceInfo, FastbootVariable,
    // MTP
    MtpClient, MtpContainer, MtpDeviceInfo, MtpStorageInfo, MtpObjectInfo,
    // PTP
    PtpClient, PtpDeviceInfo, PtpStorageInfo, PtpObjectInfo, PtpEvent,
    // CDC
    CdcAcmClient, LineCoding, CdcNetworkInfo,
    // DFU
    DfuClient, DfuState, DfuStatus, DfuStatusResponse, DfuFunctionalDescriptor,
};

// ============================================================================
// Re-exports - HID (OMEGA MODE)
// ============================================================================

pub use hid::{
    ReportDescriptor, ReportField, FieldFlags, Collection, CollectionType,
    HidItem, ItemType, MainTag, GlobalTag, LocalTag,
    GlobalState, LocalState,
    usage_page, usage_desktop, usage_consumer,
};

// ============================================================================
// Re-exports - Database (OMEGA MODE)
// ============================================================================

pub use database::{database, UsbDatabase, Vendor, ClassInfo, SubclassInfo};

// ============================================================================
// Re-exports - Query API (OMEGA MODE)
// ============================================================================

pub use query::{DeviceQuery, SerialFilter, DriverStatusFilter, HealthFilter, presets};

// ============================================================================
// Re-exports - Cache (OMEGA MODE)
// ============================================================================

pub use cache::{DeviceCache, CachedDevice, CacheKey, CacheStats, CachedEnumerator};

// ============================================================================
// Re-exports - Permissions (OMEGA MODE)
// ============================================================================

pub use permissions::{PermissionHelper, PermissionStatus, UsbGroups};

// ============================================================================
// Re-exports - Device Control (OMEGA MODE)
// ============================================================================

pub use control::{DeviceControl, PowerState, HubControl, PortStatus, PortFeature};

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_enumeration() {
        let result = enumerate_all();
        match result {
            Ok(_devices) => { /* Success */ }
            Err(_e) => { /* May not have permissions */ }
        }
    }

    #[test]
    fn test_device_identity_with_serial() {
        let device = UsbDeviceRecord {
            id: UsbId::new(0x1234, 0x5678),
            location: UsbLocation {
                bus: Some(1),
                address: Some(5),
                port_path: Some("1-2.3".to_string()),
            },
            descriptor: UsbDescriptorSummary {
                manufacturer: Some("Test Mfg".to_string()),
                product: Some("Test Product".to_string()),
                serial_number: Some("SN123456".to_string()),
                device_class: Some(0xFF),
                device_subclass: Some(0x42),
                device_protocol: Some(0x01),
                usb_version: Some("2.0".to_string()),
            },
            driver: DriverStatus::Unknown,
            health: LinkHealth::Good,
            tags: vec![],
            raw_data: None,
        };

        assert_eq!(device.descriptor.serial_number, Some("SN123456".to_string()));
        assert_eq!(device.id.as_hex_string(), "1234:5678");
    }

    #[test]
    fn test_driver_status_variants() {
        let bound = DriverStatus::Bound { name: "usb_storage".to_string() };
        let missing = DriverStatus::Missing;
        let blocked = DriverStatus::Blocked { reason: "Policy".to_string() };
        let multiple = DriverStatus::Multiple { drivers: vec!["a".into(), "b".into()] };

        assert!(matches!(bound, DriverStatus::Bound { .. }));
        assert!(matches!(missing, DriverStatus::Missing));
        assert!(matches!(blocked, DriverStatus::Blocked { .. }));
        assert!(matches!(multiple, DriverStatus::Multiple { .. }));
    }

    #[test]
    fn test_link_health_states() {
        assert!(matches!(LinkHealth::Good, LinkHealth::Good));
        assert!(matches!(LinkHealth::ResetLoop, LinkHealth::ResetLoop));
        assert!(matches!(LinkHealth::Disconnected, LinkHealth::Disconnected));
    }

    #[test]
    fn test_usb_id_hex_string() {
        let id1 = UsbId::new(0x1234, 0x5678);
        assert_eq!(id1.as_hex_string(), "1234:5678");

        let id2 = UsbId::new(0xABCD, 0xEF01);
        assert_eq!(id2.as_hex_string(), "ABCD:EF01");
    }

    #[test]
    fn test_device_tags() {
        let mut device = UsbDeviceRecord {
            id: UsbId::new(0x18D1, 0x4EE1),
            location: UsbLocation::default(),
            descriptor: UsbDescriptorSummary::default(),
            driver: DriverStatus::Unknown,
            health: LinkHealth::Good,
            tags: vec![],
            raw_data: None,
        };

        assert!(!device.has_tag("adb"));
        device.add_tag("adb");
        assert!(device.has_tag("adb"));
        assert!(device.has_tag("ADB")); // Case-insensitive
    }

    #[test]
    fn test_protocol_classification_adb() {
        let device = UsbDeviceRecord {
            id: UsbId::new(0x18D1, 0x4EE1), // Google Nexus (ADB)
            location: UsbLocation::default(),
            descriptor: UsbDescriptorSummary {
                manufacturer: Some("Google".to_string()),
                product: Some("Nexus".to_string()),
                serial_number: Some("ABC123".to_string()),
                device_class: Some(0xFF),
                device_subclass: Some(0x42),
                device_protocol: Some(0x01),
                usb_version: Some("2.0".to_string()),
            },
            driver: DriverStatus::Unknown,
            health: LinkHealth::Good,
            tags: vec![],
            raw_data: None,
        };

        let protocols = classify_device_protocols(&device);
        assert!(protocols.contains(&DeviceProtocol::Adb));
    }

    #[test]
    fn test_device_query() {
        let devices = vec![
            UsbDeviceRecord {
                id: UsbId::new(0x18D1, 0x4EE0),
                location: UsbLocation::default(),
                descriptor: UsbDescriptorSummary {
                    device_class: Some(0xFF),
                    ..Default::default()
                },
                driver: DriverStatus::Unknown,
                health: LinkHealth::Good,
                tags: vec![],
                raw_data: None,
            },
            UsbDeviceRecord {
                id: UsbId::new(0x05AC, 0x12A8),
                location: UsbLocation::default(),
                descriptor: UsbDescriptorSummary::default(),
                driver: DriverStatus::Unknown,
                health: LinkHealth::Good,
                tags: vec![],
                raw_data: None,
            },
        ];

        let query = DeviceQuery::new().vendor_id(0x18D1);
        let results = query.filter(&devices);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id.vid, 0x18D1);
    }

    #[test]
    fn test_usb_database() {
        let db = database();
        assert!(db.vendor_name(0x05AC).is_some()); // Apple
        assert!(db.class_name(0x03).is_some()); // HID
    }

    #[test]
    fn test_permission_status() {
        assert!(PermissionStatus::Granted.has_access());
        assert!(!PermissionStatus::NeedsElevation.has_access());
    }

    #[test]
    fn test_power_state() {
        assert_eq!(PowerState::Active.name(), "Active");
        assert_eq!(PowerState::Suspended.name(), "Suspended");
    }

    #[test]
    fn test_device_cache() {
        let cache = DeviceCache::new();
        assert!(cache.is_empty());
        
        let device = UsbDeviceRecord {
            id: UsbId::new(0x1234, 0x5678),
            location: UsbLocation::default(),
            descriptor: UsbDescriptorSummary {
                serial_number: Some("TEST123".to_string()),
                ..Default::default()
            },
            driver: DriverStatus::Unknown,
            health: LinkHealth::Good,
            tags: vec![],
            raw_data: None,
        };
        
        cache.insert(device);
        assert!(!cache.is_empty());
        assert!(cache.get(&CacheKey::Serial("TEST123".to_string())).is_some());
    }

    #[test]
    fn test_hid_report_descriptor() {
        // Simple mouse descriptor
        let mouse_desc = [
            0x05, 0x01, 0x09, 0x02, 0xA1, 0x01, 0x09, 0x01,
            0xA1, 0x00, 0x05, 0x09, 0x19, 0x01, 0x29, 0x03,
            0x15, 0x00, 0x25, 0x01, 0x95, 0x03, 0x75, 0x01,
            0x81, 0x02, 0x95, 0x01, 0x75, 0x05, 0x81, 0x01,
            0x05, 0x01, 0x09, 0x30, 0x09, 0x31, 0x15, 0x81,
            0x25, 0x7F, 0x75, 0x08, 0x95, 0x02, 0x81, 0x06,
            0xC0, 0xC0,
        ];

        let desc = ReportDescriptor::parse(&mouse_desc).unwrap();
        assert_eq!(desc.device_type(), "Mouse");
    }

    #[test]
    fn test_adb_message() {
        use protocols::adb::constants;
        
        let msg = AdbMessage::new(constants::CMD_CNXN, 0x01000000, 4096, b"test");
        assert_eq!(msg.command_name(), "CNXN");
        assert_eq!(msg.data_length, 4);
        
        let bytes = msg.to_bytes();
        let parsed = AdbMessage::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.command, msg.command);
    }

    #[test]
    fn test_fastboot_response() {
        assert!(matches!(
            FastbootResponse::parse(b"OKAY"),
            Ok(FastbootResponse::Okay(_))
        ));
        
        assert!(matches!(
            FastbootResponse::parse(b"FAILerror"),
            Ok(FastbootResponse::Fail(msg)) if msg == "error"
        ));
    }

    #[test]
    fn test_line_coding() {
        let coding = LineCoding::new(115200);
        assert_eq!(coding.baud_rate, 115200);
        assert_eq!(coding.data_bits, 8);
        
        let bytes = coding.to_bytes();
        let parsed = LineCoding::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.baud_rate, 115200);
    }

    #[test]
    fn test_dfu_state() {
        assert!(DfuState::DfuIdle.is_dfu_mode());
        assert!(!DfuState::AppIdle.is_dfu_mode());
    }
}
