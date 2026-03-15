//! Device fingerprinting and workflow recommendation

use crate::types::{
    DeviceFamily, DeviceFingerprint, DeviceMode, DevicePlatform, FingerprintConfidence,
    WorkflowRecommendation,
};

/// Fingerprint a device based on all available information
pub fn fingerprint_device(
    vendor_id: u16,
    product_id: u16,
    product_name: Option<&str>,
    manufacturer: Option<&str>,
    mode: &DeviceMode,
) -> DeviceFingerprint {
    // Apple devices
    if vendor_id == 0x05ac {
        return fingerprint_apple_device(product_id, product_name, mode);
    }

    // Android devices
    if is_android_vendor(vendor_id) {
        return fingerprint_android_device(vendor_id, product_id, product_name, manufacturer, mode);
    }

    // Storage devices
    if is_storage_device(vendor_id, product_name) {
        return DeviceFingerprint {
            family: DeviceFamily::UsbStorage,
            model_hint: product_name.map(|s| s.to_string()),
            confidence: FingerprintConfidence::High,
        };
    }

    // Peripherals
    if is_peripheral(vendor_id, product_name) {
        return DeviceFingerprint {
            family: DeviceFamily::Peripheral,
            model_hint: product_name.map(|s| s.to_string()),
            confidence: FingerprintConfidence::Medium,
        };
    }

    // Unknown
    DeviceFingerprint {
        family: DeviceFamily::Unknown,
        model_hint: None,
        confidence: FingerprintConfidence::Unknown,
    }
}

fn fingerprint_apple_device(
    product_id: u16,
    product_name: Option<&str>,
    mode: &DeviceMode,
) -> DeviceFingerprint {
    let (family, confidence) = match mode {
        DeviceMode::Recovery | DeviceMode::Dfu => (DeviceFamily::IPhone, FingerprintConfidence::High),
        DeviceMode::Normal => {
            // Try to determine from product name
            if let Some(name) = product_name {
                let name_lower = name.to_lowercase();
                if name_lower.contains("ipad") {
                    (DeviceFamily::IPad, FingerprintConfidence::High)
                } else if name_lower.contains("iphone") {
                    (DeviceFamily::IPhone, FingerprintConfidence::High)
                } else {
                    (DeviceFamily::IPhone, FingerprintConfidence::Medium)
                }
            } else {
                (DeviceFamily::IPhone, FingerprintConfidence::Medium)
            }
        }
        _ => (DeviceFamily::IPhone, FingerprintConfidence::Low),
    };

    DeviceFingerprint {
        family,
        model_hint: product_name.map(|s| s.to_string()),
        confidence,
    }
}

fn fingerprint_android_device(
    vendor_id: u16,
    product_id: u16,
    product_name: Option<&str>,
    manufacturer: Option<&str>,
    mode: &DeviceMode,
) -> DeviceFingerprint {
    // Check if it's a tablet
    let is_tablet = product_name
        .map(|n| n.to_lowercase().contains("tablet"))
        .unwrap_or(false);

    let family = if is_tablet {
        DeviceFamily::AndroidTablet
    } else {
        DeviceFamily::AndroidPhone
    };

    let confidence = match mode {
        DeviceMode::Adb | DeviceMode::Fastboot | DeviceMode::Bootloader => {
            FingerprintConfidence::High
        }
        DeviceMode::Normal => FingerprintConfidence::Medium,
        _ => FingerprintConfidence::Low,
    };

    DeviceFingerprint {
        family,
        model_hint: product_name.map(|s| s.to_string()),
        confidence,
    }
}

fn is_android_vendor(vendor_id: u16) -> bool {
    matches!(
        vendor_id,
        0x18d1 | 0x04e8 | 0x0fce | 0x2a70 | 0x12d1 | 0x22d9 | 0x2717
    )
}

fn is_storage_device(vendor_id: u16, product_name: Option<&str>) -> bool {
    if vendor_id == 0x0781 {
        return true;
    }

    if let Some(name) = product_name {
        let name_lower = name.to_lowercase();
        name_lower.contains("flash")
            || name_lower.contains("storage")
            || name_lower.contains("disk")
    } else {
        false
    }
}

fn is_peripheral(vendor_id: u16, product_name: Option<&str>) -> bool {
    // Logitech, Realtek
    if matches!(vendor_id, 0x046d | 0x0bda) {
        return true;
    }

    if let Some(name) = product_name {
        let name_lower = name.to_lowercase();
        name_lower.contains("keyboard")
            || name_lower.contains("mouse")
            || name_lower.contains("camera")
            || name_lower.contains("receiver")
    } else {
        false
    }
}

/// Recommend workflow based on device characteristics
pub fn recommend_workflow(
    platform: &DevicePlatform,
    mode: &DeviceMode,
    fingerprint: &DeviceFingerprint,
) -> WorkflowRecommendation {
    match (platform, mode, &fingerprint.family) {
        // Apple workflows
        (DevicePlatform::Apple, DeviceMode::Normal, _) => {
            WorkflowRecommendation::AppleNormalInspection
        }
        (DevicePlatform::Apple, DeviceMode::Recovery, _) => {
            WorkflowRecommendation::AppleRecoveryWorkflow
        }
        (DevicePlatform::Apple, DeviceMode::Dfu, _) => WorkflowRecommendation::AppleDfuWorkflow,

        // Android workflows
        (DevicePlatform::Android, DeviceMode::Adb, _) => WorkflowRecommendation::AndroidAdbWorkflow,
        (DevicePlatform::Android, DeviceMode::Fastboot, _) => {
            WorkflowRecommendation::AndroidFastbootWorkflow
        }
        (DevicePlatform::Android, DeviceMode::Bootloader, _) => {
            WorkflowRecommendation::AndroidFastbootWorkflow
        }

        // Storage
        (_, DeviceMode::MassStorage, _) | (_, _, DeviceFamily::UsbStorage) => {
            WorkflowRecommendation::MassStorageInspection
        }

        // Peripherals
        (_, _, DeviceFamily::Peripheral) => WorkflowRecommendation::GenericPeripheralInspection,

        // Default
        _ => WorkflowRecommendation::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apple_dfu_fingerprint() {
        let fp = fingerprint_device(0x05ac, 0x1227, Some("iPhone"), Some("Apple Inc."), &DeviceMode::Dfu);
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
        assert_eq!(workflow, WorkflowRecommendation::GenericPeripheralInspection);
    }
}
