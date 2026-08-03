//! Cross-platform, read-only driver visibility contract.
//!
//! The core library never installs, updates, replaces, or binds drivers. Platform backends
//! may populate this report from SetupAPI/CfgMgr32, sysfs/udev, IOKit, or ARCWYRE-native
//! facilities. Missing evidence remains explicit rather than being guessed.

use crate::types::{DeviceInfo, DevicePlatform};
use serde::{Deserialize, Serialize};

/// Operating-system backend that supplied driver evidence.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DriverBackend {
    WindowsSetupApi,
    WindowsCfgMgr32,
    LinuxSysfs,
    LinuxUdev,
    MacOsIoKit,
    ArcwyreNative,
    LibusbFallback,
    Unknown,
}

/// Normalized driver state visible to the library.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DriverState {
    Bound,
    Present,
    Missing,
    Disabled,
    Failed,
    PermissionDenied,
    Unknown,
}

/// Confidence assigned to the driver report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DriverConfidence {
    Unknown,
    Low,
    Medium,
    High,
    Exact,
}

/// Evidence that supports a driver-state conclusion.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DriverEvidence {
    BackendRecord,
    KernelBinding,
    DeviceNode,
    ServiceName,
    Provider,
    Version,
    SignatureState,
    ProblemCode,
    PermissionError,
    LibusbEnumeration,
    PlatformClassification,
}

/// Read-only, normalized driver intelligence for one device observation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DriverReport {
    pub backend: DriverBackend,
    pub state: DriverState,
    pub confidence: DriverConfidence,
    pub driver_name: Option<String>,
    pub service_name: Option<String>,
    pub provider: Option<String>,
    pub version: Option<String>,
    pub signed: Option<bool>,
    pub problem_code: Option<String>,
    pub device_node: Option<String>,
    pub evidence: Vec<DriverEvidence>,
    pub message: Option<String>,
}

impl DriverReport {
    /// Conservative fallback produced when only libusb enumeration is available.
    pub fn passive_fallback(device: &DeviceInfo) -> Self {
        let mut evidence = vec![
            DriverEvidence::LibusbEnumeration,
            DriverEvidence::PlatformClassification,
        ];

        let state = DriverState::Present;
        let confidence = match device.platform {
            DevicePlatform::Unknown => DriverConfidence::Low,
            _ => DriverConfidence::Medium,
        };

        let message = Some(
            "device is visible to passive enumeration; kernel driver binding and package metadata were not queried"
                .to_string(),
        );

        Self {
            backend: DriverBackend::LibusbFallback,
            state,
            confidence,
            driver_name: None,
            service_name: None,
            provider: None,
            version: None,
            signed: None,
            problem_code: None,
            device_node: None,
            evidence: std::mem::take(&mut evidence),
            message,
        }
    }

    pub fn unknown() -> Self {
        Self {
            backend: DriverBackend::Unknown,
            state: DriverState::Unknown,
            confidence: DriverConfidence::Unknown,
            driver_name: None,
            service_name: None,
            provider: None,
            version: None,
            signed: None,
            problem_code: None,
            device_node: None,
            evidence: Vec::new(),
            message: None,
        }
    }

    /// True only when the backend supplied evidence beyond generic libusb visibility.
    pub fn is_platform_enriched(&self) -> bool {
        !matches!(self.backend, DriverBackend::LibusbFallback | DriverBackend::Unknown)
    }
}

/// Adapter contract implemented by passive platform-specific driver backends.
pub trait DriverInspector {
    fn backend(&self) -> DriverBackend;
    fn inspect(&self, device: &DeviceInfo) -> crate::Result<DriverReport>;
}

/// Platform-neutral placeholder adapters. They establish the API boundary without claiming
/// native evidence before the corresponding backend is implemented and hardware-validated.
pub struct WindowsDriverInspector;
pub struct LinuxDriverInspector;
pub struct MacOsDriverInspector;
pub struct ArcwyreDriverInspector;

macro_rules! passive_stub {
    ($name:ty, $backend:expr, $message:expr) => {
        impl DriverInspector for $name {
            fn backend(&self) -> DriverBackend {
                $backend
            }

            fn inspect(&self, _device: &DeviceInfo) -> crate::Result<DriverReport> {
                Ok(DriverReport {
                    backend: self.backend(),
                    state: DriverState::Unknown,
                    confidence: DriverConfidence::Unknown,
                    driver_name: None,
                    service_name: None,
                    provider: None,
                    version: None,
                    signed: None,
                    problem_code: None,
                    device_node: None,
                    evidence: Vec::new(),
                    message: Some($message.to_string()),
                })
            }
        }
    };
}

passive_stub!(
    WindowsDriverInspector,
    DriverBackend::WindowsSetupApi,
    "Windows SetupAPI/CfgMgr32 enrichment adapter is defined but not yet implemented"
);
passive_stub!(
    LinuxDriverInspector,
    DriverBackend::LinuxSysfs,
    "Linux sysfs/udev enrichment adapter is defined but not yet implemented"
);
passive_stub!(
    MacOsDriverInspector,
    DriverBackend::MacOsIoKit,
    "macOS IOKit enrichment adapter is defined but not yet implemented"
);
passive_stub!(
    ArcwyreDriverInspector,
    DriverBackend::ArcwyreNative,
    "ARCWYRE native driver enrichment adapter is defined but not yet implemented"
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        DeviceFamily, DeviceFingerprint, DeviceMode, DeviceTransport, FingerprintConfidence,
        WorkflowRecommendation,
    };

    fn fixture(platform: DevicePlatform) -> DeviceInfo {
        DeviceInfo {
            bus_number: 1,
            address: 2,
            vendor_id: 0x18d1,
            product_id: 0x4ee1,
            vendor_name: Some("Google".into()),
            manufacturer: Some("Google".into()),
            product_name: Some("Android ADB".into()),
            serial_number: Some("SERIAL".into()),
            platform,
            transport: DeviceTransport::Usb3,
            mode: DeviceMode::Adb,
            fingerprint: DeviceFingerprint {
                family: DeviceFamily::AndroidPhone,
                model_hint: None,
                confidence: FingerprintConfidence::High,
            },
            recommended_workflow: WorkflowRecommendation::AndroidAdbWorkflow,
            matched_profile: Some("android-adb".into()),
        }
    }

    #[test]
    fn passive_fallback_never_claims_binding_metadata() {
        let report = DriverReport::passive_fallback(&fixture(DevicePlatform::Android));
        assert_eq!(report.backend, DriverBackend::LibusbFallback);
        assert_eq!(report.state, DriverState::Present);
        assert_eq!(report.driver_name, None);
        assert_eq!(report.signed, None);
        assert!(!report.is_platform_enriched());
    }

    #[test]
    fn unimplemented_native_adapter_reports_unknown() {
        let report = WindowsDriverInspector
            .inspect(&fixture(DevicePlatform::Android))
            .expect("stub must return a report");
        assert_eq!(report.backend, DriverBackend::WindowsSetupApi);
        assert_eq!(report.state, DriverState::Unknown);
        assert_eq!(report.confidence, DriverConfidence::Unknown);
    }
}
