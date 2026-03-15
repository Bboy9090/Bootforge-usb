//! Tests for known device profiles

use libbootforge::{
    detect::profiles::{known_device_profiles, match_known_profile},
    types::{
        DeviceFamily, DeviceFingerprint, DeviceInfo, DeviceMode, DevicePlatform, DeviceTransport,
        FingerprintConfidence, WorkflowRecommendation,
    },
};

fn create_test_device(vendor_id: u16, product_id: u16) -> DeviceInfo {
    DeviceInfo {
        bus_number: 1,
        address: 2,
        vendor_id,
        product_id,
        vendor_name: None,
        manufacturer: None,
        product_name: None,
        serial_number: None,
        platform: DevicePlatform::Unknown,
        transport: DeviceTransport::Usb2,
        mode: DeviceMode::Unknown,
        fingerprint: DeviceFingerprint {
            family: DeviceFamily::Unknown,
            model_hint: None,
            confidence: FingerprintConfidence::Unknown,
        },
        recommended_workflow: WorkflowRecommendation::Unknown,
        matched_profile: None,
    }
}

#[test]
fn test_known_profiles_exist() {
    let profiles = known_device_profiles();
    assert!(!profiles.is_empty());
    assert!(profiles.len() >= 9);
}

#[test]
fn test_apple_dfu_profile_match() {
    let device = create_test_device(0x05ac, 0x1227);
    let profile = match_known_profile(&device);
    assert!(profile.is_some());
    let profile = profile.unwrap();
    assert_eq!(profile.display_name, "Apple DFU Device");
    assert_eq!(profile.expected_platform, DevicePlatform::Apple);
    assert_eq!(profile.expected_mode, Some(DeviceMode::Dfu));
    assert_eq!(
        profile.expected_workflow,
        Some(WorkflowRecommendation::AppleDfuWorkflow)
    );
}

#[test]
fn test_apple_recovery_profile_match() {
    let device = create_test_device(0x05ac, 0x1281);
    let profile = match_known_profile(&device);
    assert!(profile.is_some());
    let profile = profile.unwrap();
    assert_eq!(profile.display_name, "Apple Recovery Device");
    assert_eq!(profile.expected_mode, Some(DeviceMode::Recovery));
    assert_eq!(
        profile.expected_workflow,
        Some(WorkflowRecommendation::AppleRecoveryWorkflow)
    );
}

#[test]
fn test_google_fastboot_profile_match() {
    let device = create_test_device(0x18d1, 0x4ee7);
    let profile = match_known_profile(&device);
    assert!(profile.is_some());
    let profile = profile.unwrap();
    assert_eq!(profile.display_name, "Google Fastboot Device");
    assert_eq!(profile.expected_platform, DevicePlatform::Android);
    assert_eq!(profile.expected_mode, Some(DeviceMode::Fastboot));
}

#[test]
fn test_sandisk_profile_match() {
    let device = create_test_device(0x0781, 0x1234);
    let profile = match_known_profile(&device);
    assert!(profile.is_some());
    let profile = profile.unwrap();
    assert_eq!(profile.display_name, "SanDisk USB Storage");
    assert_eq!(profile.expected_platform, DevicePlatform::GenericUsb);
    assert_eq!(profile.expected_mode, Some(DeviceMode::MassStorage));
}

#[test]
fn test_samsung_adb_profile() {
    let device = create_test_device(0x04e8, 0x6860);
    let profile = match_known_profile(&device);
    assert!(profile.is_some());
    let profile = profile.unwrap();
    assert_eq!(profile.display_name, "Samsung ADB Device");
}

#[test]
fn test_unknown_device_no_match() {
    let device = create_test_device(0x9999, 0x9999);
    let profile = match_known_profile(&device);
    assert!(profile.is_none());
}

#[test]
fn test_profile_has_notes() {
    let profiles = known_device_profiles();
    let apple_dfu = profiles
        .iter()
        .find(|p| p.vendor_id == 0x05ac && p.product_id == Some(0x1227))
        .unwrap();
    assert!(apple_dfu.notes.is_some());
}
