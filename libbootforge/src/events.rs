//! USB device event monitoring

use crate::{device::DeviceInfo, enumeration::enumerate_devices, Result};
use std::collections::HashMap;
use std::time::Duration;

/// Device event types
#[derive(Debug, Clone, PartialEq)]
pub enum DeviceEvent {
    /// A new device was connected
    Connected(DeviceInfo),

    /// A device was disconnected
    Disconnected {
        vendor_id: u16,
        product_id: u16,
        bus_number: u8,
        device_address: u8,
    },
}

/// Monitor for USB device connection and disconnection events
pub struct DeviceEventMonitor {
    known_devices: HashMap<(u8, u8), DeviceInfo>,
}

impl DeviceEventMonitor {
    /// Create a new device event monitor
    pub fn new() -> Result<Self> {
        let devices = enumerate_devices()?;
        let mut known_devices = HashMap::new();

        for device in devices {
            known_devices.insert((device.bus_number, device.device_address), device);
        }

        Ok(Self { known_devices })
    }

    /// Poll for device events and return any changes
    pub fn poll(&mut self) -> Result<Vec<DeviceEvent>> {
        let current_devices = enumerate_devices()?;
        let mut events = Vec::new();

        // Create a map of current devices
        let mut current_map = HashMap::new();
        for device in current_devices {
            current_map.insert((device.bus_number, device.device_address), device);
        }

        // Detect new devices (connected)
        for (key, device) in &current_map {
            if !self.known_devices.contains_key(key) {
                events.push(DeviceEvent::Connected(device.clone()));
            }
        }

        // Detect removed devices (disconnected)
        for (key, device) in &self.known_devices {
            if !current_map.contains_key(key) {
                events.push(DeviceEvent::Disconnected {
                    vendor_id: device.vendor_id,
                    product_id: device.product_id,
                    bus_number: device.bus_number,
                    device_address: device.device_address,
                });
            }
        }

        // Update known devices
        self.known_devices = current_map;

        Ok(events)
    }

    /// Wait for device events with a timeout
    pub fn wait_for_events(&mut self, timeout: Duration) -> Result<Vec<DeviceEvent>> {
        let poll_interval = Duration::from_millis(100);
        let start = std::time::Instant::now();

        loop {
            let events = self.poll()?;
            if !events.is_empty() {
                return Ok(events);
            }

            if start.elapsed() >= timeout {
                return Ok(Vec::new());
            }

            std::thread::sleep(poll_interval);
        }
    }

    /// Get a list of currently known devices
    pub fn current_devices(&self) -> Vec<&DeviceInfo> {
        self.known_devices.values().collect()
    }
}

impl Default for DeviceEventMonitor {
    fn default() -> Self {
        Self::new().unwrap_or(Self {
            known_devices: HashMap::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires USB subsystem - not available in CI
    fn test_event_monitor_creation() {
        let result = DeviceEventMonitor::new();
        assert!(result.is_ok());
    }

    #[test]
    #[ignore] // Requires USB subsystem - not available in CI
    fn test_poll_returns_events() {
        let mut monitor = DeviceEventMonitor::new().unwrap();
        let result = monitor.poll();
        assert!(result.is_ok());
    }

    #[test]
    #[ignore] // Requires USB subsystem - not available in CI
    fn test_current_devices() {
        let monitor = DeviceEventMonitor::new().unwrap();
        let devices = monitor.current_devices();
        // Should not panic, number of devices is system-dependent
        let _ = devices.len();
    }
}
