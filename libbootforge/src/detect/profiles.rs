//! Known device profiles for common devices

use crate::types::{DeviceInfo, DeviceMode, DevicePlatform, WorkflowRecommendation};
use serde::{Deserialize, Serialize};

/// Known device profile for identification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KnownDeviceProfile {
    pub vendor_id: u16,
    pub product_id: Option<u16>,
    pub display_name: String,
    pub notes: Option<String>,
    pub expected_platform: DevicePlatform,
    pub expected_mode: Option<DeviceMode>,
    pub expected_workflow: Option<WorkflowRecommendation>,
}

/// Get list of built-in known device profiles
pub fn known_device_profiles() -> Vec<KnownDeviceProfile> {
    vec![
        // Apple DFU Device
        KnownDeviceProfile {
            vendor_id: 0x05ac,
            product_id: Some(0x1227),
            display_name: "Apple DFU Device".to_string(),
            notes: Some("iPhone/iPad in Device Firmware Update mode".to_string()),
            expected_platform: DevicePlatform::Apple,
            expected_mode: Some(DeviceMode::Dfu),
            expected_workflow: Some(WorkflowRecommendation::AppleDfuWorkflow),
        },
        // Apple Recovery Device
        KnownDeviceProfile {
            vendor_id: 0x05ac,
            product_id: Some(0x1281),
            display_name: "Apple Recovery Device".to_string(),
            notes: Some("iPhone/iPad in Recovery mode".to_string()),
            expected_platform: DevicePlatform::Apple,
            expected_mode: Some(DeviceMode::Recovery),
            expected_workflow: Some(WorkflowRecommendation::AppleRecoveryWorkflow),
        },
        // Apple Normal Device
        KnownDeviceProfile {
            vendor_id: 0x05ac,
            product_id: Some(0x12a8),
            display_name: "Apple iOS Device (Normal)".to_string(),
            notes: Some("iPhone/iPad in normal operating mode".to_string()),
            expected_platform: DevicePlatform::Apple,
            expected_mode: Some(DeviceMode::Normal),
            expected_workflow: Some(WorkflowRecommendation::AppleNormalInspection),
        },
        // Google Fastboot Device
        KnownDeviceProfile {
            vendor_id: 0x18d1,
            product_id: Some(0x4ee7),
            display_name: "Google Fastboot Device".to_string(),
            notes: Some("Android device in Fastboot mode".to_string()),
            expected_platform: DevicePlatform::Android,
            expected_mode: Some(DeviceMode::Fastboot),
            expected_workflow: Some(WorkflowRecommendation::AndroidFastbootWorkflow),
        },
        // Google ADB Device
        KnownDeviceProfile {
            vendor_id: 0x18d1,
            product_id: Some(0x4ee1),
            display_name: "Google ADB Device".to_string(),
            notes: Some("Android device with ADB enabled".to_string()),
            expected_platform: DevicePlatform::Android,
            expected_mode: Some(DeviceMode::Adb),
            expected_workflow: Some(WorkflowRecommendation::AndroidAdbWorkflow),
        },
        // Samsung ADB Device
        KnownDeviceProfile {
            vendor_id: 0x04e8,
            product_id: Some(0x6860),
            display_name: "Samsung ADB Device".to_string(),
            notes: Some("Samsung device with ADB enabled".to_string()),
            expected_platform: DevicePlatform::Android,
            expected_mode: Some(DeviceMode::Adb),
            expected_workflow: Some(WorkflowRecommendation::AndroidAdbWorkflow),
        },
        // Samsung Bootloader
        KnownDeviceProfile {
            vendor_id: 0x04e8,
            product_id: Some(0x685d),
            display_name: "Samsung Bootloader Device".to_string(),
            notes: Some("Samsung device in Download/Odin mode".to_string()),
            expected_platform: DevicePlatform::Android,
            expected_mode: Some(DeviceMode::Bootloader),
            expected_workflow: Some(WorkflowRecommendation::AndroidFastbootWorkflow),
        },
        // SanDisk Mass Storage
        KnownDeviceProfile {
            vendor_id: 0x0781,
            product_id: None,
            display_name: "SanDisk USB Storage".to_string(),
            notes: Some("SanDisk USB flash drive or card reader".to_string()),
            expected_platform: DevicePlatform::GenericUsb,
            expected_mode: Some(DeviceMode::MassStorage),
            expected_workflow: Some(WorkflowRecommendation::MassStorageInspection),
        },
        // Logitech Peripheral
        KnownDeviceProfile {
            vendor_id: 0x046d,
            product_id: None,
            display_name: "Logitech Peripheral".to_string(),
            notes: Some("Logitech keyboard, mouse, or receiver".to_string()),
            expected_platform: DevicePlatform::GenericUsb,
            expected_mode: None,
            expected_workflow: Some(WorkflowRecommendation::GenericPeripheralInspection),
        },
    ]
}

/// Match a device against known profiles
pub fn match_known_profile(device: &DeviceInfo) -> Option<KnownDeviceProfile> {
    let profiles = known_device_profiles();

    for profile in profiles {
        // Match vendor ID
        if profile.vendor_id != device.vendor_id {
            continue;
        }

        // If profile has specific product ID, match it
        if let Some(profile_pid) = profile.product_id {
            if profile_pid == device.product_id {
                return Some(profile);
            }
        } else {
            // Profile matches any product ID for this vendor
            return Some(profile);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        DeviceFamily, DeviceFingerprint, DeviceTransport, FingerprintConfidence,
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
    fn test_match_apple_dfu() {
        let device = create_test_device(0x05ac, 0x1227);
        let profile = match_known_profile(&device);
        assert!(profile.is_some());
        assert_eq!(profile.unwrap().display_name, "Apple DFU Device");
    }

    #[test]
    fn test_match_apple_recovery() {
        let device = create_test_device(0x05ac, 0x1281);
        let profile = match_known_profile(&device);
        assert!(profile.is_some());
        assert_eq!(profile.unwrap().display_name, "Apple Recovery Device");
    }

    #[test]
    fn test_match_google_fastboot() {
        let device = create_test_device(0x18d1, 0x4ee7);
        let profile = match_known_profile(&device);
        assert!(profile.is_some());
        assert_eq!(profile.unwrap().display_name, "Google Fastboot Device");
    }

    #[test]
    fn test_match_sandisk() {
        let device = create_test_device(0x0781, 0x1234);
        let profile = match_known_profile(&device);
        assert!(profile.is_some());
        assert_eq!(profile.unwrap().display_name, "SanDisk USB Storage");
    }

    #[test]
    fn test_no_match() {
        let device = create_test_device(0x9999, 0x9999);
        let profile = match_known_profile(&device);
        assert!(profile.is_none());
    }
}
