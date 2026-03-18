// ForgeWorks Core - Device Analysis Service
// COMPLIANCE-FIRST: Analysis only, no execution

use serde::{Deserialize, Serialize};
use chrono::Utc;
use libbootforge::{device::{DeviceInfo, DeviceMode}, enumerate_devices};

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
    Clean,                    // No modifications detected
    SoftwareModified,         // Software-level modifications
    HardwareModified,         // Hardware-level modifications
    ServiceModified,          // Service-level modifications (IMEI, etc.)
    Unknown,                  // Cannot determine
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
pub fn analyze(metadata: &str) -> DeviceProfile {
    // Mock device analysis - in production, this would use USB enumeration
    // For now, we classify based on metadata string

    let (classification, _risk_level) = classify_from_metadata(metadata);
    let capability_class = determine_capability_class(&classification);

    DeviceProfile {
        device_id: uuid::Uuid::new_v4().to_string(),
        model: extract_model(metadata),
        manufacturer: extract_manufacturer(metadata),
        security_state: format!("Analyzed - {:?}", classification),
        capability_class,
        classification: classification.clone(),
        restrictions: vec![
            "No modification capability".to_string(),
            "Analysis only".to_string(),
            "Read-only operations".to_string(),
        ],
        analysis_timestamp: Utc::now(),
        non_invasive: true, // Always true
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
    let classification = classify_from_usb_device(device);
    let capability_class = determine_capability_class(&classification);

    DeviceProfile {
        device_id: format!("{:04x}:{:04x}:{:03}:{:03}",
            device.vendor_id,
            device.product_id,
            device.bus_number,
            device.device_address
        ),
        model: device.product_name.clone().unwrap_or_else(|| "Unknown Device".to_string()),
        manufacturer: device.manufacturer.clone().unwrap_or_else(|| "Unknown".to_string()),
        security_state: format!("Mode: {:?}", device.device_mode),
        capability_class,
        classification,
        restrictions: vec![
            "No modification capability".to_string(),
            "Analysis only".to_string(),
            "Read-only operations".to_string(),
        ],
        analysis_timestamp: Utc::now(),
        non_invasive: true,
    }
}

fn classify_from_usb_device(device: &DeviceInfo) -> DeviceClassification {
    // Detect special modes (DFU, Recovery) as potential modifications
    match device.device_mode {
        DeviceMode::Dfu | DeviceMode::Recovery => DeviceClassification::HardwareModified,
        DeviceMode::Diagnostic => DeviceClassification::SoftwareModified,
        DeviceMode::Normal | DeviceMode::Unknown => DeviceClassification::Clean,
    }
}

fn extract_model(metadata: &str) -> String {
    // Mock extraction - in production, parse from USB descriptors
    if metadata.contains("iPhone") {
        "iPhone 13 Pro".to_string()
    } else if metadata.contains("Samsung") {
        "Galaxy S21".to_string()
    } else {
        "Unknown Device".to_string()
    }
}

fn extract_manufacturer(metadata: &str) -> String {
    // Mock extraction
    if metadata.contains("iPhone") {
        "Apple".to_string()
    } else if metadata.contains("Samsung") {
        "Samsung".to_string()
    } else {
        "Unknown".to_string()
    }
}

fn classify_from_metadata(metadata: &str) -> (DeviceClassification, RiskLevel) {
    // Mock classification - in production, use actual device state analysis
    let lower = metadata.to_lowercase();
    
    if lower.contains("hardware") || lower.contains("checkm8") {
        (DeviceClassification::HardwareModified, RiskLevel::High)
    } else if lower.contains("service") || lower.contains("imei") {
        (DeviceClassification::ServiceModified, RiskLevel::VeryHigh)
    } else if lower.contains("software") || lower.contains("modified") {
        (DeviceClassification::SoftwareModified, RiskLevel::Medium)
    } else if lower.contains("clean") || lower.contains("stock") {
        (DeviceClassification::Clean, RiskLevel::Low)
    } else {
        (DeviceClassification::Unknown, RiskLevel::Low)
    }
}

fn determine_capability_class(classification: &DeviceClassification) -> String {
    match classification {
        DeviceClassification::HardwareModified => "Hardware-level modification detected".to_string(),
        DeviceClassification::ServiceModified => "Service-level modification detected".to_string(),
        DeviceClassification::SoftwareModified => "Software-level modification detected".to_string(),
        DeviceClassification::Clean => "No modifications detected".to_string(),
        DeviceClassification::Unknown => "Unable to determine modification status".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_device() {
        let result = analyze("iPhone 13 Pro - Clean device");
        assert_eq!(result.non_invasive, true);
        assert_eq!(result.classification, DeviceClassification::Clean);
    }

    #[test]
    fn test_classify_hardware_modified() {
        let (classification, risk) = classify_from_metadata("Device with hardware modification");
        assert_eq!(classification, DeviceClassification::HardwareModified);
        assert_eq!(risk, RiskLevel::High);
    }

    #[test]
    fn test_classify_service_modified() {
        let (classification, risk) = classify_from_metadata("Device with IMEI modification");
        assert_eq!(classification, DeviceClassification::ServiceModified);
        assert_eq!(risk, RiskLevel::VeryHigh);
    }

    #[test]
    fn test_always_non_invasive() {
        let result = analyze("Any device metadata");
        assert_eq!(result.non_invasive, true);
    }

    #[test]
    #[ignore] // Requires USB subsystem - not available in CI
    fn test_enumerate_usb_devices() {
        // This test should succeed even if no devices are connected
        let result = enumerate_usb_devices();
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_usb_device() {
        use libbootforge::device::{DeviceInfo, DeviceMode};

        let device = DeviceInfo {
            vendor_id: 0x05ac,
            product_id: 0x12a8,
            manufacturer: Some("Apple Inc.".to_string()),
            product_name: Some("iPhone".to_string()),
            serial_number: Some("ABC123".to_string()),
            class: 0,
            subclass: 0,
            protocol: 0,
            usb_version: 0x0200,
            device_version: 0x0100,
            bus_number: 1,
            device_address: 2,
            platform_path: None,
            device_mode: DeviceMode::Normal,
        };

        let profile = analyze_usb_device(&device);
        assert_eq!(profile.non_invasive, true);
        assert_eq!(profile.manufacturer, "Apple Inc.");
        assert_eq!(profile.classification, DeviceClassification::Clean);
    }

    #[test]
    fn test_classify_dfu_device() {
        use libbootforge::device::{DeviceInfo, DeviceMode};

        let device = DeviceInfo {
            vendor_id: 0x05ac,
            product_id: 0x1227,
            manufacturer: Some("Apple Inc.".to_string()),
            product_name: Some("iPhone DFU".to_string()),
            serial_number: None,
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
        assert_eq!(profile.classification, DeviceClassification::HardwareModified);
    }
}
