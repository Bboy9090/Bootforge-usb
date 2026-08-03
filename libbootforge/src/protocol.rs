//! Passive protocol classification from normalized USB observations.

use crate::types::{DeviceInfo, DeviceMode, DevicePlatform};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UsbProtocol {
    Adb,
    Fastboot,
    AppleMobile,
    AppleRecovery,
    Dfu,
    Mtp,
    Ptp,
    Cdc,
    MassStorage,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProtocolConfidence {
    Unknown,
    Low,
    Medium,
    High,
    Exact,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProtocolEvidence {
    DeviceMode,
    VendorProduct,
    Platform,
    ProductString,
    MatchedProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProtocolObservation {
    pub protocol: UsbProtocol,
    pub confidence: ProtocolConfidence,
    pub evidence: Vec<ProtocolEvidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProtocolReport {
    pub observations: Vec<ProtocolObservation>,
    pub active_probe_performed: bool,
}

impl ProtocolReport {
    pub fn from_device(device: &DeviceInfo) -> Self {
        let mut observations = Vec::new();
        let mut add = |protocol, confidence, evidence| {
            if !observations.iter().any(|item: &ProtocolObservation| item.protocol == protocol) {
                observations.push(ProtocolObservation { protocol, confidence, evidence });
            }
        };

        match device.mode {
            DeviceMode::Adb => add(UsbProtocol::Adb, ProtocolConfidence::High, vec![ProtocolEvidence::DeviceMode]),
            DeviceMode::Fastboot | DeviceMode::Bootloader => add(UsbProtocol::Fastboot, ProtocolConfidence::High, vec![ProtocolEvidence::DeviceMode]),
            DeviceMode::Dfu => add(UsbProtocol::Dfu, ProtocolConfidence::High, vec![ProtocolEvidence::DeviceMode]),
            DeviceMode::Recovery if device.platform == DevicePlatform::Apple => add(UsbProtocol::AppleRecovery, ProtocolConfidence::High, vec![ProtocolEvidence::DeviceMode, ProtocolEvidence::Platform]),
            DeviceMode::MassStorage => add(UsbProtocol::MassStorage, ProtocolConfidence::High, vec![ProtocolEvidence::DeviceMode]),
            _ => {}
        }

        if device.platform == DevicePlatform::Apple && matches!(device.mode, DeviceMode::Normal) {
            add(UsbProtocol::AppleMobile, ProtocolConfidence::Medium, vec![ProtocolEvidence::Platform, ProtocolEvidence::VendorProduct]);
        }

        let product = device.product_name.as_deref().unwrap_or_default().to_ascii_lowercase();
        let profile = device.matched_profile.as_deref().unwrap_or_default().to_ascii_lowercase();
        for (needle, protocol) in [
            ("mtp", UsbProtocol::Mtp),
            ("ptp", UsbProtocol::Ptp),
            ("cdc", UsbProtocol::Cdc),
            ("adb", UsbProtocol::Adb),
            ("fastboot", UsbProtocol::Fastboot),
            ("dfu", UsbProtocol::Dfu),
        ] {
            if product.contains(needle) || profile.contains(needle) {
                add(protocol, ProtocolConfidence::Medium, vec![ProtocolEvidence::ProductString, ProtocolEvidence::MatchedProfile]);
            }
        }

        if observations.is_empty() {
            observations.push(ProtocolObservation {
                protocol: UsbProtocol::Unknown,
                confidence: ProtocolConfidence::Unknown,
                evidence: Vec::new(),
            });
        }

        Self { observations, active_probe_performed: false }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{DeviceFamily, DeviceFingerprint, DeviceTransport, FingerprintConfidence, WorkflowRecommendation};

    fn fixture(mode: DeviceMode, platform: DevicePlatform) -> DeviceInfo {
        DeviceInfo {
            bus_number: 1, address: 2, vendor_id: 0x18d1, product_id: 0x4ee1,
            vendor_name: None, manufacturer: None, product_name: Some("Android ADB".into()),
            serial_number: Some("S1".into()), platform, transport: DeviceTransport::Usb2, mode,
            fingerprint: DeviceFingerprint { family: DeviceFamily::AndroidPhone, model_hint: None, confidence: FingerprintConfidence::High },
            recommended_workflow: WorkflowRecommendation::AndroidAdbWorkflow,
            matched_profile: Some("android-adb".into()),
        }
    }

    #[test]
    fn adb_is_classified_without_active_probe() {
        let report = ProtocolReport::from_device(&fixture(DeviceMode::Adb, DevicePlatform::Android));
        assert!(report.observations.iter().any(|item| item.protocol == UsbProtocol::Adb));
        assert!(!report.active_probe_performed);
    }
}
