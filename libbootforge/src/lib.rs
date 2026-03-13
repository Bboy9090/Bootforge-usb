//! # libbootforge
//!
//! libbootforge is a low-level USB device detection library designed for hardware discovery,
//! repair workflows, and device preparation.
//!
//! The library provides structured access to USB device information including:
//! - vendor and product identifiers
//! - device descriptors
//! - device mode detection
//! - device connection events
//!
//! libbootforge serves as the USB hardware discovery layer for the Bobby's Workshop device ecosystem.

use thiserror::Error;

pub mod device;
pub mod descriptors;
pub mod enumeration;
pub mod events;

pub use device::{DeviceInfo, DeviceMode};
pub use descriptors::DeviceDescriptor;
pub use enumeration::enumerate_devices;
pub use events::DeviceEventMonitor;

/// Error types for libbootforge operations
#[derive(Error, Debug)]
pub enum BootforgeError {
    #[error("USB error: {0}")]
    UsbError(#[from] rusb::Error),

    #[error("Device not found")]
    DeviceNotFound,

    #[error("Invalid descriptor")]
    InvalidDescriptor,

    #[error("Platform not supported")]
    PlatformNotSupported,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, BootforgeError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_exports() {
        // Verify all main types are exported
        let _info: Option<DeviceInfo> = None;
        let _mode: Option<DeviceMode> = None;
        let _desc: Option<DeviceDescriptor> = None;
    }
}
