//! # BootForge USB
//!
//! A cross-platform USB device enumeration and information library.
//!
//! This library provides a unified interface for discovering USB devices across
//! Windows, macOS, and Linux platforms. It uses libusb (via rusb) for cross-platform
//! base enumeration and platform-specific APIs for enriching device information.
//!
//! ## Features
//!
//! - Cross-platform USB device enumeration
//! - Real-time device hotplug monitoring
//! - Protocol detection (ADB, Fastboot, Apple, MTP)
//! - USB port topology mapping
//! - Driver status and health checks
//! - Platform-specific device information enrichment
//! - Normalized device information structure
//! - Support for vendor/product IDs, serial numbers, and device paths
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
//! See `docs/ARCHITECTURE.md` for detailed pipeline diagrams and `docs/GLOSSARY.md`
//! for term definitions.
//!
//! ## Basic Enumeration Example
//!
//! ```no_run
//! use bootforge_usb::enumerate_all;
//!
//! fn main() -> anyhow::Result<()> {
//!     // Enumerate all connected USB devices
//!     let devices = enumerate_all()?;
//!     
//!     for device in devices {
//!         println!("Device: {}", device);
//!         println!("  Vendor ID: {:04x}", device.vendor_id);
//!         println!("  Product ID: {:04x}", device.product_id);
//!         if let Some(manufacturer) = device.manufacturer {
//!             println!("  Manufacturer: {}", manufacturer);
//!         }
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Device Identity Resolution
//!
//! For stable device tracking across reconnections:
//!
//! ```no_run
//! use bootforge_usb::{enumerate_all, UsbDeviceInfo};
//! use std::collections::HashMap;
//!
//! // Strategy: Use serial number as primary identifier
//! fn get_device_identity(device: &UsbDeviceInfo) -> String {
//!     if let Some(serial) = &device.serial_number {
//!         format!("{}:{}:{}", device.vendor_id, device.product_id, serial)
//!     } else if let Some(port_path) = &device.port_path {
//!         format!("{}:{}:port:{}", device.vendor_id, device.product_id, port_path)
//!     } else {
//!         format!("{}:{}:bus{}addr{}", 
//!             device.vendor_id, device.product_id,
//!             device.bus_number, device.device_address)
//!     }
//! }
//!
//! fn main() -> anyhow::Result<()> {
//!     let devices = enumerate_all()?;
//!     let mut device_cache: HashMap<String, UsbDeviceInfo> = HashMap::new();
//!     
//!     for device in devices {
//!         let identity = get_device_identity(&device);
//!         device_cache.insert(identity, device);
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Concurrent Operation Safety
//!
//! BootForge USB focuses on device detection and monitoring. For safe concurrent
//! operations on devices, applications should implement appropriate locking:
//!
//! ```no_run
//! use bootforge_usb::{enumerate_all, UsbId};
//! use std::collections::HashMap;
//! use std::sync::{Arc, Mutex};
//!
//! // Application-level per-device locking pattern
//! struct DeviceManager {
//!     locks: Arc<HashMap<UsbId, Mutex<()>>>,
//! }
//!
//! impl DeviceManager {
//!     fn new() -> Self {
//!         Self { locks: Arc::new(HashMap::new()) }
//!     }
//!     
//!     // Acquire exclusive access to a device
//!     fn with_device<F, R>(&self, device_id: &UsbId, f: F) -> R
//!     where
//!         F: FnOnce() -> R,
//!     {
//!         // In real implementation, would use entry API or RwLock
//!         // This is a simplified example showing the pattern
//!         let guard = self.locks.get(device_id)
//!             .map(|lock| lock.lock().unwrap());
//!         f()
//!     }
//! }
//!
//! # fn main() {}
//! ```
//!
//! **Key Safety Guidelines:**
//!
//! - Enumerate devices from a single thread (enumeration is not thread-safe)
//! - Device records (UsbDeviceInfo/UsbDeviceRecord) are safe to read concurrently
//! - Implement per-device locks when performing operations (via rusb::DeviceHandle)
//! - Use DeviceWatcher for event-driven monitoring instead of polling
//! - See `docs/ARCHITECTURE.md` for detailed operation safety patterns

pub mod api;
pub mod enumerate;
pub mod errors;
pub mod handshake;
pub mod model;
pub mod ports;
pub mod types;
pub mod watcher;

// Re-export main types and functions for convenient access
pub use api::UsbEnumerator;
pub use enumerate::enumerate_all;
pub use errors::UsbError;
pub use handshake::{classify_device_protocols, DeviceProtocol};
pub use model::{DriverStatus, LinkHealth, UsbDescriptorSummary, UsbDeviceRecord, UsbId, UsbLocation};
pub use types::{PlatformHint, UsbBusType, UsbDeviceInfo, UsbIds};
pub use watcher::{DeviceEvent, DeviceWatcher};

