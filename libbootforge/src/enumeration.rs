//! USB device enumeration

use crate::{
    descriptors::DeviceDescriptor,
    device::{DeviceInfo, DeviceMode},
    Result,
};
use std::time::Duration;

/// Enumerate all USB devices connected to the system
pub fn enumerate_devices() -> Result<Vec<DeviceInfo>> {
    let devices = rusb::devices()?;
    let mut device_list = Vec::new();

    for device in devices.iter() {
        if let Ok(info) = get_device_info(&device) {
            device_list.push(info);
        }
    }

    Ok(device_list)
}

/// Get detailed information about a specific USB device
pub fn get_device_info(device: &rusb::Device<rusb::GlobalContext>) -> Result<DeviceInfo> {
    let device_desc = device.device_descriptor()?;
    let descriptor = DeviceDescriptor::read(device)?;

    let vendor_id = device_desc.vendor_id();
    let product_id = device_desc.product_id();
    let class = device_desc.class_code();
    let subclass = device_desc.sub_class_code();
    let protocol = device_desc.protocol_code();

    // Attempt to read string descriptors (may fail without permissions)
    let (manufacturer, product_name, serial_number) = read_string_descriptors(
        device,
        descriptor.manufacturer_index,
        descriptor.product_index,
        descriptor.serial_number_index,
    );

    // Detect device mode
    let device_mode = DeviceMode::detect(vendor_id, product_id, class);

    // Get platform-specific path
    let platform_path = get_platform_path(device);

    Ok(DeviceInfo {
        vendor_id,
        product_id,
        manufacturer,
        product_name,
        serial_number,
        class,
        subclass,
        protocol,
        usb_version: descriptor.usb_version,
        device_version: descriptor.device_version,
        bus_number: device.bus_number(),
        device_address: device.address(),
        platform_path,
        device_mode,
    })
}

/// Attempt to read string descriptors from a device
fn read_string_descriptors(
    device: &rusb::Device<rusb::GlobalContext>,
    manufacturer_index: u8,
    product_index: u8,
    serial_index: u8,
) -> (Option<String>, Option<String>, Option<String>) {
    let timeout = Duration::from_millis(100);

    // Try to open device handle (may fail without permissions)
    let handle = match device.open() {
        Ok(h) => h,
        Err(_) => return (None, None, None),
    };

    let manufacturer =
        DeviceDescriptor::read_string_descriptor(&handle, manufacturer_index, timeout)
            .ok()
            .flatten();

    let product_name = DeviceDescriptor::read_string_descriptor(&handle, product_index, timeout)
        .ok()
        .flatten();

    let serial_number = DeviceDescriptor::read_string_descriptor(&handle, serial_index, timeout)
        .ok()
        .flatten();

    (manufacturer, product_name, serial_number)
}

/// Get platform-specific device path
#[cfg(target_os = "linux")]
fn get_platform_path(device: &rusb::Device<rusb::GlobalContext>) -> Option<String> {
    Some(format!(
        "/dev/bus/usb/{:03}/{:03}",
        device.bus_number(),
        device.address()
    ))
}

#[cfg(target_os = "macos")]
fn get_platform_path(_device: &rusb::Device<rusb::GlobalContext>) -> Option<String> {
    // macOS uses IOKit for device paths
    // This would require platform-specific IOKit bindings
    None
}

#[cfg(target_os = "windows")]
fn get_platform_path(_device: &rusb::Device<rusb::GlobalContext>) -> Option<String> {
    // Windows uses SetupAPI for device paths
    // This would require platform-specific Windows API bindings
    None
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn get_platform_path(_device: &rusb::Device<rusb::GlobalContext>) -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires USB subsystem - not available in CI
    fn test_enumerate_devices() {
        let result = enumerate_devices();
        assert!(result.is_ok());
    }
}
