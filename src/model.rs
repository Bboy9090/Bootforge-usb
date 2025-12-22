use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct UsbId {
    pub vid: u16,
    pub pid: u16,
}

impl UsbId {
    pub fn new(vid: u16, pid: u16) -> Self {
        Self { vid, pid }
    }

    pub fn as_hex_string(&self) -> String {
        format!("{:04X}:{:04X}", self.vid, self.pid)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbLocation {
    pub bus: Option<u8>,
    pub address: Option<u8>,
    pub port_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbDescriptorSummary {
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub serial_number: Option<String>,
    pub device_class: Option<u8>,
    pub device_subclass: Option<u8>,
    pub device_protocol: Option<u8>,
    pub usb_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DriverStatus {
    Unknown,
    Bound { name: String },
    Missing,
    Blocked { reason: String },
    Multiple { drivers: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LinkHealth {
    Good,
    Unstable { reason: String },
    PowerIssueHint { reason: String },
    ResetLoop,
    Disconnected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbDeviceRecord {
    pub id: UsbId,
    pub location: UsbLocation,
    pub descriptor: UsbDescriptorSummary,
    pub driver: DriverStatus,
    pub health: LinkHealth,
    pub tags: Vec<String>,
    pub raw_data: Option<String>,
}

impl UsbDeviceRecord {
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t.eq_ignore_ascii_case(tag))
    }

    pub fn add_tag(&mut self, tag: impl Into<String>) {
        let tag = tag.into();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
}