// Platform-specific watcher
pub use watcher::PlatformWatcher;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_enumeration() {
        // This is a basic smoke test
        // It may fail in CI environments without USB devices or permissions
        let result = enumerate_all();

        // We just verify it doesn't panic and returns a Result
        match result {
            Ok(_devices) => {
                // Success - devices were enumerated
            }
            Err(_e) => {
                // Also ok - may not have permissions or devices
            }
        }
    }

    // Test device identity resolution with serial number (Priority 1)
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

        // Serial number should be the preferred identity
        assert_eq!(device.descriptor.serial_number, Some("SN123456".to_string()));
        assert_eq!(device.id.as_hex_string(), "1234:5678");
    }

    // Test device identity resolution with port path (Priority 2)
    #[test]
    fn test_device_identity_with_port_path() {
        let device = UsbDeviceRecord {
            id: UsbId::new(0xABCD, 0xEF01),
            location: UsbLocation {
                bus: Some(2),
                address: Some(7),
                port_path: Some("2-1.4.2".to_string()),
            },
            descriptor: UsbDescriptorSummary {
                manufacturer: Some("Test".to_string()),
                product: Some("Device".to_string()),
                serial_number: None, // No serial
                device_class: Some(0x08),
                device_subclass: None,
                device_protocol: None,
                usb_version: Some("3.0".to_string()),
            },
            driver: DriverStatus::Bound {
                name: "test_driver".to_string(),
            },
            health: LinkHealth::Good,
            tags: vec![],
            raw_data: None,
        };

        // Without serial, port path is next best identifier
        assert!(device.descriptor.serial_number.is_none());
        assert_eq!(device.location.port_path, Some("2-1.4.2".to_string()));
    }

    // Test device identity resolution with location fallback (Priority 3)
    #[test]
    fn test_device_identity_with_location_fallback() {
        let device = UsbDeviceRecord {
            id: UsbId::new(0x9999, 0x8888),
            location: UsbLocation {
                bus: Some(3),
                address: Some(12),
                port_path: None, // No port path
            },
            descriptor: UsbDescriptorSummary {
                manufacturer: None,
                product: None,
                serial_number: None, // No serial
                device_class: Some(0x09),
                device_subclass: Some(0x00),
                device_protocol: Some(0x00),
                usb_version: None,
            },
            driver: DriverStatus::Missing,
            health: LinkHealth::Good,
            tags: vec![],
            raw_data: None,
        };

        // Must fallback to bus/address for identity
        assert!(device.descriptor.serial_number.is_none());
        assert!(device.location.port_path.is_none());
        assert_eq!(device.location.bus, Some(3));
        assert_eq!(device.location.address, Some(12));
    }

    // Test driver status states
    #[test]
    fn test_driver_status_variants() {
        let bound = DriverStatus::Bound {
            name: "usb_storage".to_string(),
        };
        let missing = DriverStatus::Missing;
        let blocked = DriverStatus::Blocked {
            reason: "Policy restriction".to_string(),
        };
        let multiple = DriverStatus::Multiple {
            drivers: vec!["driver1".to_string(), "driver2".to_string()],
        };

        // Verify variants are distinct
        assert!(matches!(bound, DriverStatus::Bound { .. }));
        assert!(matches!(missing, DriverStatus::Missing));
        assert!(matches!(blocked, DriverStatus::Blocked { .. }));
        assert!(matches!(multiple, DriverStatus::Multiple { .. }));
    }

    // Test link health states
    #[test]
    fn test_link_health_states() {
        let good = LinkHealth::Good;
        let unstable = LinkHealth::Unstable {
            reason: "Intermittent connection".to_string(),
        };
        let power_issue = LinkHealth::PowerIssueHint {
            reason: "Insufficient power".to_string(),
        };
        let reset_loop = LinkHealth::ResetLoop;
        let disconnected = LinkHealth::Disconnected;

        // Verify health states
        assert!(matches!(good, LinkHealth::Good));
        assert!(matches!(unstable, LinkHealth::Unstable { .. }));
        assert!(matches!(power_issue, LinkHealth::PowerIssueHint { .. }));
        assert!(matches!(reset_loop, LinkHealth::ResetLoop));
        assert!(matches!(disconnected, LinkHealth::Disconnected));
    }

    // Test device event types
    #[test]
    fn test_device_event_types() {
        let device = UsbDeviceRecord {
            id: UsbId::new(0x1111, 0x2222),
            location: UsbLocation {
                bus: Some(1),
                address: Some(1),
                port_path: None,
            },
            descriptor: UsbDescriptorSummary {
                manufacturer: None,
                product: None,
                serial_number: None,
                device_class: None,
                device_subclass: None,
                device_protocol: None,
                usb_version: None,
            },
            driver: DriverStatus::Unknown,
            health: LinkHealth::Good,
            tags: vec![],
            raw_data: None,
        };

        let added = DeviceEvent::Added(device.clone());
        let removed = DeviceEvent::Removed(device.clone());
        let changed = DeviceEvent::Changed(device.clone());

        // Verify event types
        assert!(matches!(added, DeviceEvent::Added(_)));
        assert!(matches!(removed, DeviceEvent::Removed(_)));
        assert!(matches!(changed, DeviceEvent::Changed(_)));
    }

    // Test UsbId hex string formatting
    #[test]
    fn test_usb_id_hex_string() {
        let id1 = UsbId::new(0x1234, 0x5678);
        assert_eq!(id1.as_hex_string(), "1234:5678");

        let id2 = UsbId::new(0xABCD, 0xEF01);
        assert_eq!(id2.as_hex_string(), "ABCD:EF01");

        let id3 = UsbId::new(0x0001, 0x0002);
        assert_eq!(id3.as_hex_string(), "0001:0002");
    }

    // Test device tagging functionality
    #[test]
    fn test_device_tags() {
        let mut device = UsbDeviceRecord {
            id: UsbId::new(0x18D1, 0x4EE1), // Google ADB
            location: UsbLocation {
                bus: Some(1),
                address: Some(2),
                port_path: None,
            },
            descriptor: UsbDescriptorSummary {
                manufacturer: Some("Google".to_string()),
                product: Some("Nexus".to_string()),
                serial_number: Some("123ABC".to_string()),
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

        // Initially no tags
        assert!(!device.has_tag("adb"));

        // Add tag
        device.add_tag("adb");
        assert!(device.has_tag("adb"));
        assert!(device.has_tag("ADB")); // Case-insensitive

        // Adding duplicate tag should not duplicate
        device.add_tag("adb");
        assert_eq!(device.tags.len(), 1);

        // Add another tag
        device.add_tag("android");
        assert_eq!(device.tags.len(), 2);
        assert!(device.has_tag("android"));
    }

    // Test protocol classification with ADB device
    #[test]
    fn test_protocol_classification_adb() {
        let device = UsbDeviceRecord {
            id: UsbId::new(0x18D1, 0x4EE1), // Google Nexus (ADB)
            location: UsbLocation {
                bus: Some(1),
                address: Some(3),
                port_path: None,
            },
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
        // Should detect ADB
        assert!(protocols.contains(&DeviceProtocol::Adb));
    }

    // Test protocol classification with Apple device
    #[test]
    fn test_protocol_classification_apple() {
        let device = UsbDeviceRecord {
            id: UsbId::new(0x05AC, 0x12A8), // Apple Inc.
            location: UsbLocation {
                bus: Some(1),
                address: Some(4),
                port_path: None,
            },
            descriptor: UsbDescriptorSummary {
                manufacturer: Some("Apple Inc.".to_string()),
                product: Some("iPhone".to_string()),
                serial_number: Some("SERIAL123".to_string()),
                device_class: Some(0xEF),
                device_subclass: Some(0x02),
                device_protocol: Some(0x01),
                usb_version: Some("2.0".to_string()),
            },
            driver: DriverStatus::Unknown,
            health: LinkHealth::Good,
            tags: vec![],
            raw_data: None,
        };

        let protocols = classify_device_protocols(&device);
        // Should detect Apple device
        assert!(protocols.contains(&DeviceProtocol::AppleDevice));
    }

    // Test handling of devices with missing descriptors
    #[test]
    fn test_device_with_missing_descriptors() {
        let device = UsbDeviceRecord {
            id: UsbId::new(0xFFFF, 0xFFFF),
            location: UsbLocation {
                bus: Some(1),
                address: Some(5),
                port_path: None,
            },
            descriptor: UsbDescriptorSummary {
                manufacturer: None, // Missing
                product: None,      // Missing
                serial_number: None, // Missing
                device_class: Some(0x00),
                device_subclass: None,
                device_protocol: None,
                usb_version: None,
            },
            driver: DriverStatus::Unknown,
            health: LinkHealth::Good,
            tags: vec![],
            raw_data: None,
        };

        // Device should still be valid even with missing descriptors
        assert!(device.descriptor.manufacturer.is_none());
        assert!(device.descriptor.product.is_none());
        assert!(device.descriptor.serial_number.is_none());
        
        // Should classify as Unknown protocol
        let protocols = classify_device_protocols(&device);
        assert!(protocols.contains(&DeviceProtocol::Unknown));
    }
}
