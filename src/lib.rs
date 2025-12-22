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
//! ## Example
//!
//! ```no_run
//! use bootforge_usb::enumerate_all;
//!
//! fn main() -> anyhow::Result<()> {
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
pub use model::{DriverStatus, LinkHealth, UsbDeviceRecord, UsbId, UsbLocation};
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
}
