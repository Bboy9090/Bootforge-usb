//! Cross-platform, read-only driver visibility contract.
//!
//! The core library never installs, updates, replaces, or binds drivers. Platform backends
//! may populate this report from SetupAPI/CfgMgr32, sysfs/udev, IOKit, or ARCWYRE-native
//! facilities. Missing evidence remains explicit rather than being guessed.

use crate::types::{DeviceInfo, DevicePlatform};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DriverConfidence {
    Unknown,
    Low,
    Medium,
    High,
    Exact,
}

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
    pub fn passive_fallback(device: &DeviceInfo) -> Self {
        Self {
            backend: DriverBackend::LibusbFallback,
            state: DriverState::Present,
            confidence: match device.platform {
                DevicePlatform::Unknown => DriverConfidence::Low,
                _ => DriverConfidence::Medium,
            },
            driver_name: None,
            service_name: None,
            provider: None,
            version: None,
            signed: None,
            problem_code: None,
            device_node: None,
            evidence: vec![
                DriverEvidence::LibusbEnumeration,
                DriverEvidence::PlatformClassification,
            ],
            message: Some(
                "device is visible to passive enumeration; native driver metadata was not queried"
                    .to_string(),
            ),
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

    pub fn is_platform_enriched(&self) -> bool {
        !matches!(self.backend, DriverBackend::LibusbFallback | DriverBackend::Unknown)
    }
}

pub trait DriverInspector {
    fn backend(&self) -> DriverBackend;
    fn inspect(&self, device: &DeviceInfo) -> crate::Result<DriverReport>;
}

pub struct WindowsDriverInspector;
pub struct LinuxDriverInspector;
pub struct MacOsDriverInspector;
pub struct ArcwyreDriverInspector;

impl DriverInspector for LinuxDriverInspector {
    fn backend(&self) -> DriverBackend {
        DriverBackend::LinuxSysfs
    }

    fn inspect(&self, device: &DeviceInfo) -> crate::Result<DriverReport> {
        let root = Path::new("/sys/bus/usb/devices");
        let entries = match fs::read_dir(root) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::PermissionDenied => {
                return Ok(DriverReport {
                    backend: self.backend(),
                    state: DriverState::PermissionDenied,
                    confidence: DriverConfidence::High,
                    driver_name: None,
                    service_name: None,
                    provider: None,
                    version: None,
                    signed: None,
                    problem_code: None,
                    device_node: None,
                    evidence: vec![DriverEvidence::PermissionError],
                    message: Some("permission denied while reading Linux USB sysfs".into()),
                });
            }
            Err(_) => return Ok(DriverReport::unknown()),
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if read_u8(path.join("busnum")) != Some(device.bus_number)
                || read_u8(path.join("devnum")) != Some(device.address)
            {
                continue;
            }

            let driver_link = path.join("driver");
            let driver_name = fs::read_link(&driver_link)
                .ok()
                .and_then(|target| target.file_name().map(|name| name.to_string_lossy().into_owned()));
            let bound = driver_name.is_some();
            let mut evidence = vec![DriverEvidence::BackendRecord, DriverEvidence::DeviceNode];
            if bound {
                evidence.push(DriverEvidence::KernelBinding);
            }

            return Ok(DriverReport {
                backend: self.backend(),
                state: if bound { DriverState::Bound } else { DriverState::Present },
                confidence: DriverConfidence::Exact,
                driver_name: driver_name.clone(),
                service_name: driver_name,
                provider: Some("Linux kernel".into()),
                version: None,
                signed: None,
                problem_code: None,
                device_node: Some(path.to_string_lossy().into_owned()),
                evidence,
                message: Some(if bound {
                    "USB device matched by bus/address and kernel driver binding found".into()
                } else {
                    "USB device matched by bus/address; no kernel driver symlink was present".into()
                }),
            });
        }

        Ok(DriverReport {
            backend: self.backend(),
            state: DriverState::Missing,
            confidence: DriverConfidence::Medium,
            driver_name: None,
            service_name: None,
            provider: None,
            version: None,
            signed: None,
            problem_code: None,
            device_node: None,
            evidence: vec![DriverEvidence::BackendRecord],
            message: Some("device was not found in Linux USB sysfs by bus/address".into()),
        })
    }
}

fn read_u8(path: PathBuf) -> Option<u8> {
    fs::read_to_string(path).ok()?.trim().parse().ok()
}

macro_rules! passive_stub {
    ($name:ty, $backend:expr, $message:expr) => {
        impl DriverInspector for $name {
            fn backend(&self) -> DriverBackend { $backend }
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

passive_stub!(WindowsDriverInspector, DriverBackend::WindowsSetupApi, "Windows SetupAPI/CfgMgr32 enrichment adapter is defined but not yet implemented");
passive_stub!(MacOsDriverInspector, DriverBackend::MacOsIoKit, "macOS IOKit enrichment adapter is defined but not yet implemented");
passive_stub!(ArcwyreDriverInspector, DriverBackend::ArcwyreNative, "ARCWYRE native driver enrichment adapter is defined but not yet implemented");

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
    }
}
