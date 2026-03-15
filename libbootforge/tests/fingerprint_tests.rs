//! Tests for device fingerprinting and workflow recommendation

use libbootforge::{
    detect::fingerprint::{fingerprint_device, recommend_workflow},
    DeviceFamily, DeviceFingerprint, DeviceMode, DevicePlatform, FingerprintConfidence,
    WorkflowRecommendation,
};

#[test]
fn test_apple_dfu_fingerprint() {
    let fp = fingerprint_device(
        0x05ac,
        0x1227,
        Some("iPhone"),
        Some("Apple Inc."),
        &DeviceMode::Dfu,
    );
    assert_eq!(fp.family, DeviceFamily::IPhone);
    assert_eq!(fp.confidence, FingerprintConfidence::High);
}

#[test]
fn test_apple_recovery_workflow() {
    let fp = DeviceFingerprint {
        family: DeviceFamily::IPhone,
        model_hint: Some("iPhone".to_string()),
        confidence: FingerprintConfidence::High,
    };
    let workflow = recommend_workflow(&DevicePlatform::Apple, &DeviceMode::Recovery, &fp);
    assert_eq!(workflow, WorkflowRecommendation::AppleRecoveryWorkflow);
}

#[test]
fn test_apple_dfu_workflow() {
    let fp = DeviceFingerprint {
        family: DeviceFamily::IPhone,
        model_hint: Some("iPhone".to_string()),
        confidence: FingerprintConfidence::High,
    };
    let workflow = recommend_workflow(&DevicePlatform::Apple, &DeviceMode::Dfu, &fp);
    assert_eq!(workflow, WorkflowRecommendation::AppleDfuWorkflow);
}

#[test]
fn test_google_fastboot() {
    let fp = fingerprint_device(
        0x18d1,
        0x4ee7,
        Some("Android Device"),
        Some("Google"),
        &DeviceMode::Fastboot,
    );
    assert_eq!(fp.family, DeviceFamily::AndroidPhone);
    assert_eq!(fp.confidence, FingerprintConfidence::High);

    let workflow = recommend_workflow(&DevicePlatform::Android, &DeviceMode::Fastboot, &fp);
    assert_eq!(workflow, WorkflowRecommendation::AndroidFastbootWorkflow);
}

#[test]
fn test_sandisk_storage() {
    let fp = fingerprint_device(
        0x0781,
        0x0000,
        Some("USB Flash Drive"),
        Some("SanDisk"),
        &DeviceMode::MassStorage,
    );
    assert_eq!(fp.family, DeviceFamily::UsbStorage);
    assert_eq!(fp.confidence, FingerprintConfidence::High);

    let workflow = recommend_workflow(&DevicePlatform::GenericUsb, &DeviceMode::MassStorage, &fp);
    assert_eq!(workflow, WorkflowRecommendation::MassStorageInspection);
}

#[test]
fn test_logitech_peripheral() {
    let fp = fingerprint_device(
        0x046d,
        0x0000,
        Some("USB Receiver"),
        Some("Logitech"),
        &DeviceMode::Unknown,
    );
    assert_eq!(fp.family, DeviceFamily::Peripheral);

    let workflow = recommend_workflow(&DevicePlatform::GenericUsb, &DeviceMode::Unknown, &fp);
    assert_eq!(
        workflow,
        WorkflowRecommendation::GenericPeripheralInspection
    );
}

#[test]
fn test_android_tablet_fingerprint() {
    let fp = fingerprint_device(
        0x18d1,
        0x0000,
        Some("Android Tablet"),
        Some("Google"),
        &DeviceMode::Adb,
    );
    assert_eq!(fp.family, DeviceFamily::AndroidTablet);
    assert_eq!(fp.confidence, FingerprintConfidence::High);
}

#[test]
fn test_android_adb_workflow() {
    let fp = DeviceFingerprint {
        family: DeviceFamily::AndroidPhone,
        model_hint: Some("Pixel".to_string()),
        confidence: FingerprintConfidence::High,
    };
    let workflow = recommend_workflow(&DevicePlatform::Android, &DeviceMode::Adb, &fp);
    assert_eq!(workflow, WorkflowRecommendation::AndroidAdbWorkflow);
}

#[test]
fn test_ipad_fingerprint() {
    let fp = fingerprint_device(
        0x05ac,
        0x12a8,
        Some("iPad"),
        Some("Apple Inc."),
        &DeviceMode::Normal,
    );
    assert_eq!(fp.family, DeviceFamily::IPad);
    assert_eq!(fp.confidence, FingerprintConfidence::High);
}

#[test]
fn test_apple_normal_inspection() {
    let fp = DeviceFingerprint {
        family: DeviceFamily::IPhone,
        model_hint: Some("iPhone".to_string()),
        confidence: FingerprintConfidence::High,
    };
    let workflow = recommend_workflow(&DevicePlatform::Apple, &DeviceMode::Normal, &fp);
    assert_eq!(workflow, WorkflowRecommendation::AppleNormalInspection);
}

#[test]
fn test_unknown_device() {
    let fp = fingerprint_device(
        0x9999,
        0x9999,
        Some("Unknown Device"),
        Some("Unknown"),
        &DeviceMode::Unknown,
    );
    assert_eq!(fp.family, DeviceFamily::Unknown);
    assert_eq!(fp.confidence, FingerprintConfidence::Unknown);

    let workflow = recommend_workflow(&DevicePlatform::Unknown, &DeviceMode::Unknown, &fp);
    assert_eq!(workflow, WorkflowRecommendation::Unknown);
}
