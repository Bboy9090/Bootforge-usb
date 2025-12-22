use crate::model::UsbDeviceRecord;

pub mod adb_probe;
pub mod apple_probe;
pub mod fastboot_probe;
pub mod mtp_probe;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceProtocol {
    Adb,
    Fastboot,
    AppleDevice,
    Mtp,
    Unknown,
}

/// Classify the protocols that a device supports
pub fn classify_device_protocols(device: &UsbDeviceRecord) -> Vec<DeviceProtocol> {
    let mut protocols = Vec::new();

    // Check for ADB
    if adb_probe::is_adb_device(device) {
        protocols.push(DeviceProtocol::Adb);
    }

    // Check for Fastboot
    if fastboot_probe::is_fastboot_device(device) {
        protocols.push(DeviceProtocol::Fastboot);
    }

    // Check for Apple devices
    if apple_probe::is_apple_device(device) {
        protocols.push(DeviceProtocol::AppleDevice);
    }

    // Check for MTP
    if mtp_probe::is_mtp_device(device) {
        protocols.push(DeviceProtocol::Mtp);
    }

    if protocols.is_empty() {
        protocols.push(DeviceProtocol::Unknown);
    }

    protocols
}
