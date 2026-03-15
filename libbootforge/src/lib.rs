//! # libbootforge
//!
//! libbootforge is a low-level USB device detection library designed for hardware discovery,
//! repair workflows, and device preparation.
//!
//! The library provides structured access to USB device information including:
//! - vendor and product identifiers
//! - device descriptors
//! - device mode detection
//! - device fingerprinting
//! - workflow recommendations
//! - device connection events
//!
//! libbootforge serves as the USB hardware discovery layer for the Bobby's Workshop device ecosystem.

pub mod detect;
pub mod error;
pub mod session;
pub mod types;

// Keep old modules for backward compatibility
pub mod descriptors;
pub mod device;
pub mod enumeration;
pub mod events;

// Re-export main types
pub use detect::scanner::scan_devices;
pub use error::{BootforgeError, Result};
pub use types::{
    DeviceFamily, DeviceFingerprint, DeviceInfo, DeviceMode, DevicePlatform, DeviceTransport,
    FingerprintConfidence, WorkflowRecommendation,
};

// Legacy exports for backward compatibility
pub use descriptors::DeviceDescriptor;
pub use enumeration::enumerate_devices;
pub use events::DeviceEventMonitor;

/// Scan devices and return JSON string
pub fn scan_devices_json() -> Result<String> {
    let devices = scan_devices()?;
    serde_json::to_string_pretty(&devices)
        .map_err(|e| BootforgeError::JsonSerializationFailed(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_exports() {
        // Verify all main types are exported
        let _info: Option<DeviceInfo> = None;
        let _mode: Option<DeviceMode> = None;
        let _platform: Option<DevicePlatform> = None;
        let _family: Option<DeviceFamily> = None;
    }
}
