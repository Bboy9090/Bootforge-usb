//! Tests for reports and session logging

use libbootforge::{
    session::{
        history::{create_device_event, DeviceEventType},
        log::SessionLog,
        report::create_scan_report,
    },
    types::{
        DeviceFamily, DeviceFingerprint, DeviceInfo, DeviceMode, DevicePlatform, DeviceTransport,
        FingerprintConfidence, WorkflowRecommendation,
    },
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
fn test_create_scan_report_total_devices() {
    let devices = vec![
        create_test_device(),
        create_test_device(),
        create_test_device(),
    ];
    let report = create_scan_report(devices);
    assert_eq!(report.total_devices, 3);
    assert_eq!(report.devices.len(), 3);
}

#[test]
fn test_create_empty_scan_report() {
    let report = create_scan_report(vec![]);
    assert_eq!(report.total_devices, 0);
    assert_eq!(report.devices.len(), 0);
    assert!(!report.generated_at.is_empty());
}

#[test]
fn test_session_log_initialization() {
    let log = SessionLog::new();
    assert!(log.session_id.starts_with("session_"));
    assert!(!log.started_at.is_empty());
    assert!(log.ended_at.is_none());
    assert_eq!(log.events.len(), 0);
}

#[test]
fn test_session_log_add_event() {
    let mut log = SessionLog::new();
    let device = create_test_device();
    let event = create_device_event(DeviceEventType::Connected, device);

    log.add_event(event);
    assert_eq!(log.events.len(), 1);
    assert_eq!(log.events[0].event_type, DeviceEventType::Connected);
}

#[test]
fn test_session_log_close() {
    let mut log = SessionLog::new();
    assert!(log.ended_at.is_none());

    log.close();
    assert!(log.ended_at.is_some());
    assert!(!log.ended_at.as_ref().unwrap().is_empty());
}

#[test]
fn test_session_log_multiple_events() {
    let mut log = SessionLog::new();
    let device = create_test_device();

    let event1 = create_device_event(DeviceEventType::Connected, device.clone());
    let event2 = create_device_event(DeviceEventType::Rescanned, device.clone());
    let event3 = create_device_event(DeviceEventType::Disconnected, device);

    log.add_event(event1);
    log.add_event(event2);
    log.add_event(event3);

    assert_eq!(log.events.len(), 3);
}

#[test]
fn test_device_event_has_timestamp() {
    let device = create_test_device();
    let event = create_device_event(DeviceEventType::Connected, device);
    assert!(!event.timestamp.is_empty());
}

#[test]
fn test_scan_report_preserves_device_info() {
    let device = create_test_device();
    let devices = vec![device.clone()];
    let report = create_scan_report(devices);

    assert_eq!(report.devices[0].vendor_id, device.vendor_id);
    assert_eq!(report.devices[0].product_id, device.product_id);
    assert_eq!(report.devices[0].platform, device.platform);
    assert_eq!(report.devices[0].mode, device.mode);
}
