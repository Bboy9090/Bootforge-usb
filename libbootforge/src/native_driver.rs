//! Host-native, read-only driver inspection routing.

use crate::driver::{
    ArcwyreDriverInspector, DriverInspector, DriverReport, LinuxDriverInspector,
    MacOsDriverInspector, WindowsDriverInspector,
};
use crate::types::DeviceInfo;

/// Inspect the device with the current host's passive native backend.
///
/// Backend failures never mutate device state and degrade to the conservative libusb report.
pub fn inspect_platform_driver(device: &DeviceInfo) -> DriverReport {
    let result = inspect_native(device);
    result.unwrap_or_else(|_| DriverReport::passive_fallback(device))
}

#[cfg(windows)]
fn inspect_native(device: &DeviceInfo) -> crate::Result<DriverReport> {
    WindowsDriverInspector.inspect(device)
}

#[cfg(target_os = "linux")]
fn inspect_native(device: &DeviceInfo) -> crate::Result<DriverReport> {
    LinuxDriverInspector.inspect(device)
}

#[cfg(target_os = "macos")]
fn inspect_native(device: &DeviceInfo) -> crate::Result<DriverReport> {
    MacOsDriverInspector.inspect(device)
}

#[cfg(target_os = "arcwyre")]
fn inspect_native(device: &DeviceInfo) -> crate::Result<DriverReport> {
    ArcwyreDriverInspector.inspect(device)
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos", target_os = "arcwyre")))]
fn inspect_native(device: &DeviceInfo) -> crate::Result<DriverReport> {
    Ok(DriverReport::passive_fallback(device))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        DeviceFamily, DeviceFingerprint, DeviceMode, DevicePlatform, DeviceTransport,
        FingerprintConfidence, WorkflowRecommendation,
    };

    #[test]
    fn router_always_returns_a_report() {
        let device = DeviceInfo {
            bus_number: 255,
            address: 255,
            vendor_id: 0xffff,
            product_id: 0xffff,
            vendor_name: None,
            manufacturer: None,
            product_name: None,
            serial_number: None,
            platform: DevicePlatform::Unknown,
            transport: DeviceTransport::Unknown,
            mode: DeviceMode::Unknown,
            fingerprint: DeviceFingerprint {
                family: DeviceFamily::Unknown,
                model_hint: None,
                confidence: FingerprintConfidence::Unknown,
            },
            recommended_workflow: WorkflowRecommendation::Unknown,
            matched_profile: None,
        };

        let _report = inspect_platform_driver(&device);
    }
}
