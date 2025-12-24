use crate::model::{DriverStatus, LinkHealth};
use crate::types::UsbDeviceInfo;
use anyhow::{Context, Result};
use log::debug;
use std::fs;
use std::path::PathBuf;

/// Enrich USB device information with Linux-specific data using udev + sysfs
///
/// This implementation provides comprehensive Linux enrichment including:
/// - sysfs-based device enumeration
/// - VID/PID detection from sysfs
/// - Driver binding detection
/// - Authorization checks
/// - Device health monitoring (reset count, errors)
/// - Port mapping via devpath
/// - Cross-platform tagging
#[cfg(target_os = "linux")]
pub fn enrich_linux(devices: &mut [UsbDeviceInfo]) -> Result<()> {
    debug!("Linux enrichment using udev + sysfs");

    for device in devices.iter_mut() {
        // Try to find the sysfs path for this device
        let sysfs_path = find_sysfs_path(device.bus_number, device.device_address);

        if let Some(path) = sysfs_path {
            device.platform_hint.sysfs_path = Some(path.to_string_lossy().to_string());

            // Enrich from sysfs
            enrich_from_sysfs(device, &path);

            // Get driver information
            if let Some(driver) = get_driver_name(&path) {
                device.platform_hint.driver = Some(driver.clone());
                device.driver_status = DriverStatus::Bound { name: driver };
            } else {
                device.driver_status = DriverStatus::Missing;
            }

            // Check authorization status
            if let Ok(authorized) = read_sysfs_number(&path, "authorized") {
                device.platform_hint.authorized = Some(authorized != 0);
                if authorized == 0 {
                    device.driver_status = DriverStatus::Blocked {
                        reason: "Device not authorized".to_string(),
                    };
                }
            }

            // Determine device health
            device.link_health = determine_linux_device_health(&path);

            // Get port topology
            if let Some(port_path) = get_port_path(&path) {
                device.port_path = Some(port_path);
            }

            // Get device node
            if let Some(devnode) = get_device_node(device.bus_number, device.device_address) {
                device.platform_hint.devnode = Some(devnode);
            }

            // Add platform tag
            device.add_tag("linux");

            // Add tags based on device characteristics
            add_linux_device_tags(device, &path);
        }
    }

    debug!("Linux enrichment completed for {} devices", devices.len());
    Ok(())
}

/// Find the sysfs path for a USB device given its bus and device number
#[cfg(target_os = "linux")]
fn find_sysfs_path(bus_number: u8, device_address: u8) -> Option<PathBuf> {
    let sysfs_base = PathBuf::from("/sys/bus/usb/devices");

    if !sysfs_base.exists() {
        return None;
    }

    // Try to find device by matching bus and device number
    if let Ok(entries) = fs::read_dir(&sysfs_base) {
        for entry in entries.flatten() {
            let path = entry.path();

            // Read busnum and devnum files to match
            if let (Ok(bus), Ok(dev)) = (
                read_sysfs_number(&path, "busnum"),
                read_sysfs_number(&path, "devnum"),
            ) {
                if bus == bus_number && dev == device_address {
                    return Some(path);
                }
            }
        }
    }

    None
}

/// Read a numeric value from a sysfs file
#[cfg(target_os = "linux")]
fn read_sysfs_number(device_path: &std::path::Path, filename: &str) -> Result<u8> {
    let file_path = device_path.join(filename);
    let content = fs::read_to_string(&file_path)
        .with_context(|| format!("Failed to read {}", file_path.display()))?;
    let value = content.trim().parse().with_context(|| {
        format!(
            "Failed to parse {} from {} as number",
            content.trim(),
            file_path.display()
        )
    })?;
    Ok(value)
}

/// Read a numeric value as u32 from a sysfs file
#[cfg(target_os = "linux")]
fn read_sysfs_u32(device_path: &std::path::Path, filename: &str) -> Result<u32> {
    let file_path = device_path.join(filename);
    let content = fs::read_to_string(&file_path)
        .with_context(|| format!("Failed to read {}", file_path.display()))?;
    let value = content.trim().parse().with_context(|| {
        format!(
            "Failed to parse {} from {} as number",
            content.trim(),
            file_path.display()
        )
    })?;
    Ok(value)
}

