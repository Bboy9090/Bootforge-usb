//! # libbootforge
//!
//! Low-level, read-only-first forensic USB detection, identity correlation, protocol
//! classification, driver visibility, health reporting, and event intelligence.

pub mod detect;
pub mod driver;
pub mod driver_watch;
pub mod error;
pub mod forensic;
pub mod health;
pub mod identity;
pub mod native_driver;
pub mod notification;
pub mod protocol;
pub mod session;
pub mod types;

// Backward-compatible modules.
pub mod descriptors;
pub mod device;
pub mod enumeration;
pub mod events;

pub use detect::scanner::scan_devices;
pub use driver::{
    ArcwyreDriverInspector, DriverBackend, DriverConfidence, DriverEvidence, DriverInspector,
    DriverReport, DriverState, LinuxDriverInspector, MacOsDriverInspector,
    WindowsDriverInspector,
};
pub use driver_watch::{
    DriverChange, DriverChangeField, DriverChangeMonitor, DriverStateTracker,
};
pub use error::{BootforgeError, Result};
pub use events::ForensicEventMonitor;
pub use forensic::{ForensicEvent, ForensicEventKind, ObservationSource};
pub use health::{HealthReport, HealthSignal, HealthState, HealthTracker};
pub use identity::{
    correlate_reconnect, DeviceIdentity, IdentityConfidence, IdentityEvidence, ReconnectMatch,
};
pub use native_driver::inspect_platform_driver;
pub use notification::{
    NotificationSignal, NotificationWake, PollingWake, WakeReason, WakeResult,
};
pub use protocol::{
    ProtocolConfidence, ProtocolEvidence, ProtocolObservation, ProtocolReport, UsbProtocol,
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
        let _protocol: Option<ProtocolReport> = None;
        let _driver: Option<DriverReport> = None;
        let _driver_change: Option<DriverChange> = None;
        let _driver_tracker: Option<DriverStateTracker> = None;
        let _driver_monitor: Option<DriverChangeMonitor> = None;
        let _wake_signal: Option<NotificationSignal> = None;
        let _wake_result: Option<WakeResult> = None;
        let _health: Option<HealthReport> = None;
        let _health_tracker: Option<HealthTracker> = None;
        let _watcher: Option<ForensicEventMonitor> = None;
        let _router: fn(&DeviceInfo) -> DriverReport = inspect_platform_driver;
    }
}
