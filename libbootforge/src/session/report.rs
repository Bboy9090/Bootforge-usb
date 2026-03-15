//! Scan report generation and export

use crate::error::{BootforgeError, Result};
use crate::session::history::current_timestamp_string;
use crate::types::DeviceInfo;
use serde::{Deserialize, Serialize};
use std::fs;

/// Scan report containing device snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    pub generated_at: String,
    pub total_devices: usize,
    pub devices: Vec<DeviceInfo>,
}

/// Create a scan report from a list of devices
pub fn create_scan_report(devices: Vec<DeviceInfo>) -> ScanReport {
    let total_devices = devices.len();
    ScanReport {
        generated_at: current_timestamp_string(),
        total_devices,
        devices,
    }
}

/// Write scan report to JSON file
pub fn write_scan_report_json(path: &str, report: &ScanReport) -> Result<()> {
    let json = serde_json::to_string_pretty(report)
        .map_err(|e| BootforgeError::JsonSerializationFailed(e.to_string()))?;
    fs::write(path, json).map_err(BootforgeError::IoError)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        DeviceFamily, DeviceFingerprint, DeviceMode, DevicePlatform, DeviceTransport,
        FingerprintConfidence, WorkflowRecommendation,
    };

    fn create_test_device() -> DeviceInfo {
        DeviceInfo {
            bus_number: 1,
            address: 2,
            vendor_id: 0x05ac,
            product_id: 0x1227,
            vendor_name: Some("Apple".to_string()),
            manufacturer: Some("Apple Inc.".to_string()),
            product_name: Some("iPhone".to_string()),
            serial_number: None,
            platform: DevicePlatform::Apple,
            transport: DeviceTransport::Usb2,
            mode: DeviceMode::Dfu,
            fingerprint: DeviceFingerprint {
                family: DeviceFamily::IPhone,
                model_hint: Some("iPhone".to_string()),
                confidence: FingerprintConfidence::High,
            },
            recommended_workflow: WorkflowRecommendation::AppleDfuWorkflow,
            matched_profile: Some("Apple DFU Device".to_string()),
        }
    }

    #[test]
    fn test_create_scan_report() {
        let devices = vec![create_test_device(), create_test_device()];
        let report = create_scan_report(devices);

        assert_eq!(report.total_devices, 2);
        assert_eq!(report.devices.len(), 2);
        assert!(!report.generated_at.is_empty());
    }

    #[test]
    fn test_create_empty_report() {
        let report = create_scan_report(vec![]);
        assert_eq!(report.total_devices, 0);
        assert_eq!(report.devices.len(), 0);
    }
}
