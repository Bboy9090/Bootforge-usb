//! # libbootforge
//!
//! Low-level, read-only-first forensic USB detection, identity correlation, protocol
//! classification, health reporting, and event intelligence.

pub mod detect;
pub mod error;
pub mod forensic;
pub mod identity;
pub mod session;
pub mod types;

// Backward-compatible modules.
pub mod descriptors;
pub mod device;
pub mod enumeration;
pub mod events;

pub use detect::scanner::scan_devices;
pub use error::{BootforgeError, Result};
pub use forensic::{ForensicEvent, ForensicEventKind, ObservationSource};
pub use identity::{
    correlate_reconnect, DeviceIdentity, IdentityConfidence, IdentityEvidence, ReconnectMatch,
};
pub use types::{
    DeviceFamily, DeviceFingerprint, DeviceInfo, DeviceMode, DevicePlatform, DeviceTransport,
    FingerprintConfidence, WorkflowRecommendation,
};

// Legacy exports retained for existing consumers.
pub use descriptors::DeviceDescriptor;
pub use enumeration::enumerate_devices;
pub use events::DeviceEventMonitor;

/// Scan devices and return a deterministic pretty-printed JSON document.
pub fn scan_devices_json() -> Result<String> {
    let devices = scan_devices()?;
    serde_json::to_string_pretty(&devices)
        .map_err(|error| BootforgeError::JsonSerializationFailed(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_api_exports_core_forensic_types() {
        let _device: Option<DeviceInfo> = None;
        let _identity: Option<DeviceIdentity> = None;
        let _event: Option<ForensicEvent> = None;
        let _match: Option<ReconnectMatch> = None;
    }
}
