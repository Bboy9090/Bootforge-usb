//! Tests for device classification logic

use libbootforge::{
    detect::classifier::{classify_mode, classify_platform, classify_transport, classify_vendor_name},
    DeviceMode, DevicePlatform, DeviceTransport,
};

#[test]
fn test_apple_dfu_classification() {
    let mode = classify_mode(0x05ac, 0x1227);
    assert_eq!(mode, DeviceMode::Dfu);
}

#[test]
fn test_apple_recovery_classification() {
    let mode = classify_mode(0x05ac, 0x1281);
    assert_eq!(mode, DeviceMode::Recovery);
}

#[test]
fn test_apple_platform_classification() {
    let platform = classify_platform(0x05ac);
    assert_eq!(platform, DevicePlatform::Apple);
}

#[test]
fn test_google_fastboot_classification() {
    let mode = classify_mode(0x18d1, 0x4ee7);
    assert_eq!(mode, DeviceMode::Fastboot);
}

#[test]
fn test_usb3_transport_classification() {
    let transport = classify_transport(0x0300);
    assert_eq!(transport, DeviceTransport::Usb3);
}

#[test]
fn test_usb2_transport_classification() {
    let transport = classify_transport(0x0200);
    assert_eq!(transport, DeviceTransport::Usb2);
}

#[test]
fn test_apple_vendor_name_resolution() {
    let vendor = classify_vendor_name(0x05ac);
    assert_eq!(vendor, Some("Apple".to_string()));
}

#[test]
fn test_google_vendor_name_resolution() {
    let vendor = classify_vendor_name(0x18d1);
    assert_eq!(vendor, Some("Google".to_string()));
}

#[test]
fn test_samsung_vendor_name_resolution() {
    let vendor = classify_vendor_name(0x04e8);
    assert_eq!(vendor, Some("Samsung".to_string()));
}

#[test]
fn test_unknown_vendor() {
    let vendor = classify_vendor_name(0x9999);
    assert_eq!(vendor, None);
}

#[test]
fn test_android_platform_classification() {
    assert_eq!(classify_platform(0x18d1), DevicePlatform::Android);
    assert_eq!(classify_platform(0x04e8), DevicePlatform::Android);
    assert_eq!(classify_platform(0x2717), DevicePlatform::Android);
}

#[test]
fn test_generic_usb_classification() {
    assert_eq!(classify_platform(0x0781), DevicePlatform::GenericUsb);
    assert_eq!(classify_platform(0x046d), DevicePlatform::GenericUsb);
}

#[test]
fn test_samsung_adb_mode() {
    let mode = classify_mode(0x04e8, 0x6860);
    assert_eq!(mode, DeviceMode::Adb);
}

#[test]
fn test_samsung_bootloader_mode() {
    let mode = classify_mode(0x04e8, 0x685d);
    assert_eq!(mode, DeviceMode::Bootloader);
}

#[test]
fn test_sandisk_mass_storage() {
    let mode = classify_mode(0x0781, 0x1234);
    assert_eq!(mode, DeviceMode::MassStorage);
}
