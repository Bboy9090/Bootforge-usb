//! Evidence-grade, normalized USB event records.

use crate::driver::DriverReport;
use crate::health::HealthReport;
use crate::identity::{DeviceIdentity, IdentityConfidence, IdentityEvidence};
use crate::native_driver::inspect_platform_driver;
use crate::protocol::ProtocolReport;
use crate::types::{DeviceInfo, DeviceMode, DevicePlatform};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ForensicEventKind {
    DeviceObserved,
    DeviceConnected,
    DeviceDisconnected,
    DeviceReconnected,
    DeviceChanged,
    ModeChanged,
    DriverChanged,
    ProtocolObserved,
    HealthChanged,
    EnumerationFailed,
    PermissionDenied,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ObservationSource {
    Libusb,
    WindowsSetupApi,
    WindowsCfgMgr32,
    LinuxSysfs,
    LinuxUdev,
    MacOsIoKit,
    ArcwyreNative,
    ProtocolClassifier,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ForensicEvent {
    pub schema_version: u16,
    pub sequence: u64,
    pub observed_at: DateTime<Utc>,
    pub kind: ForensicEventKind,
    pub source: ObservationSource,
    pub device_id: String,
    pub identity_confidence: IdentityConfidence,
    pub identity_evidence: Vec<IdentityEvidence>,
    pub vendor_id: u16,
    pub product_id: u16,
    pub platform: DevicePlatform,
    pub mode: DeviceMode,
    pub bus_number: u8,
    pub address: u8,
    pub protocol_report: ProtocolReport,
    pub driver_report: DriverReport,
    pub health_report: HealthReport,
    pub message: Option<String>,
}

impl ForensicEvent {
    /// Construct a normalized event and enrich it with passive host-native driver evidence.
    pub fn from_device(
        sequence: u64,
        kind: ForensicEventKind,
        source: ObservationSource,
        device: &DeviceInfo,
        message: Option<String>,
    ) -> Self {
        let identity = DeviceIdentity::from_device(device);
        Self {
            schema_version: 1,
            sequence,
            observed_at: Utc::now(),
            kind,
            source,
            device_id: identity.stable_id,
            identity_confidence: identity.confidence,
            identity_evidence: identity.evidence,
            vendor_id: device.vendor_id,
            product_id: device.product_id,
            platform: device.platform,
            mode: device.mode,
            bus_number: device.bus_number,
            address: device.address,
            protocol_report: ProtocolReport::from_device(device),
            driver_report: inspect_platform_driver(device),
            health_report: HealthReport::unknown(),
            message,
        }
    }

    pub fn with_driver_report(mut self, report: DriverReport) -> Self {
        self.driver_report = report;
        self
    }

    pub fn with_health_report(mut self, report: HealthReport) -> Self {
        self.health_report = report;
        self
    }

    pub fn to_json_line(&self) -> crate::Result<String> {
        serde_json::to_string(self)
            .map_err(|error| crate::BootforgeError::JsonSerializationFailed(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::UsbProtocol;
    use crate::types::{
        DeviceFamily, DeviceFingerprint, DeviceTransport, FingerprintConfidence,
        WorkflowRecommendation,
    };

    fn fixture() -> DeviceInfo {
        DeviceInfo {
            bus_number: 255,
            address: 255,
            vendor_id: 0xffff,
            product_id: 0xffff,
            vendor_name: Some("Test".into()),
            manufacturer: Some("Test".into()),
            product_name: Some("Android ADB".into()),
            serial_number: Some("SERIAL1".into()),
            platform: DevicePlatform::Android,
            transport: DeviceTransport::Usb3,
            mode: DeviceMode::Adb,
            fingerprint: DeviceFingerprint {
                family: DeviceFamily::AndroidPhone,
                model_hint: None,
                confidence: FingerprintConfidence::High,
            },
            recommended_workflow: WorkflowRecommendation::AndroidAdbWorkflow,
            matched_profile: Some("android-adb".into()),
        }
    }

    #[test]
    fn event_contains_identity_protocol_and_driver_intelligence() {
        let event = ForensicEvent::from_device(
            7,
            ForensicEventKind::DeviceConnected,
            ObservationSource::Libusb,
            &fixture(),
            None,
        );
        assert_eq!(event.schema_version, 1);
        assert_eq!(event.sequence, 7);
        assert!(event.device_id.starts_with("bfusb-"));
        assert_eq!(event.identity_confidence, IdentityConfidence::Exact);
        assert!(event
            .protocol_report
            .observations
            .iter()
            .any(|item| item.protocol == UsbProtocol::Adb));
        assert!(!event.protocol_report.active_probe_performed);
        assert!(event.driver_report.message.is_some());
        assert_eq!(event.health_report.score, None);
    }

    #[test]
    fn json_line_is_single_complete_record() {
        let event = ForensicEvent::from_device(
            1,
            ForensicEventKind::DeviceObserved,
            ObservationSource::Libusb,
            &fixture(),
            Some("passive observation".into()),
        );
        let line = event.to_json_line().expect("serialization must succeed");
        assert!(!line.contains('\n'));
        assert!(line.contains("DeviceObserved"));
        assert!(line.contains("protocol_report"));
        assert!(line.contains("driver_report"));
        assert!(line.contains("health_report"));
    }
}
