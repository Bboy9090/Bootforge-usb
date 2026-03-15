//! Device event history tracking

use crate::types::DeviceInfo;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Type of device event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeviceEventType {
    Connected,
    Disconnected,
    Rescanned,
}

/// Device event with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceEvent {
    pub timestamp: String,
    pub event_type: DeviceEventType,
    pub device: DeviceInfo,
}

/// Get current timestamp as string
pub fn current_timestamp_string() -> String {
    let now: DateTime<Utc> = Utc::now();
    now.to_rfc3339()
}

/// Create a device event with current timestamp
pub fn create_device_event(event_type: DeviceEventType, device: DeviceInfo) -> DeviceEvent {
    DeviceEvent {
        timestamp: current_timestamp_string(),
        event_type,
        device,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        DeviceFamily, DeviceFingerprint, DeviceMode, DevicePlatform, DeviceTransport,
        FingerprintConfidence, WorkflowRecommendation,
    };

    #[test]
    fn test_create_device_event() {
        let device = DeviceInfo {
            bus_number: 1,
            address: 2,
            vendor_id: 0x05ac,
            product_id: 0x1227,
            vendor_name: Some("Apple".to_string()),
            manufacturer: Some("Apple Inc.".to_string()),
            product_name: Some("iPhone".to_string()),
            serial_number: None,
            platform: DevicePlatform::Apple,
            transport: DeviceTransport::Usb2,
            mode: DeviceMode::Dfu,
            fingerprint: DeviceFingerprint {
                family: DeviceFamily::IPhone,
                model_hint: Some("iPhone".to_string()),
                confidence: FingerprintConfidence::High,
            },
            recommended_workflow: WorkflowRecommendation::AppleDfuWorkflow,
            matched_profile: Some("Apple DFU Device".to_string()),
        };

        let event = create_device_event(DeviceEventType::Connected, device);
        assert_eq!(event.event_type, DeviceEventType::Connected);
        assert!(!event.timestamp.is_empty());
    }

    #[test]
    fn test_timestamp_generation() {
        let ts1 = current_timestamp_string();
        let ts2 = current_timestamp_string();
        assert!(!ts1.is_empty());
        assert!(!ts2.is_empty());
    }
}
