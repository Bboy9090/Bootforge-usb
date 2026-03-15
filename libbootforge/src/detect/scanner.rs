//! USB device scanning and information gathering

use crate::error::{BootforgeError, Result};
use crate::types::DeviceInfo;
use rusb::{Context, Device, DeviceHandle, UsbContext};
use std::time::Duration;

use super::classifier::{
    classify_mode, classify_platform, classify_transport, classify_vendor_name,
};
use super::fingerprint::{fingerprint_device, recommend_workflow};
use super::profiles::match_known_profile;

/// Scan all USB devices and return detailed information
pub fn scan_devices() -> Result<Vec<DeviceInfo>> {
    let context = Context::new().map_err(|e| BootforgeError::UsbScanFailed(e.to_string()))?;
    let devices = context
        .devices()
        .map_err(|e| BootforgeError::UsbScanFailed(e.to_string()))?;

    let mut device_list = Vec::new();

    for device in devices.iter() {
        if let Ok(info) = read_device_info(&device) {
            device_list.push(info);
        }
    }

    Ok(device_list)
}

/// Read detailed information from a single device
fn read_device_info(device: &Device<Context>) -> Result<DeviceInfo> {
    let device_desc = device
        .device_descriptor()
        .map_err(|e| BootforgeError::DescriptorReadFailed(e.to_string()))?;

    let vendor_id = device_desc.vendor_id();
    let product_id = device_desc.product_id();
    let usb_version = device_desc.usb_version();

    // Convert USB version to BCD format
    let usb_version_bcd = (usb_version.major() as u16) << 8
        | (usb_version.minor() as u16) << 4
        | (usb_version.sub_minor() as u16);

    // Read string descriptors
    let (manufacturer, product_name, serial_number) = read_string_descriptors(
        device,
        device_desc.manufacturer_string_index().unwrap_or(0),
        device_desc.product_string_index().unwrap_or(0),
        device_desc.serial_number_string_index().unwrap_or(0),
    );

    // Classify device
    let vendor_name = classify_vendor_name(vendor_id);
    let platform = classify_platform(vendor_id);
    let transport = classify_transport(usb_version_bcd);
    let mode = classify_mode(vendor_id, product_id);

    // Fingerprint device
    let fingerprint = fingerprint_device(
        vendor_id,
        product_id,
        product_name.as_deref(),
        manufacturer.as_deref(),
        &mode,
    );

    // Recommend workflow
    let recommended_workflow = recommend_workflow(&platform, &mode, &fingerprint);

    Ok(DeviceInfo {
        bus_number: device.bus_number(),
        address: device.address(),
        vendor_id,
        product_id,
        vendor_name,
        manufacturer,
        product_name,
        serial_number,
        platform,
        transport,
        mode,
        fingerprint,
        recommended_workflow,
    })
}

/// Attempt to read string descriptors from a device
fn read_string_descriptors<T: UsbContext>(
    device: &Device<T>,
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

    let manufacturer = read_string_descriptor(&handle, manufacturer_index, timeout);
    let product_name = read_string_descriptor(&handle, product_index, timeout);
    let serial_number = read_string_descriptor(&handle, serial_index, timeout);

    (manufacturer, product_name, serial_number)
}

/// Read a single string descriptor
fn read_string_descriptor<T: UsbContext>(
    handle: &DeviceHandle<T>,
    index: u8,
    _timeout: Duration,
) -> Option<String> {
    if index == 0 {
        return None;
    }

    match handle.read_string_descriptor_ascii(index) {
        Ok(s) => Some(s),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires USB hardware access
    fn test_scan_devices() {
        let result = scan_devices();
        assert!(result.is_ok());
    }
}
