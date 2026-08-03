//! Normalized USB topology snapshots built from passive enumeration.

use crate::DeviceInfo;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TopologyNodeKind {
    Controller,
    RootHub,
    Hub,
    Device,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TopologyPath {
    pub bus_number: u8,
    pub device_address: u8,
    pub ports: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TopologyNode {
    pub id: String,
    pub parent_id: Option<String>,
    pub kind: TopologyNodeKind,
    pub path: TopologyPath,
    pub vendor_id: Option<u16>,
    pub product_id: Option<u16>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TopologySnapshot {
    pub nodes: Vec<TopologyNode>,
}

impl TopologySnapshot {
    pub fn from_devices(devices: &[DeviceInfo]) -> Self {
        let mut nodes = BTreeMap::new();
        for device in devices {
            let controller_id = format!("controller-bus-{}", device.bus_number);
            nodes.entry(controller_id.clone()).or_insert(TopologyNode {
                id: controller_id.clone(),
                parent_id: None,
                kind: TopologyNodeKind::Controller,
                path: TopologyPath { bus_number: device.bus_number, device_address: 0, ports: Vec::new() },
                vendor_id: None,
                product_id: None,
                label: Some(format!("USB controller bus {}", device.bus_number)),
            });
            let root_id = format!("root-hub-bus-{}", device.bus_number);
            nodes.entry(root_id.clone()).or_insert(TopologyNode {
                id: root_id.clone(),
                parent_id: Some(controller_id),
                kind: TopologyNodeKind::RootHub,
                path: TopologyPath { bus_number: device.bus_number, device_address: 0, ports: Vec::new() },
                vendor_id: None,
                product_id: None,
                label: Some(format!("Root hub bus {}", device.bus_number)),
            });
            let id = format!("usb-{}-{}-{:04x}-{:04x}", device.bus_number, device.address, device.vendor_id, device.product_id);
            nodes.insert(id.clone(), TopologyNode {
                id,
                parent_id: Some(root_id),
                kind: TopologyNodeKind::Device,
                path: TopologyPath { bus_number: device.bus_number, device_address: device.address, ports: Vec::new() },
                vendor_id: Some(device.vendor_id),
                product_id: Some(device.product_id),
                label: device.product_name.clone().or_else(|| device.manufacturer.clone()),
            });
        }
        Self { nodes: nodes.into_values().collect() }
    }

    pub fn children_of(&self, parent_id: &str) -> Vec<&TopologyNode> {
        self.nodes.iter().filter(|node| node.parent_id.as_deref() == Some(parent_id)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DeviceFamily, DeviceFingerprint, DeviceMode, DevicePlatform, DeviceTransport, FingerprintConfidence, WorkflowRecommendation};

    fn device(bus: u8, address: u8) -> DeviceInfo {
        DeviceInfo { bus_number: bus, address, vendor_id: 0x1234, product_id: 0x5678, vendor_name: None, manufacturer: None, product_name: Some("Test".into()), serial_number: None, platform: DevicePlatform::GenericUsb, transport: DeviceTransport::Usb2, mode: DeviceMode::Normal, fingerprint: DeviceFingerprint { family: DeviceFamily::Peripheral, model_hint: None, confidence: FingerprintConfidence::Low }, recommended_workflow: WorkflowRecommendation::GenericPeripheralInspection, matched_profile: None }
    }

    #[test]
    fn groups_devices_under_bus_root_hub() {
        let snapshot = TopologySnapshot::from_devices(&[device(1, 2), device(1, 3)]);
        assert_eq!(snapshot.children_of("root-hub-bus-1").len(), 2);
        assert_eq!(snapshot.nodes.iter().filter(|n| n.kind == TopologyNodeKind::Controller).count(), 1);
    }
}
