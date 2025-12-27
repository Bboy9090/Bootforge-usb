use crate::model::UsbDeviceRecord;

pub mod adb_probe;
pub mod apple_probe;
pub mod fastboot_probe;
pub mod mtp_probe;

/// High-level protocol types that USB devices may support.
/// 
/// These protocols are detected during Stage 4 (Protocol Classification)
/// based on USB descriptors, vendor/product IDs, and interface classes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceProtocol {
    /// Android Debug Bridge
    Adb,
    /// Android Fastboot (bootloader mode)
    Fastboot,
    /// Apple iOS/iPadOS device
    AppleDevice,
    /// Media Transfer Protocol
    Mtp,
    /// Protocol not identified
    Unknown,
}

/// Classify the protocols that a confirmed device supports.
/// 
/// This is Stage 4 of the detection pipeline: Protocol Classification.
/// It analyzes the device's characteristics and tags it with applicable
/// protocols (ADB, Fastboot, Apple, MTP, etc.).
/// 
/// A device may support multiple protocols (e.g., an Android device in
/// normal mode supports both ADB and MTP).
/// 
/// # Example
/// ```no_run
/// use bootforge_usb::{enumerate_all, classify_device_protocols};
/// use bootforge_usb::api::UsbEnumerator;
/// use bootforge_usb::enumerate::FallbackEnumerator;
/// 
/// let enumerator = FallbackEnumerator::default();
/// let devices = enumerator.enumerate()?;
/// 
/// for device in devices {
///     let protocols = classify_device_protocols(&device);
///     println!("Device supports: {:?}", protocols);
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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
