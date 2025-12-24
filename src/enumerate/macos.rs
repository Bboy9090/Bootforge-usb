use crate::types::UsbDeviceInfo;
use anyhow::Result;

#[cfg(target_os = "macos")]
use crate::model::{DriverStatus, LinkHealth};
#[cfg(target_os = "macos")]
use log::debug;

#[cfg(target_os = "macos")]
use core_foundation::{
    base::{kCFAllocatorDefault, CFType, TCFType},
    dictionary::{CFDictionary, CFDictionaryRef},
    number::{CFNumber, CFNumberRef},
    string::{CFString, CFStringRef},
};
#[cfg(target_os = "macos")]
use io_kit_sys::{
    kIOMasterPortDefault, ret::kIOReturnSuccess, types::*, usb::lib::*, IORegistryEntryCreateCFProperties,
    IORegistryEntryGetPath, IOServiceGetMatchingServices, IOServiceMatching,
};
#[cfg(target_os = "macos")]
use std::ffi::CStr;
#[cfg(target_os = "macos")]
use std::ptr;

/// Enrich USB device information with macOS-specific data using IOKit
///
/// This implementation provides comprehensive macOS enrichment including:
/// - IORegistry device enumeration
/// - VID/PID detection and matching
/// - Driver status from IOKit
/// - Device health monitoring
/// - Port topology via location IDs
/// - Cross-platform tagging
#[cfg(target_os = "macos")]
pub fn enrich_macos(devices: &mut [UsbDeviceInfo]) -> Result<()> {
    debug!("macOS enrichment using IOKit");

    unsafe {
        // Try both IOUSBHostDevice (modern) and IOUSBDevice (legacy)
        for service_name in &["IOUSBHostDevice", "IOUSBDevice"] {
            let matching = IOServiceMatching(service_name.as_ptr() as *const i8);
            if matching.is_null() {
                continue;
            }

            let mut iterator: io_iterator_t = 0;
            let result = IOServiceGetMatchingServices(kIOMasterPortDefault, matching, &mut iterator);

            if result != kIOReturnSuccess {
                continue;
            }

            let _cleanup = IoIteratorGuard(iterator);

            // Enumerate all USB devices
            loop {
                let service = io_kit_sys::IOIteratorNext(iterator);
                if service == 0 {
                    break;
                }

                let _service_cleanup = IoServiceGuard(service);

                // Get device properties
                if let Ok(props) = get_device_properties(service) {
                    // Extract VID/PID
                    if let Some((vid, pid)) = extract_vid_pid(&props) {
                        // Find matching device in our list
                        if let Some(device) = devices.iter_mut().find(|d| {
                            d.vendor_id == vid
                                && d.product_id == pid
                                && d.platform_hint.ioregistry_path.is_none()
                        }) {
                            // Get IORegistry path
                            if let Ok(path) = get_registry_path(service) {
                                device.platform_hint.ioregistry_path = Some(path);
                            }

                            // Get location ID for port topology
                            if let Some(location_id) = get_number_property(&props, "locationID") {
                                device.platform_hint.location_id = Some(location_id);
                                device.port_path = Some(format_location_path(location_id));
                            }

                            // Get driver information
                            if let Some(driver) = get_driver_name(service) {
                                device.platform_hint.driver_name = Some(driver.clone());
                                device.driver_status = DriverStatus::Bound { name: driver };
                            } else {
                                device.driver_status = DriverStatus::Missing;
                            }

                            // Determine device health
                            device.link_health = determine_macos_device_health(&props);

                            // Add platform tag
                            device.add_tag("macos");

                            // Add tags based on device characteristics
                            add_macos_device_tags(device, &props);
                        }
                    }
                }
            }
        }
    }

    debug!("macOS enrichment completed for {} devices", devices.len());
    Ok(())
}

/// Guard to ensure IOIterator is released
#[cfg(target_os = "macos")]
struct IoIteratorGuard(io_iterator_t);

