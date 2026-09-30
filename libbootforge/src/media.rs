//! Cross-platform boot-media capability planning.
//!
//! This module is intentionally non-destructive. It answers what a host can prepare
//! for a requested target and which native backend will eventually be required.
//! Actual disk writes remain outside libbootforge's read-only core.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HostPlatform {
    Windows,
    MacOs,
    Linux,
    Unknown,
}

impl HostPlatform {
    pub fn current() -> Self {
        if cfg!(target_os = "windows") {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::MacOs
        } else if cfg!(target_os = "linux") {
            Self::Linux
        } else {
            Self::Unknown
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MediaTarget {
    WindowsInstaller,
    LinuxLive,
    MacOsOfficialInstaller,
    MacOsRawImage,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CapabilityLevel {
    Supported,
    SupportedWithNativeBackend,
    PreparationOnly,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MediaCapability {
    pub host: HostPlatform,
    pub target: MediaTarget,
    pub level: CapabilityLevel,
    pub backend: Option<&'static str>,
    pub notes: &'static str,
}

pub fn media_capability(host: HostPlatform, target: MediaTarget) -> MediaCapability {
    use CapabilityLevel::*;
    use HostPlatform::*;
    use MediaTarget::*;

    let (level, backend, notes) = match (host, target) {
        (Windows, WindowsInstaller) => (
            SupportedWithNativeBackend,
            Some("windows-media"),
            "Windows host can prepare Windows installer media using native disk and filesystem APIs.",
        ),
        (Windows, LinuxLive) => (
            SupportedWithNativeBackend,
            Some("windows-raw-image"),
            "Windows host can write validated Linux hybrid/raw images through the Windows disk backend.",
        ),
        (Windows, MacOsRawImage) => (
            SupportedWithNativeBackend,
            Some("windows-raw-image"),
            "Windows host may restore a user-supplied compatible raw macOS image; BootForge must not claim this is equivalent to Apple's official installer workflow.",
        ),
        (Windows, MacOsOfficialInstaller) => (
            PreparationOnly,
            None,
            "Apple's official createinstallmedia workflow requires macOS and an Apple installer application. Windows can validate or stage source material but cannot produce the official installer path.",
        ),

        (MacOs, WindowsInstaller) => (
            SupportedWithNativeBackend,
            Some("macos-windows-media"),
            "macOS host can prepare Windows installer media with FAT/exFAT staging plus large-WIM handling when required.",
        ),
        (MacOs, LinuxLive) => (
            SupportedWithNativeBackend,
            Some("macos-raw-image"),
            "macOS host can write validated Linux hybrid/raw images through the native disk backend.",
        ),
        (MacOs, MacOsOfficialInstaller) => (
            SupportedWithNativeBackend,
            Some("macos-createinstallmedia"),
            "macOS host can use Apple's supported createinstallmedia workflow when a valid installer application is present.",
        ),
        (MacOs, MacOsRawImage) => (
            SupportedWithNativeBackend,
            Some("macos-raw-image"),
            "macOS host can restore compatible user-supplied raw images through the native disk backend.",
        ),

        (Linux, WindowsInstaller) => (
            SupportedWithNativeBackend,
            Some("linux-windows-media"),
            "Linux host can prepare Windows installer media with partition/filesystem tooling and WIM splitting when needed.",
        ),
        (Linux, LinuxLive) => (
            SupportedWithNativeBackend,
            Some("linux-raw-image"),
            "Linux host can write validated Linux hybrid/raw images through the native block-device backend.",
        ),
        (Linux, MacOsRawImage) => (
            SupportedWithNativeBackend,
            Some("linux-raw-image"),
            "Linux host may restore a user-supplied compatible raw macOS image; BootForge must not claim this is equivalent to Apple's official installer workflow.",
        ),
        (Linux, MacOsOfficialInstaller) => (
            PreparationOnly,
            None,
            "Apple's official createinstallmedia workflow requires macOS and an Apple installer application. Linux can validate or stage source material but cannot produce the official installer path.",
        ),

        (Unknown, _) => (
            Unsupported,
            None,
            "The current operating system is not a supported BootForge media-creation host.",
        ),
    };

    MediaCapability {
        host,
        target,
        level,
        backend,
        notes,
    }
}

pub fn host_matrix(host: HostPlatform) -> Vec<MediaCapability> {
    [
        MediaTarget::WindowsInstaller,
        MediaTarget::LinuxLive,
        MediaTarget::MacOsOfficialInstaller,
        MediaTarget::MacOsRawImage,
    ]
    .into_iter()
    .map(|target| media_capability(host, target))
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_supported_desktop_host_can_plan_windows_and_linux_media() {
        for host in [HostPlatform::Windows, HostPlatform::MacOs, HostPlatform::Linux] {
            for target in [MediaTarget::WindowsInstaller, MediaTarget::LinuxLive] {
                let cap = media_capability(host, target);
                assert_ne!(cap.level, CapabilityLevel::Unsupported);
                assert!(cap.backend.is_some());
            }
        }
    }

    #[test]
    fn official_macos_installer_is_native_to_macos() {
        assert_eq!(
            media_capability(HostPlatform::MacOs, MediaTarget::MacOsOfficialInstaller).level,
            CapabilityLevel::SupportedWithNativeBackend
        );
        assert_eq!(
            media_capability(HostPlatform::Windows, MediaTarget::MacOsOfficialInstaller).level,
            CapabilityLevel::PreparationOnly
        );
        assert_eq!(
            media_capability(HostPlatform::Linux, MediaTarget::MacOsOfficialInstaller).level,
            CapabilityLevel::PreparationOnly
        );
    }

    #[test]
    fn all_three_hosts_have_a_raw_macos_image_path() {
        for host in [HostPlatform::Windows, HostPlatform::MacOs, HostPlatform::Linux] {
            let cap = media_capability(host, MediaTarget::MacOsRawImage);
            assert_eq!(cap.level, CapabilityLevel::SupportedWithNativeBackend);
            assert!(cap.backend.is_some());
        }
    }
}
