// ForgeWorks Core - Device Analysis Service
// COMPLIANCE-FIRST: Analysis only, no execution

use chrono::Utc;
use libbootforge::{
    device::{DeviceInfo, DeviceMode},
    enumerate_devices,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceProfile {
    pub device_id: String,
    pub model: String,
    pub manufacturer: String,
    pub security_state: String,
    pub capability_class: String,
    pub classification: DeviceClassification,
    pub restrictions: Vec<String>,
    pub analysis_timestamp: chrono::DateTime<chrono::Utc>,
    pub non_invasive: bool, // Always true - we never modify
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceClassification {
    Clean,            // No modifications detected
    SoftwareModified, // Software-level modifications
    HardwareModified, // Hardware-level modifications
    ServiceModified,  // Service-level modifications (IMEI, etc.)
    Unknown,          // Cannot determine
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    VeryHigh,
}

/**
 * Analyze device (non-invasive, read-only)
 *
 * This function performs diagnostic analysis only.
 * It does NOT execute any modifications, exploits, or bypasses.
 */
pub fn analyze(_metadata: &str) -> DeviceProfile {
    DeviceProfile {
        device_id: format!("metadata-only-{}", uuid::Uuid::new_v4()),
        model: "Unverified".to_string(),
        manufacturer: "Unverified".to_string(),
        security_state: "Metadata-only input; hardware security state is unverified".to_string(),
        capability_class: "Evidence required before classification".to_string(),
        classification: DeviceClassification::Unknown,
        restrictions: vec![
            "Metadata text is not hardware evidence".to_string(),
            "Real USB/device observations required".to_string(),
            "No modification-state inference permitted".to_string(),
            "Analysis only".to_string(),
        ],
        analysis_timestamp: Utc::now(),
        non_invasive: true,
    }
}

/**
 * Enumerate all connected USB devices using libbootforge
 *
 * This function uses libbootforge to detect USB devices.
 * It is read-only and does not modify any devices.
 */
pub fn enumerate_usb_devices() -> Result<Vec<DeviceInfo>, String> {
    enumerate_devices().map_err(|e| format!("Failed to enumerate devices: {}", e))
}

/**
 * Analyze a USB device from libbootforge DeviceInfo
 *
 * Converts low-level USB device information into a high-level DeviceProfile
 */
pub fn analyze_usb_device(device: &DeviceInfo) -> DeviceProfile {
    let classification = DeviceClassification::Unknown;
    let capability_class = "Observed hardware identity and mode; modification state unverified".to_string();

    DeviceProfile {
        device_id: format!(
            "{:04x}:{:04x}:{:03}:{:03}",
            device.vendor_id, device.product_id, device.bus_number, device.device_address
        ),
        model: device
            .product_name
            .clone()
            .unwrap_or_else(|| "Unknown Device".to_string()),
        manufacturer: device
            .manufacturer
            .clone()
            .unwrap_or_else(|| "Unknown".to_string()),
        security_state: format!("Observed USB mode: {:?}; modification state unverified", device.device_mode),
        capability_class,
        classification,
        restrictions: vec![
            "No modification capability".to_string(),
            "Analysis only".to_string(),
            "Read-only operations".to_string(),
            "Modification state requires independent evidence".to_string(),
        ],
        analysis_timestamp: Utc::now(),
        non_invasive: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_only_analysis_never_claims_device_state() {
        for metadata in [
            "iPhone 13 Pro - Clean device",
            "Device with hardware modification",
            "Device with IMEI modification",
            "Samsung stock",
        ] {
            let result = analyze(metadata);
            assert!(result.non_invasive);
            assert_eq!(result.classification, DeviceClassification::Unknown);
            assert_eq!(result.model, "Unverified");
            assert!(result.security_state.contains("unverified"));
        }
    }

    #[test]
    fn usb_observation_preserves_real_identity_but_not_modification_inference() {
        use libbootforge::device::{DeviceInfo, DeviceMode};

        let device = DeviceInfo {
            vendor_id: 0x05ac,
            product_id: 0x1227,
            manufacturer: Some("Apple Inc.".to_string()),
            product_name: Some("iPhone DFU".to_string()),
            serial_number: Some("ABC123".to_string()),
            class: 0xfe,
            subclass: 0,
            protocol: 0,
            usb_version: 0x0200,
            device_version: 0x0100,
            bus_number: 1,
            device_address: 3,
            platform_path: None,
            device_mode: DeviceMode::Dfu,
        };

        let profile = analyze_usb_device(&device);
        assert_eq!(profile.manufacturer, "Apple Inc.");
        assert_eq!(profile.model, "iPhone DFU");
        assert_eq!(profile.classification, DeviceClassification::Unknown);
        assert!(profile.security_state.contains("DFU"));
        assert!(profile.security_state.contains("unverified"));
    }

    #[test]
    #[ignore]
    fn enumerate_usb_devices_uses_real_usb_subsystem() {
        let result = enumerate_usb_devices();
        assert!(result.is_ok());
    }
}