#[cfg(target_os = "macos")]
impl Drop for IoIteratorGuard {
    fn drop(&mut self) {
        unsafe {
            io_kit_sys::IOObjectRelease(self.0);
        }
    }
}

/// Guard to ensure IOService is released
#[cfg(target_os = "macos")]
struct IoServiceGuard(io_service_t);

#[cfg(target_os = "macos")]
impl Drop for IoServiceGuard {
    fn drop(&mut self) {
        unsafe {
            io_kit_sys::IOObjectRelease(self.0);
        }
    }
}

/// Get device properties from IORegistry
#[cfg(target_os = "macos")]
fn get_device_properties(service: io_service_t) -> Result<CFDictionary<CFString, CFType>> {
    unsafe {
        let mut props: CFDictionaryRef = ptr::null();
        let result = IORegistryEntryCreateCFProperties(
            service,
            &mut props,
            kCFAllocatorDefault,
            0,
        );

        if result != kIOReturnSuccess || props.is_null() {
            return Err(anyhow::anyhow!("Failed to get IORegistry properties"));
        }

        Ok(CFDictionary::wrap_under_create_rule(props))
    }
}

/// Get IORegistry path for a service
#[cfg(target_os = "macos")]
fn get_registry_path(service: io_service_t) -> Result<String> {
    unsafe {
        let mut path = [0i8; 512];
        let result = IORegistryEntryGetPath(
            service,
            b"IOService\0".as_ptr() as *const i8,
            path.as_mut_ptr(),
        );

        if result != kIOReturnSuccess {
            return Err(anyhow::anyhow!("Failed to get IORegistry path"));
        }

        let c_str = CStr::from_ptr(path.as_ptr());
        Ok(c_str.to_string_lossy().into_owned())
    }
}

/// Extract VID/PID from device properties
#[cfg(target_os = "macos")]
fn extract_vid_pid(props: &CFDictionary<CFString, CFType>) -> Option<(u16, u16)> {
    let vid = get_number_property(props, "idVendor")? as u16;
    let pid = get_number_property(props, "idProduct")? as u16;
    Some((vid, pid))
}

/// Get a number property from the dictionary
#[cfg(target_os = "macos")]
fn get_number_property(props: &CFDictionary<CFString, CFType>, key: &str) -> Option<u32> {
    let key_string = CFString::new(key);
    let value = props.find(&key_string)?;
    
    unsafe {
        let number = CFNumber::wrap_under_get_rule(value.as_CFTypeRef() as CFNumberRef);
        let mut result: i64 = 0;
        if number.to_i64(&mut result) {
            Some(result as u32)
        } else {
            None
        }
    }
}

/// Get a string property from the dictionary
#[cfg(target_os = "macos")]
fn get_string_property(props: &CFDictionary<CFString, CFType>, key: &str) -> Option<String> {
    let key_string = CFString::new(key);
    let value = props.find(&key_string)?;
    
    unsafe {
        let string = CFString::wrap_under_get_rule(value.as_CFTypeRef() as CFStringRef);
        Some(string.to_string())
    }
}

/// Get driver name for the device
#[cfg(target_os = "macos")]
fn get_driver_name(service: io_service_t) -> Option<String> {
    unsafe {
        // Try to get the driver name from IOService
        let mut driver: io_service_t = 0;
        let result = io_kit_sys::IORegistryEntryGetChildEntry(
            service,
            b"IOService\0".as_ptr() as *const i8,
            &mut driver,
        );

        if result == kIOReturnSuccess && driver != 0 {
            let _driver_cleanup = IoServiceGuard(driver);
            
            // Get driver properties
            if let Ok(props) = get_device_properties(driver) {
                if let Some(name) = get_string_property(&props, "IOClass") {
                    return Some(name);
                }
            }
        }

        // Fallback: check for CFBundleIdentifier
        if let Ok(props) = get_device_properties(service) {
            if let Some(bundle) = get_string_property(&props, "CFBundleIdentifier") {
                return Some(bundle);
            }
        }

        None
    }
}

