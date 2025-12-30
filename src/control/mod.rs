//! Device Control - Reset, Power Cycle, and Management - OMEGA MODE
//!
//! Provides device control operations including reset, power cycle, and authorization.

use crate::errors::UsbError;
use crate::model::UsbDeviceRecord;
use std::time::Duration;

/// Device control operations
pub struct DeviceControl;

impl DeviceControl {
    /// Reset a USB device (soft reset via rusb)
    pub fn reset(vid: u16, pid: u16) -> Result<(), UsbError> {
        let handle = rusb::open_device_with_vid_pid(vid, pid)
            .ok_or_else(|| UsbError::DeviceNotFound(format!("{:04X}:{:04X}", vid, pid)))?;
        
        handle.reset()?;
        Ok(())
    }
    
    /// Reset device by bus and address
    pub fn reset_by_address(bus: u8, address: u8) -> Result<(), UsbError> {
        let devices = rusb::devices()?;
        
        for device in devices.iter() {
            if device.bus_number() == bus && device.address() == address {
                let handle = device.open()?;
                handle.reset()?;
                return Ok(());
            }
        }
        
        Err(UsbError::DeviceNotFound(format!("Bus {} Address {}", bus, address)))
    }
    
    /// Soft reset via control transfer (device-specific)
    pub fn soft_reset(vid: u16, pid: u16) -> Result<(), UsbError> {
        // This is a USB standard device request
        let handle = rusb::open_device_with_vid_pid(vid, pid)
            .ok_or_else(|| UsbError::DeviceNotFound(format!("{:04X}:{:04X}", vid, pid)))?;
        
        // Some devices support a vendor-specific reset
        // This is a common pattern but may not work on all devices
        let _ = handle.write_control(
            0x40, // bmRequestType: vendor, device
            0xFF, // bRequest: vendor-specific reset (common)
            0,    // wValue
            0,    // wIndex
            &[],
            Duration::from_secs(1),
        );
        
        Ok(())
    }
    
    /// Power cycle a USB port (Linux only via sysfs)
    #[cfg(target_os = "linux")]
    pub fn power_cycle(device: &UsbDeviceRecord) -> Result<(), UsbError> {
        use std::fs;
        use std::thread::sleep;
        
        let port_path = device.location.port_path.as_ref()
            .ok_or_else(|| UsbError::Platform("No port path available".into()))?;
        
        // Find the USB device path in sysfs
        let sysfs_path = format!("/sys/bus/usb/devices/{}", port_path);
        let auth_path = format!("{}/authorized", sysfs_path);
        
        // Check if we can control authorization
        if !std::path::Path::new(&auth_path).exists() {
            return Err(UsbError::Platform("Authorization control not available".into()));
        }
        
        // Deauthorize (disconnect)
        fs::write(&auth_path, "0")
            .map_err(|e| UsbError::Platform(format!("Failed to deauthorize: {}", e)))?;
        
        // Wait for device to be removed
        sleep(Duration::from_millis(500));
        
        // Reauthorize (reconnect)
        fs::write(&auth_path, "1")
            .map_err(|e| UsbError::Platform(format!("Failed to reauthorize: {}", e)))?;
        
        Ok(())
    }
    
    #[cfg(not(target_os = "linux"))]
    pub fn power_cycle(_device: &UsbDeviceRecord) -> Result<(), UsbError> {
        Err(UsbError::Platform("Power cycle not supported on this platform".into()))
    }
    
    /// Unbind driver from device (Linux only)
    #[cfg(target_os = "linux")]
    pub fn unbind_driver(device: &UsbDeviceRecord) -> Result<(), UsbError> {
        use std::fs;
        
        let port_path = device.location.port_path.as_ref()
            .ok_or_else(|| UsbError::Platform("No port path available".into()))?;
        
        let unbind_path = "/sys/bus/usb/drivers/usb/unbind";
        
        fs::write(unbind_path, port_path)
            .map_err(|e| UsbError::Platform(format!("Failed to unbind: {}", e)))?;
        
        Ok(())
    }
    
    #[cfg(not(target_os = "linux"))]
    pub fn unbind_driver(_device: &UsbDeviceRecord) -> Result<(), UsbError> {
        Err(UsbError::Platform("Driver unbind not supported on this platform".into()))
    }
    
    /// Rebind driver to device (Linux only)
    #[cfg(target_os = "linux")]
    pub fn rebind_driver(device: &UsbDeviceRecord) -> Result<(), UsbError> {
        use std::fs;
        
        let port_path = device.location.port_path.as_ref()
            .ok_or_else(|| UsbError::Platform("No port path available".into()))?;
        
        let bind_path = "/sys/bus/usb/drivers/usb/bind";
        
        fs::write(bind_path, port_path)
            .map_err(|e| UsbError::Platform(format!("Failed to bind: {}", e)))?;
        
        Ok(())
    }
    
