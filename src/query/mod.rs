//! Device Query API - OMEGA MODE
//!
//! Rich filtering and search capabilities for USB devices.

use crate::model::{UsbDeviceRecord, DriverStatus, LinkHealth};
use crate::handshake::DeviceProtocol;
use std::collections::HashSet;

/// Device query builder for filtering USB devices
#[derive(Debug, Clone, Default)]
pub struct DeviceQuery {
    /// Filter by vendor IDs
    vendor_ids: Option<HashSet<u16>>,
    /// Filter by product IDs
    product_ids: Option<HashSet<u16>>,
    /// Filter by VID:PID pairs
    device_ids: Option<HashSet<(u16, u16)>>,
    /// Filter by device class
    device_classes: Option<HashSet<u8>>,
    /// Filter by bus number
    bus_numbers: Option<HashSet<u8>>,
    /// Filter by manufacturer (contains, case-insensitive)
    manufacturer_contains: Option<String>,
    /// Filter by product name (contains, case-insensitive)
    product_contains: Option<String>,
    /// Filter by serial number (exact or prefix)
    serial_filter: Option<SerialFilter>,
    /// Filter by driver status
    driver_status: Option<DriverStatusFilter>,
    /// Filter by health status
    health_status: Option<HealthFilter>,
    /// Filter by protocol
    protocols: Option<HashSet<DeviceProtocol>>,
    /// Filter by tags
    tags: Option<HashSet<String>>,
    /// Has serial number
    has_serial: Option<bool>,
    /// USB version filter
    usb_version: Option<String>,
    /// Limit results
    limit: Option<usize>,
}

/// Serial number filter options
#[derive(Debug, Clone)]
pub enum SerialFilter {
    /// Exact match
    Exact(String),
    /// Starts with prefix
    Prefix(String),
    /// Contains substring
    Contains(String),
}

/// Driver status filter
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriverStatusFilter {
    /// Any bound driver
    Bound,
    /// Missing driver
    Missing,
    /// Blocked
    Blocked,
    /// Multiple drivers
    Multiple,
    /// Unknown status
    Unknown,
}

/// Health status filter
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthFilter {
    /// Good health
    Good,
    /// Any issue
    HasIssue,
    /// Disconnected
    Disconnected,
}

impl DeviceQuery {
    /// Create a new empty query (matches all devices)
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Filter by vendor ID
    pub fn vendor_id(mut self, vid: u16) -> Self {
        self.vendor_ids.get_or_insert_with(HashSet::new).insert(vid);
        self
    }
    
    /// Filter by multiple vendor IDs
    pub fn vendor_ids(mut self, vids: impl IntoIterator<Item = u16>) -> Self {
        let set = self.vendor_ids.get_or_insert_with(HashSet::new);
        set.extend(vids);
        self
    }
    
    /// Filter by product ID
    pub fn product_id(mut self, pid: u16) -> Self {
        self.product_ids.get_or_insert_with(HashSet::new).insert(pid);
        self
    }
    
    /// Filter by VID:PID pair
    pub fn device_id(mut self, vid: u16, pid: u16) -> Self {
        self.device_ids.get_or_insert_with(HashSet::new).insert((vid, pid));
        self
    }
    
    /// Filter by device class
    pub fn device_class(mut self, class: u8) -> Self {
        self.device_classes.get_or_insert_with(HashSet::new).insert(class);
        self
    }
    
    /// Filter by bus number
    pub fn bus(mut self, bus: u8) -> Self {
        self.bus_numbers.get_or_insert_with(HashSet::new).insert(bus);
        self
    }
    
    /// Filter by manufacturer name (case-insensitive contains)
    pub fn manufacturer(mut self, name: &str) -> Self {
        self.manufacturer_contains = Some(name.to_lowercase());
        self
    }
    
    /// Filter by product name (case-insensitive contains)
    pub fn product(mut self, name: &str) -> Self {
        self.product_contains = Some(name.to_lowercase());
        self
    }
    
    /// Filter by exact serial number
    pub fn serial(mut self, serial: &str) -> Self {
        self.serial_filter = Some(SerialFilter::Exact(serial.to_string()));
        self
    }
    
