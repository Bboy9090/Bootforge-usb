//! Explainable, passive anomaly findings. Findings are evidence, not certainty.

use crate::{CompositeReport, DescriptorSnapshot, DeviceInfo};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AnomalySeverity {
    Info,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AnomalyKind {
    MalformedDescriptors,
    SuspiciousHidComposite,
    VendorSpecificOnly,
    MissingSerial,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnomalyFinding {
    pub kind: AnomalyKind,
    pub severity: AnomalySeverity,
    pub confidence_percent: u8,
    pub evidence: Vec<String>,
}

pub fn analyze_passively(
    device: &DeviceInfo,
    descriptors: &DescriptorSnapshot,
    composite: &CompositeReport,
) -> Vec<AnomalyFinding> {
    let mut findings = Vec::new();
    if !descriptors.issues.is_empty() {
        findings.push(AnomalyFinding {
            kind: AnomalyKind::MalformedDescriptors,
            severity: AnomalySeverity::Medium,
            confidence_percent: 100,
            evidence: vec![format!("{} descriptor issue(s)", descriptors.issues.len())],
        });
    }
    let has_hid = composite
        .interfaces
        .iter()
        .any(|interface| interface.class == 3);
    let has_vendor = composite
        .interfaces
        .iter()
        .any(|interface| interface.class == 255);
    if has_hid && has_vendor {
        findings.push(AnomalyFinding {
            kind: AnomalyKind::SuspiciousHidComposite,
            severity: AnomalySeverity::Low,
            confidence_percent: 55,
            evidence: vec!["HID and vendor-specific interfaces coexist".into()],
        });
    }
    if !composite.interfaces.is_empty()
        && composite
            .interfaces
            .iter()
            .all(|interface| interface.class == 255)
    {
        findings.push(AnomalyFinding {
            kind: AnomalyKind::VendorSpecificOnly,
            severity: AnomalySeverity::Info,
            confidence_percent: 70,
            evidence: vec!["all observed interfaces are vendor-specific".into()],
        });
    }
    if device
        .serial_number
        .as_deref()
        .map(str::trim)
        .unwrap_or("")
        .is_empty()
    {
        findings.push(AnomalyFinding {
            kind: AnomalyKind::MissingSerial,
            severity: AnomalySeverity::Info,
            confidence_percent: 100,
            evidence: vec!["device did not expose a usable serial number".into()],
        });
    }
    findings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        DeviceFamily, DeviceFingerprint, DeviceMode, DevicePlatform, DeviceTransport,
        FingerprintConfidence, WorkflowRecommendation,
    };

    fn device() -> DeviceInfo {
        DeviceInfo {
            bus_number: 1,
            address: 2,
            vendor_id: 1,
            product_id: 2,
            vendor_name: None,
            manufacturer: None,
            product_name: None,
            serial_number: None,
            platform: DevicePlatform::GenericUsb,
            transport: DeviceTransport::Usb2,
            mode: DeviceMode::Normal,
            fingerprint: DeviceFingerprint {
                family: DeviceFamily::Peripheral,
                model_hint: None,
                confidence: FingerprintConfidence::Low,
            },
            recommended_workflow: WorkflowRecommendation::GenericPeripheralInspection,
            matched_profile: None,
        }
    }

    #[test]
    fn missing_serial_is_reported_without_claiming_malice() {
        let findings = analyze_passively(
            &device(),
            &DescriptorSnapshot::decode(&[]),
            &CompositeReport {
                interfaces: Vec::new(),
                is_composite: false,
                malformed_interface_descriptors: 0,
            },
        );
        assert!(findings
            .iter()
            .any(|finding| finding.kind == AnomalyKind::MissingSerial));
    }
}
