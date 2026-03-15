//! Tests for device filtering logic

use libbootforge::{scan_devices, DeviceInfo, DeviceMode, DevicePlatform};

/// Helper function to filter by platform
fn filter_by_platform(devices: Vec<DeviceInfo>, platform: DevicePlatform) -> Vec<DeviceInfo> {
    devices
        .into_iter()
        .filter(|d| d.platform == platform)
        .collect()
}

/// Helper function to filter by vendor
fn filter_by_vendor(devices: Vec<DeviceInfo>, vendor_id: u16) -> Vec<DeviceInfo> {
    devices
        .into_iter()
        .filter(|d| d.vendor_id == vendor_id)
        .collect()
}

/// Helper function to filter by mode
fn filter_by_mode(devices: Vec<DeviceInfo>, mode: DeviceMode) -> Vec<DeviceInfo> {
    devices.into_iter().filter(|d| d.mode == mode).collect()
}

#[test]
fn test_filter_by_platform() {
    // Create mock devices
    let devices = vec![];
    let filtered = filter_by_platform(devices, DevicePlatform::Apple);
    assert_eq!(filtered.len(), 0);
}

#[test]
fn test_filter_by_vendor() {
    let devices = vec![];
    let filtered = filter_by_vendor(devices, 0x05ac);
    assert_eq!(filtered.len(), 0);
}

#[test]
fn test_filter_by_mode() {
    let devices = vec![];
    let filtered = filter_by_mode(devices, DeviceMode::Recovery);
    assert_eq!(filtered.len(), 0);
}

#[test]
#[ignore] // Requires USB hardware
fn test_filter_apple_devices() {
    if let Ok(devices) = scan_devices() {
        let apple_devices = filter_by_platform(devices, DevicePlatform::Apple);
        // Just verify it doesn't panic
        let _ = apple_devices.len();
    }
}

#[test]
#[ignore] // Requires USB hardware
fn test_filter_android_devices() {
    if let Ok(devices) = scan_devices() {
        let android_devices = filter_by_platform(devices, DevicePlatform::Android);
        // Just verify it doesn't panic
        let _ = android_devices.len();
    }
}

#[test]
#[ignore] // Requires USB hardware
fn test_filter_recovery_mode() {
    if let Ok(devices) = scan_devices() {
        let recovery_devices = filter_by_mode(devices, DeviceMode::Recovery);
        // Just verify it doesn't panic
        let _ = recovery_devices.len();
    }
}
