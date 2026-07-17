//! Core types for device information and classification

use serde::{Deserialize, Serialize};

/// Complete device information including fingerprint and workflow recommendation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceInfo {
    pub bus_number: u8,
    pub address: u8,
    pub vendor_id: u16,
    pub product_id: u16,
    pub vendor_name: Option<String>,
    pub manufacturer: Option<String>,
    pub product_name: Option<String>,
    pub serial_number: Option<String>,
    pub platform: DevicePlatform,
    pub transport: DeviceTransport,
    pub mode: DeviceMode,
    pub fingerprint: DeviceFingerprint,
    pub recommended_workflow: WorkflowRecommendation,
    pub matched_profile: Option<String>,
}

/// Device operating mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeviceMode {
    Normal,
    Recovery,
    Dfu,
    Bootloader,
    Fastboot,
    Adb,
    MassStorage,
    Unknown,
}

impl DeviceMode {
    /// Parse mode from string (for CLI)
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "normal" => Some(DeviceMode::Normal),
            "recovery" => Some(DeviceMode::Recovery),
            "dfu" => Some(DeviceMode::Dfu),
            "bootloader" => Some(DeviceMode::Bootloader),
            "fastboot" => Some(DeviceMode::Fastboot),
            "adb" => Some(DeviceMode::Adb),
            "massstorage" => Some(DeviceMode::MassStorage),
            "unknown" => Some(DeviceMode::Unknown),
            _ => None,
        }
    }
}

/// Device platform/ecosystem
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DevicePlatform {
    Apple,
    Android,
    GenericUsb,
    Unknown,
}

/// USB transport type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeviceTransport {
    Usb2,
    Usb3,
    Unknown,
}

/// Device fingerprint with family and confidence
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceFingerprint {
    pub family: DeviceFamily,
    pub model_hint: Option<String>,
    pub confidence: FingerprintConfidence,
}

/// Device family classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeviceFamily {
    IPhone,
    IPad,
    AndroidPhone,
    AndroidTablet,
    UsbStorage,
    Peripheral,
    Unknown,
}

/// Fingerprint confidence level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FingerprintConfidence {
    High,
    Medium,
    Low,
    Unknown,
}

/// Recommended workflow for device
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkflowRecommendation {
    AppleNormalInspection,
    AppleRecoveryWorkflow,
    AppleDfuWorkflow,
    AndroidAdbWorkflow,
    AndroidFastbootWorkflow,
    MassStorageInspection,
    GenericPeripheralInspection,
    Unknown,
}
