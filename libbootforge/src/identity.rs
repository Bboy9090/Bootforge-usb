//! Stable, evidence-backed USB identity and reconnect correlation.
//!
//! Identity is intentionally probabilistic. Operating systems do not always expose a serial
//! number or stable topology path, so every result includes confidence and the evidence used.

use crate::types::DeviceInfo;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Confidence assigned to a device identity or reconnect match.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum IdentityConfidence {
    Unknown,
    Low,
    Medium,
    High,
    Exact,
}

/// Individual evidence used to construct or correlate an identity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IdentityEvidence {
    SerialNumber,
    VendorProduct,
    Manufacturer,
    ProductName,
    BusAddress,
    MatchedProfile,
    Platform,
    Mode,
}

/// Opaque, deterministic identity derived only from normalized observable fields.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceIdentity {
    pub stable_id: String,
    pub confidence: IdentityConfidence,
    pub evidence: Vec<IdentityEvidence>,
}

/// Result of comparing a newly observed device with a previous observation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReconnectMatch {
    pub is_match: bool,
    pub confidence: IdentityConfidence,
    pub score: u16,
    pub evidence: Vec<IdentityEvidence>,
}

impl DeviceIdentity {
    /// Build a deterministic identity from a normalized device snapshot.
    pub fn from_device(device: &DeviceInfo) -> Self {
        let mut components = vec![
            format!("vid={:04x}", device.vendor_id),
            format!("pid={:04x}", device.product_id),
            format!("platform={:?}", device.platform),
        ];
        let mut evidence = vec![IdentityEvidence::VendorProduct, IdentityEvidence::Platform];

        if let Some(serial) = normalized(&device.serial_number) {
            components.push(format!("serial={serial}"));
            evidence.push(IdentityEvidence::SerialNumber);
        } else {
            if let Some(manufacturer) = normalized(&device.manufacturer) {
                components.push(format!("manufacturer={manufacturer}"));
                evidence.push(IdentityEvidence::Manufacturer);
            }
            if let Some(product) = normalized(&device.product_name) {
                components.push(format!("product={product}"));
                evidence.push(IdentityEvidence::ProductName);
            }
            if let Some(profile) = normalized(&device.matched_profile) {
                components.push(format!("profile={profile}"));
                evidence.push(IdentityEvidence::MatchedProfile);
            }
        }

        components.sort();
        let digest = Sha256::digest(components.join("|").as_bytes());
        let stable_id = format!("bfusb-{:x}", digest);

        let confidence = if evidence.contains(&IdentityEvidence::SerialNumber) {
            IdentityConfidence::Exact
        } else if evidence.contains(&IdentityEvidence::MatchedProfile)
            && evidence.contains(&IdentityEvidence::ProductName)
        {
            IdentityConfidence::High
        } else if evidence.contains(&IdentityEvidence::ProductName)
            || evidence.contains(&IdentityEvidence::Manufacturer)
        {
            IdentityConfidence::Medium
        } else {
            IdentityConfidence::Low
        };

        Self { stable_id, confidence, evidence }
    }
}

/// Compare two observations and explain whether they likely represent the same physical device.
pub fn correlate_reconnect(previous: &DeviceInfo, current: &DeviceInfo) -> ReconnectMatch {
    let mut score = 0_u16;
    let mut evidence = Vec::new();

    if previous.vendor_id == current.vendor_id && previous.product_id == current.product_id {
        score += 30;
        evidence.push(IdentityEvidence::VendorProduct);
    } else {
        return ReconnectMatch {
            is_match: false,
            confidence: IdentityConfidence::Unknown,
            score: 0,
            evidence,
        };
    }

    if equal_normalized(&previous.serial_number, &current.serial_number) {
        score += 60;
        evidence.push(IdentityEvidence::SerialNumber);
    }
    if equal_normalized(&previous.manufacturer, &current.manufacturer) {
        score += 10;
        evidence.push(IdentityEvidence::Manufacturer);
    }
    if equal_normalized(&previous.product_name, &current.product_name) {
        score += 15;
        evidence.push(IdentityEvidence::ProductName);
    }
    if equal_normalized(&previous.matched_profile, &current.matched_profile) {
        score += 15;
        evidence.push(IdentityEvidence::MatchedProfile);
    }
    if previous.platform == current.platform {
        score += 5;
        evidence.push(IdentityEvidence::Platform);
    }
    if previous.bus_number == current.bus_number && previous.address == current.address {
        score += 5;
        evidence.push(IdentityEvidence::BusAddress);
    }
    if previous.mode == current.mode {
        score += 5;
        evidence.push(IdentityEvidence::Mode);
    }

    let confidence = match score {
        90.. => IdentityConfidence::Exact,
        70..=89 => IdentityConfidence::High,
        50..=69 => IdentityConfidence::Medium,
        35..=49 => IdentityConfidence::Low,
        _ => IdentityConfidence::Unknown,
    };

    ReconnectMatch {
        is_match: score >= 50,
        confidence,
        score,
        evidence,
    }
}

fn normalized(value: &Option<String>) -> Option<String> {
    value
        .as_ref()
        .map(|item| item.trim().to_lowercase())
        .filter(|item| !item.is_empty())
}

fn equal_normalized(left: &Option<String>, right: &Option<String>) -> bool {
    match (normalized(left), normalized(right)) {
        (Some(left), Some(right)) => left == right,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        DeviceFamily, DeviceFingerprint, DeviceMode, DevicePlatform, DeviceTransport,
        FingerprintConfidence, WorkflowRecommendation,
    };

    fn device(serial: Option<&str>, address: u8) -> DeviceInfo {
        DeviceInfo {
            bus_number: 1,
            address,
            vendor_id: 0x05ac,
            product_id: 0x1227,
            vendor_name: Some("Apple".into()),
            manufacturer: Some("Apple Inc.".into()),
            product_name: Some("Apple Mobile Device (DFU Mode)".into()),
            serial_number: serial.map(str::to_string),
            platform: DevicePlatform::Apple,
            transport: DeviceTransport::Usb2,
            mode: DeviceMode::Dfu,
            fingerprint: DeviceFingerprint {
                family: DeviceFamily::IPhone,
                model_hint: None,
                confidence: FingerprintConfidence::High,
            },
            recommended_workflow: WorkflowRecommendation::AppleDfuWorkflow,
            matched_profile: Some("apple-dfu".into()),
        }
    }

    #[test]
    fn stable_identity_survives_address_change_when_serial_exists() {
        let first = DeviceIdentity::from_device(&device(Some("ABC123"), 2));
        let second = DeviceIdentity::from_device(&device(Some("abc123"), 9));
        assert_eq!(first.stable_id, second.stable_id);
        assert_eq!(first.confidence, IdentityConfidence::Exact);
    }

    #[test]
    fn serial_match_correlates_reconnect_across_address_change() {
        let result = correlate_reconnect(&device(Some("ABC123"), 2), &device(Some("ABC123"), 9));
        assert!(result.is_match);
        assert_eq!(result.confidence, IdentityConfidence::Exact);
        assert!(result.score >= 90);
    }

    #[test]
    fn different_vid_pid_is_never_correlated() {
        let first = device(None, 2);
        let mut second = device(None, 2);
        second.product_id = 0x9999;
        let result = correlate_reconnect(&first, &second);
        assert!(!result.is_match);
        assert_eq!(result.score, 0);
    }
}
