//! Safety-gated orchestration for destructive media writes.
//!
//! This crate is intentionally separate from libbootforge. The read-only discovery
//! library must never gain raw-write side effects.

use libbootforge::{HostPlatform, MediaTarget};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerifiedWriteTarget {
    pub device_id: String,
    pub display_name: String,
    pub capacity_bytes: u64,
    pub removable: bool,
    pub system_disk: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WriteRequest {
    pub source_path: String,
    pub host: HostPlatform,
    pub target: MediaTarget,
    pub destination: VerifiedWriteTarget,
    pub confirmation_phrase: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WritePlan {
    pub source_path: String,
    pub destination_id: String,
    pub host: HostPlatform,
    pub target: MediaTarget,
    pub backend: &'static str,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum WriteSafetyError {
    #[error("destination is not verified removable media")]
    NotRemovable,
    #[error("destination is marked as a system disk")]
    SystemDisk,
    #[error("confirmation phrase does not match destination identity")]
    ConfirmationMismatch,
    #[error("no write backend is defined for this host/target pair")]
    UnsupportedRoute,
}

pub fn required_confirmation(target: &VerifiedWriteTarget) -> String {
    format!("ERASE {}", target.device_id)
}

pub fn validate_write_request(request: &WriteRequest) -> Result<WritePlan, WriteSafetyError> {
    if !request.destination.removable {
        return Err(WriteSafetyError::NotRemovable);
    }
    if request.destination.system_disk {
        return Err(WriteSafetyError::SystemDisk);
    }
    if request.confirmation_phrase != required_confirmation(&request.destination) {
        return Err(WriteSafetyError::ConfirmationMismatch);
    }

    let backend = backend_for(request.host, request.target)
        .ok_or(WriteSafetyError::UnsupportedRoute)?;

    Ok(WritePlan {
        source_path: request.source_path.clone(),
        destination_id: request.destination.device_id.clone(),
        host: request.host,
        target: request.target,
        backend,
    })
}

fn backend_for(host: HostPlatform, target: MediaTarget) -> Option<&'static str> {
    use HostPlatform::*;
    use MediaTarget::*;

    match (host, target) {
        (Windows, WindowsInstaller) => Some("windows-media"),
        (Windows, LinuxLive | MacOsRawImage) => Some("windows-raw-image"),

        (MacOs, WindowsInstaller) => Some("macos-windows-media"),
        (MacOs, LinuxLive | MacOsRawImage) => Some("macos-raw-image"),
        (MacOs, MacOsOfficialInstaller) => Some("macos-createinstallmedia"),

        (Linux, WindowsInstaller) => Some("linux-windows-media"),
        (Linux, LinuxLive | MacOsRawImage) => Some("linux-raw-image"),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target() -> VerifiedWriteTarget {
        VerifiedWriteTarget {
            device_id: "disk-test-001".into(),
            display_name: "Test USB".into(),
            capacity_bytes: 16_000_000_000,
            removable: true,
            system_disk: false,
        }
    }

    #[test]
    fn rejects_non_removable_targets() {
        let mut destination = target();
        destination.removable = false;

        let request = WriteRequest {
            source_path: "image.iso".into(),
            host: HostPlatform::Linux,
            target: MediaTarget::LinuxLive,
            confirmation_phrase: required_confirmation(&destination),
            destination,
        };

        assert_eq!(
            validate_write_request(&request),
            Err(WriteSafetyError::NotRemovable)
        );
    }

    #[test]
    fn rejects_system_disk_even_if_marked_removable() {
        let mut destination = target();
        destination.system_disk = true;

        let request = WriteRequest {
            source_path: "image.iso".into(),
            host: HostPlatform::MacOs,
            target: MediaTarget::LinuxLive,
            confirmation_phrase: required_confirmation(&destination),
            destination,
        };

        assert_eq!(
            validate_write_request(&request),
            Err(WriteSafetyError::SystemDisk)
        );
    }

    #[test]
    fn requires_target_specific_confirmation() {
        let destination = target();
        let request = WriteRequest {
            source_path: "image.iso".into(),
            host: HostPlatform::Windows,
            target: MediaTarget::WindowsInstaller,
            confirmation_phrase: "ERASE something-else".into(),
            destination,
        };

        assert_eq!(
            validate_write_request(&request),
            Err(WriteSafetyError::ConfirmationMismatch)
        );
    }

    #[test]
    fn produces_backend_plan_for_safe_route() {
        let destination = target();
        let request = WriteRequest {
            source_path: "ubuntu.iso".into(),
            host: HostPlatform::Linux,
            target: MediaTarget::LinuxLive,
            confirmation_phrase: required_confirmation(&destination),
            destination,
        };

        let plan = validate_write_request(&request).expect("valid request");
        assert_eq!(plan.backend, "linux-raw-image");
        assert_eq!(plan.destination_id, "disk-test-001");
    }
}
