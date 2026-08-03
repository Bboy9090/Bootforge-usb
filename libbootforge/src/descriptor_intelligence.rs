//! Bounded, read-only USB descriptor decoding with explicit malformed-input reporting.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const MAX_DESCRIPTOR_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DescriptorKind {
    Device,
    Configuration,
    String,
    Interface,
    Endpoint,
    DeviceQualifier,
    InterfaceAssociation,
    Bos,
    Hid,
    DfuFunctional,
    Unknown(u8),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DescriptorIssue {
    InputTooLarge { length: usize, maximum: usize },
    TruncatedHeader { offset: usize },
    InvalidLength { offset: usize, length: u8 },
    TruncatedBody { offset: usize, declared: usize, available: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DecodedDescriptor {
    pub offset: usize,
    pub length: u8,
    pub kind: DescriptorKind,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DescriptorSnapshot {
    pub sha256: String,
    pub descriptors: Vec<DecodedDescriptor>,
    pub issues: Vec<DescriptorIssue>,
    pub complete: bool,
}

impl DescriptorSnapshot {
    pub fn decode(input: &[u8]) -> Self {
        if input.len() > MAX_DESCRIPTOR_BYTES {
            return Self {
                sha256: digest(input),
                descriptors: Vec::new(),
                issues: vec![DescriptorIssue::InputTooLarge {
                    length: input.len(),
                    maximum: MAX_DESCRIPTOR_BYTES,
                }],
                complete: false,
            };
        }

        let mut descriptors = Vec::new();
        let mut issues = Vec::new();
        let mut offset = 0;
        while offset < input.len() {
            if input.len() - offset < 2 {
                issues.push(DescriptorIssue::TruncatedHeader { offset });
                break;
            }
            let length = input[offset];
            if length < 2 {
                issues.push(DescriptorIssue::InvalidLength { offset, length });
                break;
            }
            let end = offset.saturating_add(length as usize);
            if end > input.len() {
                issues.push(DescriptorIssue::TruncatedBody {
                    offset,
                    declared: length as usize,
                    available: input.len() - offset,
                });
                break;
            }
            descriptors.push(DecodedDescriptor {
                offset,
                length,
                kind: kind(input[offset + 1]),
                bytes: input[offset..end].to_vec(),
            });
            offset = end;
        }
        Self {
            sha256: digest(input),
            complete: issues.is_empty() && offset == input.len(),
            descriptors,
            issues,
        }
    }
}

fn kind(value: u8) -> DescriptorKind {
    match value {
        1 => DescriptorKind::Device,
        2 => DescriptorKind::Configuration,
        3 => DescriptorKind::String,
        4 => DescriptorKind::Interface,
        5 => DescriptorKind::Endpoint,
        6 => DescriptorKind::DeviceQualifier,
        11 => DescriptorKind::InterfaceAssociation,
        15 => DescriptorKind::Bos,
        0x21 => DescriptorKind::Hid,
        0x21 | 0x22 => DescriptorKind::DfuFunctional,
        other => DescriptorKind::Unknown(other),
    }
}

fn digest(input: &[u8]) -> String {
    format!("{:x}", Sha256::digest(input))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_multiple_descriptors_deterministically() {
        let input = [2, 1, 3, 4, 0xaa];
        let snapshot = DescriptorSnapshot::decode(&input);
        assert!(snapshot.complete);
        assert_eq!(snapshot.descriptors.len(), 2);
        assert_eq!(snapshot, DescriptorSnapshot::decode(&input));
    }

    #[test]
    fn rejects_truncated_body_without_panicking() {
        let snapshot = DescriptorSnapshot::decode(&[5, 1, 0]);
        assert!(!snapshot.complete);
        assert!(matches!(snapshot.issues[0], DescriptorIssue::TruncatedBody { .. }));
    }
}