    #[cfg(not(target_os = "linux"))]
    pub fn rebind_driver(_device: &UsbDeviceRecord) -> Result<(), UsbError> {
        Err(UsbError::Platform("Driver bind not supported on this platform".into()))
    }
    
    /// Authorize a device (Linux only - for USB authorization)
    #[cfg(target_os = "linux")]
    pub fn authorize(device: &UsbDeviceRecord, authorized: bool) -> Result<(), UsbError> {
        use std::fs;
        
        let port_path = device.location.port_path.as_ref()
            .ok_or_else(|| UsbError::Platform("No port path available".into()))?;
        
        let auth_path = format!("/sys/bus/usb/devices/{}/authorized", port_path);
        let value = if authorized { "1" } else { "0" };
        
        fs::write(&auth_path, value)
            .map_err(|e| UsbError::Platform(format!("Failed to set authorization: {}", e)))?;
        
        Ok(())
    }
    
    #[cfg(not(target_os = "linux"))]
    pub fn authorize(_device: &UsbDeviceRecord, _authorized: bool) -> Result<(), UsbError> {
        Err(UsbError::Platform("Authorization control not supported on this platform".into()))
    }
    
    /// Suspend a device (Linux only - put into low power state)
    #[cfg(target_os = "linux")]
    pub fn suspend(device: &UsbDeviceRecord) -> Result<(), UsbError> {
        use std::fs;
        
        let port_path = device.location.port_path.as_ref()
            .ok_or_else(|| UsbError::Platform("No port path available".into()))?;
        
        let power_path = format!("/sys/bus/usb/devices/{}/power/control", port_path);
        
        fs::write(&power_path, "auto")
            .map_err(|e| UsbError::Platform(format!("Failed to set power control: {}", e)))?;
        
        Ok(())
    }
    
    #[cfg(not(target_os = "linux"))]
    pub fn suspend(_device: &UsbDeviceRecord) -> Result<(), UsbError> {
        Err(UsbError::Platform("Power control not supported on this platform".into()))
    }
    
    /// Resume a device (Linux only - bring out of low power state)
    #[cfg(target_os = "linux")]
    pub fn resume(device: &UsbDeviceRecord) -> Result<(), UsbError> {
        use std::fs;
        
        let port_path = device.location.port_path.as_ref()
            .ok_or_else(|| UsbError::Platform("No port path available".into()))?;
        
        let power_path = format!("/sys/bus/usb/devices/{}/power/control", port_path);
        
        fs::write(&power_path, "on")
            .map_err(|e| UsbError::Platform(format!("Failed to set power control: {}", e)))?;
        
        Ok(())
    }
    
    #[cfg(not(target_os = "linux"))]
    pub fn resume(_device: &UsbDeviceRecord) -> Result<(), UsbError> {
        Err(UsbError::Platform("Power control not supported on this platform".into()))
    }
    
    /// Get power state (Linux only)
    #[cfg(target_os = "linux")]
    pub fn power_state(device: &UsbDeviceRecord) -> Result<PowerState, UsbError> {
        use std::fs;
        
        let port_path = device.location.port_path.as_ref()
            .ok_or_else(|| UsbError::Platform("No port path available".into()))?;
        
        let runtime_status = format!("/sys/bus/usb/devices/{}/power/runtime_status", port_path);
        
        let status = fs::read_to_string(&runtime_status)
            .map_err(|e| UsbError::Platform(format!("Failed to read power state: {}", e)))?;
        
        Ok(match status.trim() {
            "active" => PowerState::Active,
            "suspended" => PowerState::Suspended,
            "suspending" => PowerState::Suspending,
            "resuming" => PowerState::Resuming,
            _ => PowerState::Unknown,
        })
    }
    
    #[cfg(not(target_os = "linux"))]
    pub fn power_state(_device: &UsbDeviceRecord) -> Result<PowerState, UsbError> {
        Ok(PowerState::Unknown)
    }
}

/// Device power state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerState {
    /// Device is active
    Active,
    /// Device is suspended
    Suspended,
    /// Device is suspending
    Suspending,
    /// Device is resuming
    Resuming,
    /// State unknown
    Unknown,
}

impl PowerState {
    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Active => "Active",
            Self::Suspended => "Suspended",
            Self::Suspending => "Suspending",
            Self::Resuming => "Resuming",
            Self::Unknown => "Unknown",
        }
    }
}

/// Hub control operations
pub struct HubControl;

impl HubControl {
    /// Get port status (feature for hub ports)
    pub fn get_port_status(hub_vid: u16, hub_pid: u16, port: u8) -> Result<PortStatus, UsbError> {
        let handle = rusb::open_device_with_vid_pid(hub_vid, hub_pid)
            .ok_or_else(|| UsbError::DeviceNotFound("Hub not found".into()))?;
        
        let mut buf = [0u8; 4];
        
        // GET_STATUS request for hub port
        let request_type = 0xA3; // Device to host, class, other
        let request = 0x00; // GET_STATUS
        
        handle.read_control(
            request_type,
            request,
            0,
            port as u16,
            &mut buf,
            Duration::from_secs(1),
        )?;
        
        Ok(PortStatus::from_bytes(&buf))
    }
    
