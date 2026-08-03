//! Deterministic versioned forensic fingerprints for passive USB observations.

use crate::{DeviceInfo, DeviceMode, DevicePlatform, DeviceTransport};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const FORENSIC_FINGERPRINT_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ForensicFingerprintConfidence {
    Weak,
    Moderate,
    Strong,
    Exact,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ForensicFingerprintEvidence {
    VendorProduct,
    SerialNumber,
    Manufacturer,
    ProductName,
    Platform,
    Transport,
    Mode,
    MatchedProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ForensicFingerprint {
    pub schema_version: u16,
    pub value: String,
    pub confidence: ForensicFingerprintConfidence,
    pub evidence: Vec<ForensicFingerprintEvidence>,
}

impl ForensicFingerprint {
    pub fn from_device(device: &DeviceInfo) -> Self {
        let mut evidence = vec![
            ForensicFingerprintEvidence::VendorProduct,
            ForensicFingerprintEvidence::Platform,
            ForensicFingerprintEvidence::Transport,
            ForensicFingerprintEvidence::Mode,
        ];
        let serial = normalized(device.serial_number.as_deref());
        let manufacturer = normalized(device.manufacturer.as_deref());
        let product = normalized(device.product_name.as_deref());
        let profile = normalized(device.matched_profile.as_deref());
        if serial.is_some() {
            evidence.push(ForensicFingerprintEvidence::SerialNumber);
        }
        if manufacturer.is_some() {
            evidence.push(ForensicFingerprintEvidence::Manufacturer);
        }
        if product.is_some() {
            evidence.push(ForensicFingerprintEvidence::ProductName);
        }
        if profile.is_some() {
            evidence.push(ForensicFingerprintEvidence::MatchedProfile);
        }
        let canonical = format!(
            "v{}|{:04x}|{:04x}|{}|{}|{}|{}|{}|{}|{}",
            FORENSIC_FINGERPRINT_SCHEMA_VERSION,
            device.vendor_id,
            device.product_id,
            serial.as_deref().unwrap_or(""),
            manufacturer.as_deref().unwrap_or(""),
            product.as_deref().unwrap_or(""),
            platform(device.platform),
            transport(device.transport),
            mode(device.mode),
            profile.as_deref().unwrap_or("")
        );
        let value = format!("bffp1-{:x}", Sha256::digest(canonical.as_bytes()));
        let confidence = if serial.is_some() && manufacturer.is_some() && product.is_some() {
            ForensicFingerprintConfidence::Exact
        } else if serial.is_some()
            || (manufacturer.is_some() && product.is_some() && profile.is_some())
        {
            ForensicFingerprintConfidence::Strong
        } else if manufacturer.is_some() || product.is_some() || profile.is_some() {
            ForensicFingerprintConfidence::Moderate
        } else {
            ForensicFingerprintConfidence::Weak
        };
        Self {
            schema_version: FORENSIC_FINGERPRINT_SCHEMA_VERSION,
            value,
            confidence,
            evidence,
        }
    }
}

fn normalized(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_ascii_lowercase())
}
fn platform(value: DevicePlatform) -> &'static str {
    match value {
        DevicePlatform::Apple => "apple",
        DevicePlatform::Android => "android",
        DevicePlatform::GenericUsb => "generic",
        DevicePlatform::Unknown => "unknown",
    }
}
fn transport(value: DeviceTransport) -> &'static str {
    match value {
        DeviceTransport::Usb2 => "usb2",
        DeviceTransport::Usb3 => "usb3",
        DeviceTransport::Unknown => "unknown",
    }
}
fn mode(value: DeviceMode) -> &'static str {
    match value {
        DeviceMode::Normal => "normal",
        DeviceMode::Recovery => "recovery",
        DeviceMode::Dfu => "dfu",
        DeviceMode::Bootloader => "bootloader",
        DeviceMode::Fastboot => "fastboot",
        DeviceMode::Adb => "adb",
        DeviceMode::MassStorage => "mass-storage",
        DeviceMode::Unknown => "unknown",
    }
}