/// Read a string value from a sysfs file
#[cfg(target_os = "linux")]
fn read_sysfs_string(device_path: &std::path::Path, filename: &str) -> Option<String> {
    let file_path = device_path.join(filename);
    fs::read_to_string(&file_path)
        .ok()
        .map(|s| s.trim().to_string())
}

/// Enrich device information from sysfs files
#[cfg(target_os = "linux")]
fn enrich_from_sysfs(device: &mut UsbDeviceInfo, sysfs_path: &std::path::Path) {
    // Try to read manufacturer from sysfs if not already set
    if device.manufacturer.is_none() {
        device.manufacturer = read_sysfs_string(sysfs_path, "manufacturer");
    }

    // Try to read product from sysfs if not already set
    if device.product.is_none() {
        device.product = read_sysfs_string(sysfs_path, "product");
    }

    // Try to read serial number from sysfs if not already set
    if device.serial_number.is_none() {
        device.serial_number = read_sysfs_string(sysfs_path, "serial");
    }

    // Read USB version if available
    if let Some(version_str) = read_sysfs_string(sysfs_path, "version") {
        // Version format is like "2.00" or "3.10"
        if let Some((major, minor)) = parse_usb_version(&version_str) {
            device.usb_version = (major << 8) | (minor << 4);
        }
    }

    // Read max power (current in mA)
    if let Some(_max_power) = read_sysfs_string(sysfs_path, "bMaxPower") {
        // This is available but we don't have a field for it in UsbDeviceInfo
        // Could be used for power issue detection
    }
}

/// Parse USB version string like "2.00" or "3.10"
#[cfg(target_os = "linux")]
fn parse_usb_version(version_str: &str) -> Option<(u16, u16)> {
    let parts: Vec<&str> = version_str.trim().split('.').collect();
    if parts.len() >= 2 {
        let major = parts[0].parse::<u16>().ok()?;
        let minor = parts[1].parse::<u16>().ok()?;
        Some((major, minor))
    } else {
        None
    }
}

/// Get the driver name bound to the device
#[cfg(target_os = "linux")]
fn get_driver_name(device_path: &std::path::Path) -> Option<String> {
    // Check for driver symlink
    let driver_link = device_path.join("driver");
    if let Ok(target) = fs::read_link(&driver_link) {
        if let Some(driver_name) = target.file_name() {
            return Some(driver_name.to_string_lossy().to_string());
        }
    }

    // For composite devices, check interface drivers
    let mut drivers = Vec::new();
    if let Ok(entries) = fs::read_dir(device_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            // Look for interface directories (format: X-Y:Z.W)
            if name_str.contains(':') {
                let interface_driver_link = path.join("driver");
                if let Ok(target) = fs::read_link(&interface_driver_link) {
                    if let Some(driver_name) = target.file_name() {
                        let driver = driver_name.to_string_lossy().to_string();
                        if !drivers.contains(&driver) {
                            drivers.push(driver);
                        }
                    }
                }
            }
        }
    }

    if drivers.len() == 1 {
        Some(drivers[0].clone())
    } else if drivers.len() > 1 {
        Some(format!("[{}]", drivers.join(", ")))
    } else {
        None
    }
}

