//! Vendor, platform, transport, and mode classification

use crate::types::{DeviceMode, DevicePlatform, DeviceTransport};

/// Classify vendor name from vendor ID
pub fn classify_vendor_name(vendor_id: u16) -> Option<String> {
    let vendor = match vendor_id {
        0x05ac => "Apple",
        0x18d1 => "Google",
        0x04e8 => "Samsung",
        0x0fce => "Sony",
        0x2a70 => "OnePlus",
        0x12d1 => "Huawei",
        0x22d9 => "OPPO",
        0x2717 => "Xiaomi",
        0x0bda => "Realtek",
        0x0781 => "SanDisk",
        0x046d => "Logitech",
        _ => return None,
    };
    Some(vendor.to_string())
}

/// Classify device platform from vendor ID
pub fn classify_platform(vendor_id: u16) -> DevicePlatform {
    match vendor_id {
        0x05ac => DevicePlatform::Apple,
        0x18d1 | 0x04e8 | 0x0fce | 0x2a70 | 0x12d1 | 0x22d9 | 0x2717 => DevicePlatform::Android,
        0x0781 | 0x0bda | 0x046d => DevicePlatform::GenericUsb,
        _ => DevicePlatform::Unknown,
    }
}

/// Classify transport type from USB version BCD
pub fn classify_transport(usb_version_bcd: u16) -> DeviceTransport {
    match usb_version_bcd {
        0x0300..=0x03ff => DeviceTransport::Usb3,
        0x0200..=0x02ff => DeviceTransport::Usb2,
        _ => DeviceTransport::Unknown,
    }
}

/// Classify device mode from vendor and product IDs
pub fn classify_mode(vendor_id: u16, product_id: u16) -> DeviceMode {
    match (vendor_id, product_id) {
        // Apple modes
        (0x05ac, 0x12a8) => DeviceMode::Normal,
        (0x05ac, 0x1281) => DeviceMode::Recovery,
        (0x05ac, 0x1227) => DeviceMode::Dfu,

        // Google/Android modes
        (0x18d1, 0x4ee7) => DeviceMode::Fastboot,
        (0x18d1, 0x4ee1) => DeviceMode::Adb,

        // Samsung modes
        (0x04e8, 0x6860) => DeviceMode::Adb,
        (0x04e8, 0x685d) => DeviceMode::Bootloader,

        // Storage devices
        (0x0781, _) => DeviceMode::MassStorage,

        _ => DeviceMode::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vendor_classification() {
        assert_eq!(classify_vendor_name(0x05ac), Some("Apple".to_string()));
        assert_eq!(classify_vendor_name(0x18d1), Some("Google".to_string()));
        assert_eq!(classify_vendor_name(0x04e8), Some("Samsung".to_string()));
        assert_eq!(classify_vendor_name(0x9999), None);
    }

    #[test]
    fn test_platform_classification() {
        assert_eq!(classify_platform(0x05ac), DevicePlatform::Apple);
        assert_eq!(classify_platform(0x18d1), DevicePlatform::Android);
        assert_eq!(classify_platform(0x0781), DevicePlatform::GenericUsb);
    }

    #[test]
    fn test_transport_classification() {
        assert_eq!(classify_transport(0x0300), DeviceTransport::Usb3);
        assert_eq!(classify_transport(0x0200), DeviceTransport::Usb2);
    }

    #[test]
    fn test_mode_classification() {
        assert_eq!(classify_mode(0x05ac, 0x1227), DeviceMode::Dfu);
        assert_eq!(classify_mode(0x05ac, 0x1281), DeviceMode::Recovery);
        assert_eq!(classify_mode(0x18d1, 0x4ee7), DeviceMode::Fastboot);
    }
}
