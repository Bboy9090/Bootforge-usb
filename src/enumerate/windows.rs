use crate::types::UsbDeviceInfo;
use anyhow::Result;

#[cfg(target_os = "windows")]
use crate::model::{DriverStatus, LinkHealth};
#[cfg(target_os = "windows")]
use log::debug;

#[cfg(target_os = "windows")]
use windows::{
    core::GUID,
    Win32::Devices::DeviceAndDriverInstallation::*,
    Win32::Devices::Usb::GUID_DEVINTERFACE_USB_DEVICE,
    Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, ERROR_NO_MORE_ITEMS, HANDLE, NO_ERROR},
};

/// Enrich USB device information with Windows-specific data using SetupAPI
///
/// This implementation provides comprehensive Windows enrichment including:
/// - Device instance paths and hardware IDs via SetupDiGetClassDevs
/// - Driver information and status
/// - Device health monitoring
/// - Port topology mapping
/// - Cross-platform tagging
#[cfg(target_os = "windows")]
pub fn enrich_windows(devices: &mut [UsbDeviceInfo]) -> Result<()> {
    debug!("Windows enrichment using SetupAPI");

    // Get device information set for all USB devices
    let device_info_set = unsafe {
        SetupDiGetClassDevsW(
            Some(&GUID_DEVINTERFACE_USB_DEVICE),
            None,
            HANDLE::default(),
            DIGCF_PRESENT | DIGCF_DEVICEINTERFACE,
        )
    };

    if device_info_set.is_invalid() {
        warn!("Failed to get USB device information set");
        return Ok(());
    }

    // Ensure cleanup of device info set
    let _cleanup = DeviceInfoSetGuard(device_info_set);

    // Enumerate all devices in the set
    let mut device_index = 0u32;
    loop {
        let mut device_info_data = SP_DEVINFO_DATA {
            cbSize: std::mem::size_of::<SP_DEVINFO_DATA>() as u32,
            ..Default::default()
        };

        let result = unsafe {
            SetupDiEnumDeviceInfo(device_info_set, device_index, &mut device_info_data)
        };

        if !result.as_bool() {
            let error = unsafe { windows::Win32::Foundation::GetLastError() };
            if error == ERROR_NO_MORE_ITEMS {
                break;
            }
            device_index += 1;
            continue;
        }

        // Get hardware IDs to extract VID/PID
        if let Ok(hardware_ids) = get_device_registry_property_multi(
            device_info_set,
            &device_info_data,
            SPDRP_HARDWAREID,
        ) {
            // Try to match this device with one from our list
            if let Some(device) = find_device_by_hardware_id(devices, &hardware_ids) {
                // Store hardware IDs
                device.platform_hint.hardware_ids = hardware_ids.clone();

                // Get device instance path
                if let Ok(instance_id) = get_device_instance_id(device_info_set, &device_info_data)
                {
                    device.platform_hint.instance_path = Some(instance_id.clone());
                    device.platform_hint.device_path = Some(format!("\\\\?\\{}", instance_id));
                }

                // Get driver information
                if let Ok(driver_info) = get_driver_info(device_info_set, &device_info_data) {
                    device.platform_hint.driver_name = Some(driver_info.clone());
                    device.driver_status = DriverStatus::Bound { name: driver_info };
                } else {
                    device.driver_status = DriverStatus::Missing;
                }

                // Get device status and determine health
                if let Ok((status, problem)) = get_device_status(device_info_set, &device_info_data)
                {
                    device.link_health = determine_device_health(status, problem);
                }

                // Get location information for port topology
                if let Ok(location) = get_location_information(device_info_set, &device_info_data) {
                    device.port_path = Some(location);
                }

                // Add platform tag
                device.add_tag("windows");

                // Add tags based on device characteristics
                add_device_tags(device);
            }
        }

        device_index += 1;
    }

    debug!("Windows enrichment completed for {} devices", devices.len());
    Ok(())
}

/// Guard to ensure SetupDi device info set is cleaned up
#[cfg(target_os = "windows")]
struct DeviceInfoSetGuard(HDEVINFO);

#[cfg(target_os = "windows")]
impl Drop for DeviceInfoSetGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = SetupDiDestroyDeviceInfoList(self.0);
        }
    }
}

