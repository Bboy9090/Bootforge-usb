//! Device Permissions Helper - OMEGA MODE
//!
//! Cross-platform helpers for managing USB device permissions.

use crate::model::UsbDeviceRecord;

/// Permission check result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionStatus {
    /// Full access granted
    Granted,
    /// Read-only access
    ReadOnly,
    /// Access denied - need elevated privileges
    NeedsElevation,
    /// Access denied - need udev rule (Linux)
    NeedsUdevRule,
    /// Access denied - device is in use
    DeviceBusy,
    /// Access denied - unknown reason
    Denied(String),
    /// Unable to determine permission status
    Unknown,
}

impl PermissionStatus {
    /// Check if access is available
    pub fn has_access(&self) -> bool {
        matches!(self, Self::Granted | Self::ReadOnly)
    }
    
    /// Get a human-readable description
    pub fn description(&self) -> &str {
        match self {
            Self::Granted => "Full access granted",
            Self::ReadOnly => "Read-only access",
            Self::NeedsElevation => "Requires administrator/root privileges",
            Self::NeedsUdevRule => "Requires udev rule for user access",
            Self::DeviceBusy => "Device is in use by another process",
            Self::Denied(_) => "Access denied",
            Self::Unknown => "Permission status unknown",
        }
    }
}

/// Permission helper for checking and fixing device access
pub struct PermissionHelper;

impl PermissionHelper {
    /// Check permission status for a device
    pub fn check(device: &UsbDeviceRecord) -> PermissionStatus {
        #[cfg(target_os = "linux")]
        return Self::check_linux(device);
        
        #[cfg(target_os = "macos")]
        return Self::check_macos(device);
        
        #[cfg(target_os = "windows")]
        return Self::check_windows(device);
        
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        PermissionStatus::Unknown
    }
    
    /// Check if we can open a device by VID:PID
    pub fn can_open(vid: u16, pid: u16) -> PermissionStatus {
        match rusb::open_device_with_vid_pid(vid, pid) {
            Some(handle) => {
                // Try to get active configuration to verify full access
                match handle.active_configuration() {
                    Ok(_) => PermissionStatus::Granted,
                    Err(rusb::Error::Access) => PermissionStatus::ReadOnly,
                    Err(rusb::Error::Busy) => PermissionStatus::DeviceBusy,
                    Err(_) => PermissionStatus::ReadOnly,
                }
            }
            None => {
                // Device not found or permission denied
                // Try to enumerate to distinguish
                if let Ok(devices) = rusb::devices() {
                    for dev in devices.iter() {
                        if let Ok(desc) = dev.device_descriptor() {
                            if desc.vendor_id() == vid && desc.product_id() == pid {
                                // Device exists but we can't open it
                                #[cfg(target_os = "linux")]
                                return PermissionStatus::NeedsUdevRule;
                                
                                #[cfg(not(target_os = "linux"))]
                                return PermissionStatus::NeedsElevation;
                            }
                        }
                    }
                }
                PermissionStatus::Denied("Device not accessible".into())
            }
        }
    }
    
