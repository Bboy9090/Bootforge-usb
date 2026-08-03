//! # libbootforge
//!
//! Low-level, read-only-first forensic USB detection, identity correlation, protocol
//! classification, driver visibility, health reporting, and event intelligence.

pub mod anomaly;
pub mod composite;
pub mod descriptor_intelligence;
pub mod detect;
pub mod driver;
pub mod driver_watch;
pub mod error;
pub mod fingerprint;
pub mod forensic;
pub mod health;
pub mod identity;
pub mod inventory;
pub mod lifetime;
pub mod native_driver;
pub mod notification;
pub mod performance;
pub mod protocol;
pub mod recorder;
pub mod session;
pub mod topology;
pub mod types;

pub mod descriptors;
pub mod device;
pub mod enumeration;
pub mod events;

pub use anomaly::{analyze_passively, AnomalyFinding, AnomalyKind, AnomalySeverity};
pub use composite::{CompositeInterface, CompositeReport};
pub use descriptor_intelligence::{
    DecodedDescriptor, DescriptorIssue, DescriptorKind, DescriptorSnapshot, MAX_DESCRIPTOR_BYTES,
};
pub use detect::scanner::scan_devices;
pub use driver::{
    ArcwyreDriverInspector, DriverBackend, DriverConfidence, DriverEvidence, DriverInspector,
    DriverReport, DriverState, LinuxDriverInspector, MacOsDriverInspector, WindowsDriverInspector,
};
pub use driver_watch::{DriverChange, DriverChangeField, DriverChangeMonitor, DriverStateTracker};
pub use error::{BootforgeError, Result};
pub use events::ForensicEventMonitor;
pub use fingerprint::{
    ForensicFingerprint, ForensicFingerprintConfidence, ForensicFingerprintEvidence,
    FORENSIC_FINGERPRINT_SCHEMA_VERSION,
};
pub use forensic::{ForensicEvent, ForensicEventKind, ObservationSource};
pub use health::{HealthReport, HealthSignal, HealthState, HealthTracker};
pub use identity::{
    correlate_reconnect, DeviceIdentity, IdentityConfidence, IdentityEvidence, ReconnectMatch,
};
pub use inventory::{EventInventory, InventorySnapshot};
pub use lifetime::{DeviceLifetime, LifetimeTracker};
pub use native_driver::inspect_platform_driver;
pub use notification::{NotificationSignal, NotificationWake, PollingWake, WakeReason, WakeResult};
pub use performance::{BoundedEventQueue, PerformanceMetrics};
pub use protocol::{
    ProtocolConfidence, ProtocolEvidence, ProtocolObservation, ProtocolReport, UsbProtocol,
};
pub use recorder::{
    verify_session, EvidenceEnvelope, SessionRecorder, VerificationReport, RECORD_SCHEMA_VERSION,
};
pub use topology::{TopologyNode, TopologyNodeKind, TopologyPath, TopologySnapshot};
pub use types::{
    DeviceFamily, DeviceFingerprint, DeviceInfo, DeviceMode, DevicePlatform, DeviceTransport,
    FingerprintConfidence, WorkflowRecommendation,
};

pub use descriptors::DeviceDescriptor;
pub use enumeration::enumerate_devices;
pub use events::DeviceEventMonitor;

pub fn scan_devices_json() -> Result<String> {
    let devices = scan_devices()?;
    serde_json::to_string_pretty(&devices)
        .map_err(|error| BootforgeError::JsonSerializationFailed(error.to_string()))
}

pub fn scan_topology() -> Result<TopologySnapshot> {
    Ok(TopologySnapshot::from_devices(&scan_devices()?))
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
        let _envelope: Option<EvidenceEnvelope> = None;
        let _verification: Option<VerificationReport> = None;
        let _topology: Option<TopologySnapshot> = None;
        let _lifetime: Option<DeviceLifetime> = None;
        let _fingerprint: Option<ForensicFingerprint> = None;
        let _descriptor_snapshot: Option<DescriptorSnapshot> = None;
        let _composite: Option<CompositeReport> = None;
        let _finding: Option<AnomalyFinding> = None;
        let _metrics: Option<PerformanceMetrics> = None;
        let _inventory: Option<EventInventory> = None;
        let _inventory_snapshot: Option<InventorySnapshot> = None;
        let _router: fn(&DeviceInfo) -> DriverReport = inspect_platform_driver;
    }
}