/// Get a device registry property as a multi-string
#[cfg(target_os = "windows")]
fn get_device_registry_property_multi(
    device_info_set: HDEVINFO,
    device_info_data: &SP_DEVINFO_DATA,
    property: SETUP_DI_REGISTRY_PROPERTY,
) -> Result<Vec<String>> {
    let mut buffer = vec![0u16; 1024];
    let mut data_type = 0u32;
    let mut required_size = 0u32;

    let result = unsafe {
        SetupDiGetDeviceRegistryPropertyW(
            device_info_set,
            device_info_data,
            property,
            Some(&mut data_type),
            Some(buffer.as_mut_ptr() as *mut u8),
            (buffer.len() * 2) as u32,
            Some(&mut required_size),
        )
    };

    if !result.as_bool() {
        let error = unsafe { windows::Win32::Foundation::GetLastError() };
        if error == ERROR_INSUFFICIENT_BUFFER && required_size > 0 {
            buffer.resize((required_size / 2) as usize + 1, 0);
            let result2 = unsafe {
                SetupDiGetDeviceRegistryPropertyW(
                    device_info_set,
                    device_info_data,
                    property,
                    Some(&mut data_type),
                    Some(buffer.as_mut_ptr() as *mut u8),
                    (buffer.len() * 2) as u32,
                    None,
                )
            };
            if !result2.as_bool() {
                return Err(anyhow::anyhow!("Failed to get registry property"));
            }
        } else {
            return Err(anyhow::anyhow!("Failed to get registry property"));
        }
    }

    // Parse multi-string (null-separated strings, double-null terminated)
    let mut strings = Vec::new();
    let mut start = 0;
    for i in 0..buffer.len() {
        if buffer[i] == 0 {
            if i > start {
                let s = String::from_utf16_lossy(&buffer[start..i]);
                if !s.is_empty() {
                    strings.push(s);
                }
            }
            start = i + 1;
            if i + 1 < buffer.len() && buffer[i + 1] == 0 {
                break;
            }
        }
    }

    Ok(strings)
}

/// Get device instance ID
#[cfg(target_os = "windows")]
fn get_device_instance_id(
    device_info_set: HDEVINFO,
    device_info_data: &SP_DEVINFO_DATA,
) -> Result<String> {
    let mut buffer = vec![0u16; 512];
    let mut required_size = 0u32;

    let result = unsafe {
        SetupDiGetDeviceInstanceIdW(
            device_info_set,
            device_info_data,
            Some(&mut buffer),
            Some(&mut required_size),
        )
    };

    if !result.as_bool() {
        return Err(anyhow::anyhow!("Failed to get device instance ID"));
    }

    // Find null terminator
    let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
    Ok(String::from_utf16_lossy(&buffer[..len]))
}

/// Get driver information
#[cfg(target_os = "windows")]
fn get_driver_info(device_info_set: HDEVINFO, device_info_data: &SP_DEVINFO_DATA) -> Result<String> {
    // Try to get the driver description
    let mut buffer = vec![0u16; 256];
    let mut data_type = 0u32;

    let result = unsafe {
        SetupDiGetDeviceRegistryPropertyW(
            device_info_set,
            device_info_data,
            SPDRP_DRIVER,
            Some(&mut data_type),
            Some(buffer.as_mut_ptr() as *mut u8),
            (buffer.len() * 2) as u32,
            None,
        )
    };

    if result.as_bool() {
        let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
        return Ok(String::from_utf16_lossy(&buffer[..len]));
    }

    // Fallback: try to get service name
    let result2 = unsafe {
        SetupDiGetDeviceRegistryPropertyW(
            device_info_set,
            device_info_data,
            SPDRP_SERVICE,
            Some(&mut data_type),
            Some(buffer.as_mut_ptr() as *mut u8),
            (buffer.len() * 2) as u32,
            None,
        )
    };

    if result2.as_bool() {
        let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
        Ok(String::from_utf16_lossy(&buffer[..len]))
    } else {
        Err(anyhow::anyhow!("No driver information available"))
    }
}

/// Get device status and problem code
#[cfg(target_os = "windows")]
fn get_device_status(
    device_info_set: HDEVINFO,
    device_info_data: &SP_DEVINFO_DATA,
) -> Result<(u32, u32)> {
    let mut status = 0u32;
    let mut problem = 0u32;

    let result = unsafe {
        CM_Get_DevNode_Status(
            &mut status,
            &mut problem,
            device_info_data.DevInst,
            0,
        )
    };

    if result == 0 {
        Ok((status, problem))
    } else {
        Err(anyhow::anyhow!("Failed to get device status"))
    }
}