    /// Filter by serial number prefix
    pub fn serial_prefix(mut self, prefix: &str) -> Self {
        self.serial_filter = Some(SerialFilter::Prefix(prefix.to_string()));
        self
    }
    
    /// Filter by serial number contains
    pub fn serial_contains(mut self, substring: &str) -> Self {
        self.serial_filter = Some(SerialFilter::Contains(substring.to_string()));
        self
    }
    
    /// Filter devices that have a serial number
    pub fn has_serial(mut self, has: bool) -> Self {
        self.has_serial = Some(has);
        self
    }
    
    /// Filter by driver status
    pub fn driver_bound(mut self) -> Self {
        self.driver_status = Some(DriverStatusFilter::Bound);
        self
    }
    
    /// Filter devices with missing drivers
    pub fn driver_missing(mut self) -> Self {
        self.driver_status = Some(DriverStatusFilter::Missing);
        self
    }
    
    /// Filter by good health
    pub fn healthy(mut self) -> Self {
        self.health_status = Some(HealthFilter::Good);
        self
    }
    
    /// Filter devices with issues
    pub fn has_issues(mut self) -> Self {
        self.health_status = Some(HealthFilter::HasIssue);
        self
    }
    
    /// Filter by protocol
    pub fn protocol(mut self, protocol: DeviceProtocol) -> Self {
        self.protocols.get_or_insert_with(HashSet::new).insert(protocol);
        self
    }
    
    /// Filter by tag
    pub fn tag(mut self, tag: &str) -> Self {
        self.tags.get_or_insert_with(HashSet::new).insert(tag.to_lowercase());
        self
    }
    
    /// Filter by USB version
    pub fn usb_version(mut self, version: &str) -> Self {
        self.usb_version = Some(version.to_string());
        self
    }
    
    /// Limit number of results
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
    