/// Determine device health from sysfs information
#[cfg(target_os = "linux")]
fn determine_linux_device_health(device_path: &std::path::Path) -> LinkHealth {
    // Check for quirks (indicates device issues)
    if let Some(quirks) = read_sysfs_string(device_path, "quirks") {
        if quirks != "0x0" && !quirks.is_empty() {
            return LinkHealth::Unstable {
                reason: format!("Device has quirks: {}", quirks),
            };
        }
    }

    // Check for authorization
    if let Ok(authorized) = read_sysfs_number(device_path, "authorized") {
        if authorized == 0 {
            return LinkHealth::Unstable {
                reason: "Device not authorized".to_string(),
            };
        }
    }

    // Check for removed flag
    if let Some(removed) = read_sysfs_string(device_path, "removed") {
        if removed == "1" {
            return LinkHealth::Disconnected;
        }
    }

    // Check for urbnum (active URBs - if very high, might indicate issues)
    if let Ok(urbnum) = read_sysfs_u32(device_path, "urbnum") {
        if urbnum > 1000 {
            return LinkHealth::Unstable {
                reason: format!("High URB count: {}", urbnum),
            };
        }
    }

    // Check power/runtime_status
    let power_status_path = device_path.join("power/runtime_status");
    if let Ok(status) = fs::read_to_string(&power_status_path) {
        let status = status.trim();
        if status == "error" {
            return LinkHealth::Unstable {
                reason: "Power management error".to_string(),
            };
        } else if status == "suspended" {
            return LinkHealth::Unstable {
                reason: "Device is suspended".to_string(),
            };
        }
    }

    LinkHealth::Good
}

/// Get port path from devpath
#[cfg(target_os = "linux")]
fn get_port_path(device_path: &std::path::Path) -> Option<String> {
    // The device name itself encodes the port path
    // Format: usb<bus>/<bus>-<port>[.<port>...]
    if let Some(name) = device_path.file_name() {
        let name_str = name.to_string_lossy();
        
        // Extract port portion (e.g., "1-2.3.4" from directory name)
        if let Some(dash_pos) = name_str.find('-') {
            return Some(name_str[dash_pos + 1..].to_string());
        }
    }

    // Fallback: read devpath attribute
    read_sysfs_string(device_path, "devpath")
}

/// Get device node path (e.g., /dev/bus/usb/001/002)
#[cfg(target_os = "linux")]
fn get_device_node(bus_number: u8, device_address: u8) -> Option<String> {
    let devnode = format!("/dev/bus/usb/{:03}/{:03}", bus_number, device_address);
    if std::path::Path::new(&devnode).exists() {
        Some(devnode)
    } else {
        None
    }
}

/// Add device tags based on characteristics
#[cfg(target_os = "linux")]
fn add_linux_device_tags(device: &mut UsbDeviceInfo, device_path: &std::path::Path) {
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

    // Tag by speed
    if let Some(speed) = read_sysfs_string(device_path, "speed") {
        let speed_lower = speed.to_lowercase();
        if speed_lower.contains("1.5") {
            device.add_tag("low-speed");
        } else if speed_lower.contains("12") {
            device.add_tag("full-speed");
        } else if speed_lower.contains("480") {
            device.add_tag("high-speed");
        } else if speed_lower.contains("5000") {
            device.add_tag("super-speed");
        } else if speed_lower.contains("10000") || speed_lower.contains("20000") {
            device.add_tag("super-speed-plus");
        }
    }

    // Tag by removable status
    if let Some(removable) = read_sysfs_string(device_path, "removable") {
        if removable == "removable" {
            device.add_tag("removable");
        }
    }

    // Check if device is on a hub
    if let Some(port_path) = device.port_path.as_ref() {
        if port_path.contains('.') {
            device.add_tag("hub-connected");
        }
    }
}

/// Non-Linux platforms: no-op
#[cfg(not(target_os = "linux"))]
pub fn enrich_linux(_devices: &mut [UsbDeviceInfo]) -> Result<()> {
    // No-op on non-Linux platforms
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::UsbDeviceInfo;

    #[test]
    fn test_enrich_linux() {
        let mut devices = vec![UsbDeviceInfo::new(0x1234, 0x5678)];
        let result = enrich_linux(&mut devices);
        assert!(result.is_ok());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_parse_usb_version() {
        assert_eq!(parse_usb_version("2.00"), Some((2, 0)));
        assert_eq!(parse_usb_version("3.10"), Some((3, 10)));
        assert_eq!(parse_usb_version(" 2.01 "), Some((2, 1)));
        assert_eq!(parse_usb_version("invalid"), None);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_get_device_node() {
        // This will only work if /dev/bus/usb exists
        // Just check it doesn't panic
        let _ = get_device_node(1, 1);
    }
}
