//! Passive composite-device analysis from decoded descriptor snapshots.

use crate::{DecodedDescriptor, DescriptorKind, DescriptorSnapshot};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompositeInterface {
    pub number: u8,
    pub alternate_setting: u8,
    pub class: u8,
    pub subclass: u8,
    pub protocol: u8,
    pub endpoint_count: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompositeReport {
    pub interfaces: Vec<CompositeInterface>,
    pub is_composite: bool,
    pub malformed_interface_descriptors: u32,
}

impl CompositeReport {
    pub fn from_snapshot(snapshot: &DescriptorSnapshot) -> Self {
        let mut interfaces = Vec::new();
        let mut malformed = 0_u32;
        for descriptor in &snapshot.descriptors {
            if descriptor.kind != DescriptorKind::Interface {
                continue;
            }
            match parse_interface(descriptor) {
                Some(interface) => interfaces.push(interface),
                None => malformed = malformed.saturating_add(1),
            }
        }
        Self {
            is_composite: interfaces.len() > 1,
            interfaces,
            malformed_interface_descriptors: malformed,
        }
    }
}

fn parse_interface(descriptor: &DecodedDescriptor) -> Option<CompositeInterface> {
    let bytes = &descriptor.bytes;
    if bytes.len() < 9 {
        return None;
    }
    Some(CompositeInterface {
        number: bytes[2],
        alternate_setting: bytes[3],
        endpoint_count: bytes[4],
        class: bytes[5],
        subclass: bytes[6],
        protocol: bytes[7],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_multiple_interfaces() {
        let raw = [9, 4, 0, 0, 1, 3, 1, 1, 0, 9, 4, 1, 0, 2, 255, 66, 1, 0];
        let report = CompositeReport::from_snapshot(&DescriptorSnapshot::decode(&raw));
        assert!(report.is_composite);
        assert_eq!(report.interfaces.len(), 2);
    }
}