/// Format location ID as a port path
#[cfg(target_os = "macos")]
fn format_location_path(location_id: u32) -> String {
    // Location ID encodes the port path in the format:
    // Bits 24-31: Bus number
    // Bits 20-23: Port on first hub
    // Bits 16-19: Port on second hub
    // And so on...
    
    let bus = (location_id >> 24) & 0xFF;
    let mut ports = Vec::new();
    
    for i in (0..6).rev() {
        let port = (location_id >> (i * 4)) & 0xF;
        if port != 0 {
            ports.push(port);
        }
    }
    
    if ports.is_empty() {
        format!("bus-{}", bus)
    } else {
        format!("bus-{}-{}", bus, ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join("."))
    }
}

/// Determine device health from IOKit properties
#[cfg(target_os = "macos")]
fn determine_macos_device_health(props: &CFDictionary<CFString, CFType>) -> LinkHealth {
    // Check for reset count (indicates instability)
    if let Some(reset_count) = get_number_property(props, "kUSBResetCount") {
        if reset_count > 5 {
            return LinkHealth::ResetLoop;
        } else if reset_count > 2 {
            return LinkHealth::Unstable {
                reason: format!("Device has been reset {} times", reset_count),
            };
        }
    }

    // Check for suspend state
    if let Some(suspended) = get_number_property(props, "kUSBSuspendState") {
        if suspended != 0 {
            return LinkHealth::Unstable {
                reason: "Device is suspended".to_string(),
            };
        }
    }

    // Check current available (power issues)
    if let Some(current_available) = get_number_property(props, "kUSBCurrentAvailable") {
        if let Some(current_required) = get_number_property(props, "kUSBCurrentRequired") {
            if current_available < current_required {
                return LinkHealth::PowerIssueHint {
                    reason: format!(
                        "Insufficient power: available {}mA, required {}mA",
                        current_available, current_required
                    ),
                };
            }
        }
    }

    LinkHealth::Good
}

/// Add device tags based on characteristics
#[cfg(target_os = "macos")]
fn add_macos_device_tags(device: &mut UsbDeviceInfo, props: &CFDictionary<CFString, CFType>) {
    // Tag by class
    match device.class {
        0x03 => device.add_tag("hid"),
        0x08 => device.add_tag("mass-storage"),
        0x09 => device.add_tag("hub"),
        0x0A => device.add_tag("cdc"),
        0x0E => device.add_tag("video"),
        0xEF => device.add_tag("miscellaneous"),
        0xFF => device.add_tag("vendor-specific"),
        _ => {}
    }

    // Tag well-known vendors
    match device.vendor_id {
        0x05AC => device.add_tag("apple"),
        0x18D1 => device.add_tag("google"),
        0x045E => device.add_tag("microsoft"),
        0x046D => device.add_tag("logitech"),
        0x04B8 => device.add_tag("epson"),
        0x03F0 => device.add_tag("hp"),
        0x04E8 => device.add_tag("samsung"),
        _ => {}
    }

    // Tag by USB speed
    if let Some(speed) = get_number_property(props, "Device Speed") {
        match speed {
            0 => device.add_tag("low-speed"),
            1 => device.add_tag("full-speed"),
            2 => device.add_tag("high-speed"),
            3 => device.add_tag("super-speed"),
            4 => device.add_tag("super-speed-plus"),
            _ => {}
        }
    }
}

/// Non-macOS platforms: no-op
#[cfg(not(target_os = "macos"))]
pub fn enrich_macos(_devices: &mut [UsbDeviceInfo]) -> Result<()> {
    // No-op on non-macOS platforms
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::UsbDeviceInfo;

    #[test]
    fn test_enrich_macos() {
        let mut devices = vec![UsbDeviceInfo::new(0x1234, 0x5678)];
        let result = enrich_macos(&mut devices);
        assert!(result.is_ok());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_format_location_path() {
        // Test location ID formatting
        assert_eq!(format_location_path(0x14100000), "bus-20");
        assert_eq!(format_location_path(0x14120000), "bus-20-2");
        assert_eq!(format_location_path(0x14123000), "bus-20-3.2");
    }
}