    /// Check if a device matches the query
    pub fn matches(&self, device: &UsbDeviceRecord) -> bool {
        // Vendor ID filter
        if let Some(ref vids) = self.vendor_ids {
            if !vids.contains(&device.id.vid) {
                return false;
            }
        }
        
        // Product ID filter
        if let Some(ref pids) = self.product_ids {
            if !pids.contains(&device.id.pid) {
                return false;
            }
        }
        
        // VID:PID pair filter
        if let Some(ref pairs) = self.device_ids {
            if !pairs.contains(&(device.id.vid, device.id.pid)) {
                return false;
            }
        }
        
        // Device class filter
        if let Some(ref classes) = self.device_classes {
            if let Some(class) = device.descriptor.device_class {
                if !classes.contains(&class) {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        // Bus filter
        if let Some(ref buses) = self.bus_numbers {
            if let Some(bus) = device.location.bus {
                if !buses.contains(&bus) {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        // Manufacturer filter
        if let Some(ref mfg) = self.manufacturer_contains {
            match &device.descriptor.manufacturer {
                Some(m) if m.to_lowercase().contains(mfg) => {}
                _ => return false,
            }
        }
        
        // Product filter
        if let Some(ref prod) = self.product_contains {
            match &device.descriptor.product {
                Some(p) if p.to_lowercase().contains(prod) => {}
                _ => return false,
            }
        }
        
        // Serial filter
        if let Some(ref filter) = self.serial_filter {
            match (&device.descriptor.serial_number, filter) {
                (Some(serial), SerialFilter::Exact(s)) if serial == s => {}
                (Some(serial), SerialFilter::Prefix(p)) if serial.starts_with(p) => {}
                (Some(serial), SerialFilter::Contains(c)) if serial.contains(c) => {}
                _ => return false,
            }
        }
        
        // Has serial filter
        if let Some(has) = self.has_serial {
            let device_has = device.descriptor.serial_number.is_some();
            if has != device_has {
                return false;
            }
        }
        
        // Driver status filter
        if let Some(ref status) = self.driver_status {
            if !matches!(
                (&device.driver, status),
                (DriverStatus::Bound { .. }, DriverStatusFilter::Bound)
                    | (DriverStatus::Missing, DriverStatusFilter::Missing)
                    | (DriverStatus::Blocked { .. }, DriverStatusFilter::Blocked)
                    | (DriverStatus::Multiple { .. }, DriverStatusFilter::Multiple)
                    | (DriverStatus::Unknown, DriverStatusFilter::Unknown)
            ) {
                return false;
            }
        }
        
        // Health filter
        if let Some(ref health) = self.health_status {
            let matches = match (&device.health, health) {
                (LinkHealth::Good, HealthFilter::Good) => true,
                (LinkHealth::Disconnected, HealthFilter::Disconnected) => true,
                (LinkHealth::Good, HealthFilter::HasIssue) => false,
                (_, HealthFilter::HasIssue) => true,
                _ => false,
            };
            if !matches {
                return false;
            }
        }
        
        // Tags filter
        if let Some(ref tags) = self.tags {
            for tag in tags {
                if !device.has_tag(tag) {
                    return false;
                }
            }
        }
        
        // USB version filter
        if let Some(ref version) = self.usb_version {
            match &device.descriptor.usb_version {
                Some(v) if v.contains(version) => {}
                _ => return false,
            }
        }
        
        true
    }
    
    /// Filter a list of devices
    pub fn filter<'a>(&self, devices: &'a [UsbDeviceRecord]) -> Vec<&'a UsbDeviceRecord> {
        let mut results: Vec<&UsbDeviceRecord> = devices.iter()
            .filter(|d| self.matches(d))
            .collect();
        
        if let Some(limit) = self.limit {
            results.truncate(limit);
        }
        
        results
    }
    
    /// Filter and clone devices
    pub fn filter_owned(&self, devices: &[UsbDeviceRecord]) -> Vec<UsbDeviceRecord> {
        let mut results: Vec<UsbDeviceRecord> = devices.iter()
            .filter(|d| self.matches(d))
            .cloned()
            .collect();
        
        if let Some(limit) = self.limit {
            results.truncate(limit);
        }
        
        results
    }
    
    /// Find first matching device
    pub fn find_first<'a>(&self, devices: &'a [UsbDeviceRecord]) -> Option<&'a UsbDeviceRecord> {
        devices.iter().find(|d| self.matches(d))
    }
    
    /// Count matching devices
    pub fn count(&self, devices: &[UsbDeviceRecord]) -> usize {
        devices.iter().filter(|d| self.matches(d)).count()
    }
}

/// Common device queries
pub mod presets {
    use super::*;
    
    /// Query for Android devices (ADB/Fastboot)
    pub fn android_devices() -> DeviceQuery {
        DeviceQuery::new()
            .vendor_ids([
                0x18D1, // Google
                0x04E8, // Samsung
                0x22B8, // Motorola
                0x0BB4, // HTC
                0x12D1, // Huawei
                0x2717, // Xiaomi
                0x1949, // Amazon
                0x2A70, // OnePlus
                0x0FCE, // Sony Ericsson
                0x19D2, // ZTE
            ])
    }
    
    /// Query for Apple devices
    pub fn apple_devices() -> DeviceQuery {
        DeviceQuery::new().vendor_id(0x05AC)
    }
    
    /// Query for serial adapters (FTDI, CH340, CP210x, etc.)
    pub fn serial_adapters() -> DeviceQuery {
        DeviceQuery::new()
            .device_ids([
                (0x0403, 0x6001), // FTDI FT232
                (0x0403, 0x6010), // FTDI FT2232
                (0x0403, 0x6014), // FTDI FT232H
                (0x0403, 0x6015), // FTDI FT-X
                (0x10C4, 0xEA60), // CP210x
                (0x1A86, 0x7523), // CH340
                (0x1A86, 0x5523), // CH341
                (0x067B, 0x2303), // PL2303
            ])
    }
    
    /// Query for storage devices
    pub fn storage_devices() -> DeviceQuery {
        DeviceQuery::new().device_class(0x08) // Mass Storage
    }
    
    /// Query for HID devices
    pub fn hid_devices() -> DeviceQuery {
        DeviceQuery::new().device_class(0x03) // HID
    }
    
    /// Query for hubs
    pub fn hubs() -> DeviceQuery {
        DeviceQuery::new().device_class(0x09) // Hub
    }
    
    /// Query for audio devices
    pub fn audio_devices() -> DeviceQuery {
        DeviceQuery::new().device_class(0x01) // Audio
    }
    
    /// Query for video devices
    pub fn video_devices() -> DeviceQuery {
        DeviceQuery::new().device_class(0x0E) // Video
    }
    
    /// Query for printers
    pub fn printers() -> DeviceQuery {
        DeviceQuery::new().device_class(0x07) // Printer
    }
    
    /// Query for smart cards
    pub fn smart_cards() -> DeviceQuery {
        DeviceQuery::new().device_class(0x0B) // Smart Card
    }
    
    /// Query for wireless controllers (Bluetooth, etc.)
    pub fn wireless_controllers() -> DeviceQuery {
        DeviceQuery::new().device_class(0xE0) // Wireless
    }
    
    /// Query for development boards
    pub fn dev_boards() -> DeviceQuery {
        DeviceQuery::new()
            .vendor_ids([
                0x2341, // Arduino
                0x2E8A, // Raspberry Pi
                0x239A, // Adafruit
                0x303A, // Espressif
                0x1366, // SEGGER
                0x0483, // STMicroelectronics
                0x0D28, // ARM (DAPLink)
                0x1209, // Generic (pid.codes)
            ])
    }
    
    /// Query for security keys (FIDO, YubiKey, etc.)
    pub fn security_keys() -> DeviceQuery {
        DeviceQuery::new()
            .vendor_ids([
                0x1050, // Yubico
                0x096E, // Feitian
                0x20A0, // Nitrokey
                0x32A3, // SoloKeys
            ])
    }
    
    /// Query for game controllers
    pub fn game_controllers() -> DeviceQuery {
        DeviceQuery::new()
            .device_ids([
                (0x045E, 0x028E), // Xbox 360
                (0x045E, 0x02EA), // Xbox One S
                (0x045E, 0x0B12), // Xbox Series X
                (0x054C, 0x05C4), // DualShock 4 v1
                (0x054C, 0x09CC), // DualShock 4 v2
                (0x054C, 0x0CE6), // DualSense
                (0x057E, 0x2009), // Switch Pro Controller
            ])
    }
}

impl DeviceQuery {
    /// Add multiple device IDs (helper for presets)
    pub fn device_ids(mut self, ids: impl IntoIterator<Item = (u16, u16)>) -> Self {
        let set = self.device_ids.get_or_insert_with(HashSet::new);
        set.extend(ids);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    fn make_test_device(vid: u16, pid: u16, class: u8) -> UsbDeviceRecord {
        UsbDeviceRecord {
            id: UsbId::new(vid, pid),
            location: UsbLocation {
                bus: Some(1),
                address: Some(1),
                port_path: None,
            },
            descriptor: UsbDescriptorSummary {
                manufacturer: Some("Test".to_string()),
                product: Some("Device".to_string()),
                serial_number: Some("12345".to_string()),
                device_class: Some(class),
                device_subclass: None,
                device_protocol: None,
                usb_version: Some("2.0".to_string()),
            },
            driver: DriverStatus::Unknown,
            health: LinkHealth::Good,
            tags: vec![],
            raw_data: None,
        }
    }

    #[test]
    fn test_vendor_filter() {
        let devices = vec![
            make_test_device(0x18D1, 0x4EE0, 0xFF),
            make_test_device(0x05AC, 0x12A8, 0xEF),
        ];
        
        let query = DeviceQuery::new().vendor_id(0x18D1);
        let results = query.filter(&devices);
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id.vid, 0x18D1);
    }

    #[test]
    fn test_class_filter() {
        let devices = vec![
            make_test_device(0x1234, 0x5678, 0x08), // Mass storage
            make_test_device(0x1234, 0x5679, 0x03), // HID
        ];
        
        let query = DeviceQuery::new().device_class(0x08);
        let results = query.filter(&devices);
        
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_presets() {
        let query = presets::android_devices();
        assert!(query.vendor_ids.is_some());
        
        let query = presets::storage_devices();
        assert!(query.device_classes.is_some());
    }
}