    #[cfg(target_os = "linux")]
    fn check_linux(device: &UsbDeviceRecord) -> PermissionStatus {
        use std::fs;
        use std::os::unix::fs::MetadataExt;
        
        // Build the device path
        let (bus, addr) = match (device.location.bus, device.location.address) {
            (Some(b), Some(a)) => (b, a),
            _ => return PermissionStatus::Unknown,
        };
        
        let dev_path = format!("/dev/bus/usb/{:03}/{:03}", bus, addr);
        
        // Check if device file exists and is readable
        match fs::metadata(&dev_path) {
            Ok(meta) => {
                let mode = meta.mode();
                let uid = unsafe { libc::getuid() };
                let gid = unsafe { libc::getgid() };
                
                // Check user permissions
                if meta.uid() == uid {
                    if mode & 0o600 == 0o600 {
                        return PermissionStatus::Granted;
                    } else if mode & 0o400 == 0o400 {
                        return PermissionStatus::ReadOnly;
                    }
                }
                
                // Check group permissions
                if meta.gid() == gid {
                    if mode & 0o060 == 0o060 {
                        return PermissionStatus::Granted;
                    } else if mode & 0o040 == 0o040 {
                        return PermissionStatus::ReadOnly;
                    }
                }
                
                // Check other permissions
                if mode & 0o006 == 0o006 {
                    return PermissionStatus::Granted;
                } else if mode & 0o004 == 0o004 {
                    return PermissionStatus::ReadOnly;
                }
                
                // Need udev rule for access
                PermissionStatus::NeedsUdevRule
            }
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                PermissionStatus::NeedsUdevRule
            }
            Err(_) => PermissionStatus::Unknown,
        }
    }
    
    #[cfg(target_os = "macos")]
    fn check_macos(device: &UsbDeviceRecord) -> PermissionStatus {
        // On macOS, try to open the device
        Self::can_open(device.id.vid, device.id.pid)
    }
    
    #[cfg(target_os = "windows")]
    fn check_windows(device: &UsbDeviceRecord) -> PermissionStatus {
        // On Windows, try to open the device
        Self::can_open(device.id.vid, device.id.pid)
    }
    
    /// Generate a udev rule for a device (Linux only)
    #[cfg(target_os = "linux")]
    pub fn generate_udev_rule(vid: u16, pid: u16, mode: &str, group: Option<&str>) -> String {
        let group_part = group.map(|g| format!(", GROUP=\"{}\"", g)).unwrap_or_default();
        format!(
            "SUBSYSTEM==\"usb\", ATTR{{idVendor}}==\"{:04x}\", ATTR{{idProduct}}==\"{:04x}\", MODE=\"{}\"{}\n",
            vid, pid, mode, group_part
        )
    }
    
    #[cfg(not(target_os = "linux"))]
    pub fn generate_udev_rule(_vid: u16, _pid: u16, _mode: &str, _group: Option<&str>) -> String {
        String::new()
    }
    
    /// Generate udev rules for common devices
    #[cfg(target_os = "linux")]
    pub fn generate_common_rules() -> String {
        let mut rules = String::new();
        rules.push_str("# BootForge USB - Common device rules\n\n");
        
        // Android devices (ADB/Fastboot)
        rules.push_str("# Android (ADB/Fastboot)\n");
        for vid in [0x18D1, 0x04E8, 0x22B8, 0x0BB4, 0x12D1, 0x2717] {
            rules.push_str(&format!(
                "SUBSYSTEM==\"usb\", ATTR{{idVendor}}==\"{:04x}\", MODE=\"0666\"\n",
                vid
            ));
        }
        
        rules.push_str("\n# Development boards\n");
        for vid in [0x2341, 0x2E8A, 0x239A, 0x0483, 0x1366] {
            rules.push_str(&format!(
                "SUBSYSTEM==\"usb\", ATTR{{idVendor}}==\"{:04x}\", MODE=\"0666\"\n",
                vid
            ));
        }
        
        rules.push_str("\n# Serial adapters\n");
        rules.push_str("SUBSYSTEM==\"usb\", ATTR{idVendor}==\"0403\", MODE=\"0666\"\n"); // FTDI
        rules.push_str("SUBSYSTEM==\"usb\", ATTR{idVendor}==\"10c4\", MODE=\"0666\"\n"); // CP210x
        rules.push_str("SUBSYSTEM==\"usb\", ATTR{idVendor}==\"1a86\", MODE=\"0666\"\n"); // CH340
        
        rules
    }
    
    #[cfg(not(target_os = "linux"))]
    pub fn generate_common_rules() -> String {
        String::new()
    }
    
    /// Get installation instructions for a device
    pub fn get_instructions(device: &UsbDeviceRecord, status: &PermissionStatus) -> String {
        match status {
            PermissionStatus::Granted | PermissionStatus::ReadOnly => {
                "Device is accessible.".to_string()
            }
            PermissionStatus::NeedsElevation => {
                #[cfg(target_os = "windows")]
                return "Run your application as Administrator.".to_string();
                
                #[cfg(target_os = "macos")]
                return "Run your application with sudo, or grant USB access in System Preferences.".to_string();
                
                #[cfg(target_os = "linux")]
                return "Run your application with sudo.".to_string();
                
                #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
                "Requires elevated privileges.".to_string()
            }
            PermissionStatus::NeedsUdevRule => {
                format!(
                    "Create a udev rule:\n\
                    1. Create file /etc/udev/rules.d/99-bootforge.rules\n\
                    2. Add: SUBSYSTEM==\"usb\", ATTR{{idVendor}}==\"{:04x}\", ATTR{{idProduct}}==\"{:04x}\", MODE=\"0666\"\n\
                    3. Run: sudo udevadm control --reload-rules && sudo udevadm trigger\n\
                    4. Replug the device",
                    device.id.vid, device.id.pid
                )
            }
            PermissionStatus::DeviceBusy => {
                "Device is in use by another process. Close other applications using the device.".to_string()
            }
            PermissionStatus::Denied(reason) => {
                format!("Access denied: {}. Try running with elevated privileges.", reason)
            }
            PermissionStatus::Unknown => {
                "Unable to determine permission status. Try running with elevated privileges.".to_string()
            }
        }
    }
}

/// Platform-specific USB group information
pub struct UsbGroups;

impl UsbGroups {
    /// Get the recommended group for USB access
    #[cfg(target_os = "linux")]
    pub fn recommended_group() -> &'static str {
        // Check common group names
        "plugdev"
    }
    
    #[cfg(not(target_os = "linux"))]
    pub fn recommended_group() -> &'static str {
        ""
    }
    
    /// Check if user is in the USB group
    #[cfg(target_os = "linux")]
    pub fn user_in_group(group: &str) -> bool {
        use std::process::Command;
        
        if let Ok(output) = Command::new("groups").output() {
            if let Ok(groups) = String::from_utf8(output.stdout) {
                return groups.contains(group);
            }
        }
        false
    }
    
    #[cfg(not(target_os = "linux"))]
    pub fn user_in_group(_group: &str) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_status() {
        assert!(PermissionStatus::Granted.has_access());
        assert!(PermissionStatus::ReadOnly.has_access());
        assert!(!PermissionStatus::NeedsElevation.has_access());
    }

    #[test]
    fn test_descriptions() {
        assert!(!PermissionStatus::Granted.description().is_empty());
        assert!(!PermissionStatus::NeedsUdevRule.description().is_empty());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_udev_rule_generation() {
        let rule = PermissionHelper::generate_udev_rule(0x1234, 0x5678, "0666", Some("plugdev"));
        assert!(rule.contains("1234"));
        assert!(rule.contains("5678"));
        assert!(rule.contains("plugdev"));
    }
}
