//! Session logging for device events

use crate::session::history::{current_timestamp_string, DeviceEvent};
use serde::{Deserialize, Serialize};

/// Session log containing device events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionLog {
    pub session_id: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub events: Vec<DeviceEvent>,
}

impl SessionLog {
    /// Create a new session log
    pub fn new() -> Self {
        let timestamp = current_timestamp_string();
        let session_id = format!("session_{}", timestamp.replace([':', '-', '.'], ""));

        SessionLog {
            session_id,
            started_at: timestamp,
            ended_at: None,
            events: Vec::new(),
        }
    }

    /// Close the session log
    pub fn close(&mut self) {
        self.ended_at = Some(current_timestamp_string());
    }

    /// Add an event to the session log
    pub fn add_event(&mut self, event: DeviceEvent) {
        self.events.push(event);
    }
}

impl Default for SessionLog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::history::{create_device_event, DeviceEventType};
    use crate::types::{
        DeviceFamily, DeviceFingerprint, DeviceInfo, DeviceMode, DevicePlatform, DeviceTransport,
        FingerprintConfidence, WorkflowRecommendation,
    };

    fn create_test_device() -> DeviceInfo {
        DeviceInfo {
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
        }
    }

    #[test]
    fn test_new_session_log() {
        let log = SessionLog::new();
        assert!(log.session_id.starts_with("session_"));
        assert!(!log.started_at.is_empty());
        assert!(log.ended_at.is_none());
        assert_eq!(log.events.len(), 0);
    }

    #[test]
    fn test_close_session_log() {
        let mut log = SessionLog::new();
        assert!(log.ended_at.is_none());
        log.close();
        assert!(log.ended_at.is_some());
    }

    #[test]
    fn test_add_event() {
        let mut log = SessionLog::new();
        let device = create_test_device();
        let event = create_device_event(DeviceEventType::Connected, device);

        log.add_event(event);
        assert_eq!(log.events.len(), 1);
    }
}