/// Determine device health from status and problem codes
#[cfg(target_os = "windows")]
fn determine_device_health(status: u32, problem: u32) -> LinkHealth {
    const DN_STARTED: u32 = 0x00000008;
    const DN_HAS_PROBLEM: u32 = 0x00000400;
    const CM_PROB_FAILED_START: u32 = 10;
    const CM_PROB_DEVICE_NOT_THERE: u32 = 24;
    const CM_PROB_DISABLED: u32 = 22;

    if problem != 0 {
        match problem {
            CM_PROB_DEVICE_NOT_THERE => LinkHealth::Disconnected,
            CM_PROB_DISABLED => LinkHealth::Unstable {
                reason: "Device is disabled".to_string(),
            },
            CM_PROB_FAILED_START => LinkHealth::Unstable {
                reason: "Device failed to start".to_string(),
            },
            _ => LinkHealth::Unstable {
                reason: format!("Device has problem code: {}", problem),
            },
        }
    } else if (status & DN_HAS_PROBLEM) != 0 {
        LinkHealth::Unstable {
            reason: "Device has unspecified problem".to_string(),
        }
    } else if (status & DN_STARTED) != 0 {
        LinkHealth::Good
    } else {
        LinkHealth::Unstable {
            reason: "Device is not started".to_string(),
        }
    }
}

/// Get location information for port topology
#[cfg(target_os = "windows")]
fn get_location_information(
    device_info_set: HDEVINFO,
    device_info_data: &SP_DEVINFO_DATA,
) -> Result<String> {
    let mut buffer = vec![0u16; 256];
    let mut data_type = 0u32;

    let result = unsafe {
        SetupDiGetDeviceRegistryPropertyW(
            device_info_set,
            device_info_data,
            SPDRP_LOCATION_INFORMATION,
            Some(&mut data_type),
            Some(buffer.as_mut_ptr() as *mut u8),
            (buffer.len() * 2) as u32,
            None,
        )
    };

    if result.as_bool() {
        let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
        return Ok(String::from_utf16_lossy(&buffer[..len]));
    }

    // Fallback: try to construct from parent info
    Ok(String::from("Unknown"))
}

/// Find a device by matching hardware ID (VID/PID)
#[cfg(target_os = "windows")]
fn find_device_by_hardware_id<'a>(
    devices: &'a mut [UsbDeviceInfo],
    hardware_ids: &[String],
) -> Option<&'a mut UsbDeviceInfo> {
    for hw_id in hardware_ids {
        // Parse VID/PID from hardware ID (format: USB\VID_xxxx&PID_yyyy)
        if let Some((vid, pid)) = parse_vid_pid_from_hardware_id(hw_id) {
            // Find device with matching VID/PID that hasn't been enriched yet
            if let Some(device) = devices.iter_mut().find(|d| {
                d.vendor_id == vid
                    && d.product_id == pid
                    && d.platform_hint.instance_path.is_none()
            }) {
                return Some(device);
            }
        }
    }
    None
}

/// Parse VID/PID from Windows hardware ID string
#[cfg(target_os = "windows")]
fn parse_vid_pid_from_hardware_id(hw_id: &str) -> Option<(u16, u16)> {
    let upper = hw_id.to_uppercase();

    // Look for VID_xxxx pattern
    let vid = if let Some(vid_pos) = upper.find("VID_") {
        let vid_start = vid_pos + 4;
        if vid_start + 4 <= upper.len() {
            u16::from_str_radix(&upper[vid_start..vid_start + 4], 16).ok()?
        } else {
            return None;
        }
    } else {
        return None;
    };

    // Look for PID_xxxx pattern
    let pid = if let Some(pid_pos) = upper.find("PID_") {
        let pid_start = pid_pos + 4;
        if pid_start + 4 <= upper.len() {
            u16::from_str_radix(&upper[pid_start..pid_start + 4], 16).ok()?
        } else {
            return None;
        }
    } else {
        return None;
    };

    Some((vid, pid))
}

/// Add device tags based on characteristics
#[cfg(target_os = "windows")]
fn add_device_tags(device: &mut UsbDeviceInfo) {
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
}

/// Non-Windows platforms: no-op
#[cfg(not(target_os = "windows"))]
pub fn enrich_windows(_devices: &mut [UsbDeviceInfo]) -> Result<()> {
    // No-op on non-Windows platforms
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::UsbDeviceInfo;

    #[test]
    fn test_enrich_windows() {
        let mut devices = vec![UsbDeviceInfo::new(0x1234, 0x5678)];
        let result = enrich_windows(&mut devices);
        assert!(result.is_ok());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_parse_vid_pid() {
        assert_eq!(
            parse_vid_pid_from_hardware_id("USB\\VID_045E&PID_07A5"),
            Some((0x045E, 0x07A5))
        );
        assert_eq!(
            parse_vid_pid_from_hardware_id("usb\\vid_18d1&pid_4ee1"),
            Some((0x18D1, 0x4EE1))
        );
        assert_eq!(parse_vid_pid_from_hardware_id("INVALID"), None);
    }
}