    /// Power on a hub port
    pub fn power_on_port(hub_vid: u16, hub_pid: u16, port: u8) -> Result<(), UsbError> {
        Self::set_port_feature(hub_vid, hub_pid, port, PortFeature::Power)
    }
    
    /// Power off a hub port
    pub fn power_off_port(hub_vid: u16, hub_pid: u16, port: u8) -> Result<(), UsbError> {
        Self::clear_port_feature(hub_vid, hub_pid, port, PortFeature::Power)
    }
    
    /// Reset a hub port
    pub fn reset_port(hub_vid: u16, hub_pid: u16, port: u8) -> Result<(), UsbError> {
        Self::set_port_feature(hub_vid, hub_pid, port, PortFeature::Reset)
    }
    
    /// Set a port feature
    fn set_port_feature(hub_vid: u16, hub_pid: u16, port: u8, feature: PortFeature) -> Result<(), UsbError> {
        let handle = rusb::open_device_with_vid_pid(hub_vid, hub_pid)
            .ok_or_else(|| UsbError::DeviceNotFound("Hub not found".into()))?;
        
        let request_type = 0x23; // Host to device, class, other
        let request = 0x03; // SET_FEATURE
        
        handle.write_control(
            request_type,
            request,
            feature as u16,
            port as u16,
            &[],
            Duration::from_secs(1),
        )?;
        
        Ok(())
    }
    
    /// Clear a port feature
    fn clear_port_feature(hub_vid: u16, hub_pid: u16, port: u8, feature: PortFeature) -> Result<(), UsbError> {
        let handle = rusb::open_device_with_vid_pid(hub_vid, hub_pid)
            .ok_or_else(|| UsbError::DeviceNotFound("Hub not found".into()))?;
        
        let request_type = 0x23; // Host to device, class, other
        let request = 0x01; // CLEAR_FEATURE
        
        handle.write_control(
            request_type,
            request,
            feature as u16,
            port as u16,
            &[],
            Duration::from_secs(1),
        )?;
        
        Ok(())
    }
}

/// Hub port features
#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum PortFeature {
    /// Port connection
    Connection = 0,
    /// Port enable
    Enable = 1,
    /// Port suspend
    Suspend = 2,
    /// Port overcurrent
    OverCurrent = 3,
    /// Port reset
    Reset = 4,
    /// Port power
    Power = 8,
    /// Port low speed
    LowSpeed = 9,
    /// Port connect change
    ConnectChange = 16,
    /// Port enable change
    EnableChange = 17,
    /// Port suspend change
    SuspendChange = 18,
    /// Port overcurrent change
    OverCurrentChange = 19,
    /// Port reset change
    ResetChange = 20,
}

/// Hub port status
#[derive(Debug, Clone)]
pub struct PortStatus {
    /// Device is connected
    pub connected: bool,
    /// Port is enabled
    pub enabled: bool,
    /// Port is suspended
    pub suspended: bool,
    /// Overcurrent condition
    pub overcurrent: bool,
    /// Port is in reset
    pub reset: bool,
    /// Port is powered
    pub powered: bool,
    /// Low speed device
    pub low_speed: bool,
    /// High speed device
    pub high_speed: bool,
    /// Connection changed
    pub connect_change: bool,
    /// Enable changed
    pub enable_change: bool,
    /// Suspend changed
    pub suspend_change: bool,
    /// Overcurrent changed
    pub overcurrent_change: bool,
    /// Reset changed
    pub reset_change: bool,
}

impl PortStatus {
    fn from_bytes(data: &[u8; 4]) -> Self {
        let status = u16::from_le_bytes([data[0], data[1]]);
        let changes = u16::from_le_bytes([data[2], data[3]]);
        
        Self {
            connected: (status & 0x0001) != 0,
            enabled: (status & 0x0002) != 0,
            suspended: (status & 0x0004) != 0,
            overcurrent: (status & 0x0008) != 0,
            reset: (status & 0x0010) != 0,
            powered: (status & 0x0100) != 0,
            low_speed: (status & 0x0200) != 0,
            high_speed: (status & 0x0400) != 0,
            connect_change: (changes & 0x0001) != 0,
            enable_change: (changes & 0x0002) != 0,
            suspend_change: (changes & 0x0004) != 0,
            overcurrent_change: (changes & 0x0008) != 0,
            reset_change: (changes & 0x0010) != 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_state() {
        assert_eq!(PowerState::Active.name(), "Active");
        assert_eq!(PowerState::Suspended.name(), "Suspended");
    }

    #[test]
    fn test_port_status() {
        let data = [0x03, 0x01, 0x01, 0x00];
        let status = PortStatus::from_bytes(&data);
        assert!(status.connected);
        assert!(status.enabled);
        assert!(status.powered);
        assert!(status.connect_change);
    }
}
