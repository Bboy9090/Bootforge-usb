use crate::model::UsbDeviceRecord;
use std::sync::mpsc::Receiver;

#[derive(Debug, Clone)]
pub enum DeviceEvent {
    Added(UsbDeviceRecord),
    Removed(UsbDeviceRecord),
    Changed(UsbDeviceRecord),
}

pub trait DeviceWatcher: Send + Sync {
    fn start(&mut self) -> Result<Receiver<DeviceEvent>, Box<dyn std::error::Error>>;
    fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

// Re-export platform-specific watcher
#[cfg(target_os = "linux")]
pub use linux::LinuxDeviceWatcher as PlatformWatcher;
#[cfg(target_os = "macos")]
pub use macos::MacOSDeviceWatcher as PlatformWatcher;
#[cfg(target_os = "windows")]
pub use windows::WindowsDeviceWatcher as PlatformWatcher;
