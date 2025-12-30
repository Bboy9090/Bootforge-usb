//! USB Protocol Implementations - OMEGA MODE
//!
//! Full protocol implementations for device communication including:
//! - ADB (Android Debug Bridge)
//! - Fastboot
//! - MTP (Media Transfer Protocol)
//! - PTP (Picture Transfer Protocol)
//! - CDC (Communications Device Class)
//! - DFU (Device Firmware Upgrade)

pub mod adb;
pub mod fastboot;
pub mod mtp;
pub mod ptp;
pub mod cdc;
pub mod dfu;

// Re-export main types (avoid ambiguous glob re-exports)
pub use adb::{AdbClient, AdbMessage, AdbState, AdbStream};
pub use fastboot::{FastbootClient, FastbootResponse, FastbootDeviceInfo, FastbootVariable, FastbootPartition};
pub use mtp::{MtpClient, MtpContainer, MtpDeviceInfo, MtpStorageInfo, MtpObjectInfo};
pub use ptp::{PtpClient, PtpDeviceInfo, PtpStorageInfo, PtpObjectInfo, PtpEvent};
pub use cdc::{CdcAcmClient, LineCoding, CdcNetworkInfo};
pub use dfu::{DfuClient, DfuState, DfuStatus, DfuStatusResponse, DfuFunctionalDescriptor};

/// Protocol trait for common operations
pub trait UsbProtocol {
    /// Protocol name
    fn name(&self) -> &'static str;
    
    /// Check if the protocol is connected
    fn is_connected(&self) -> bool;
    
    /// Get the protocol version
    fn version(&self) -> Option<String>;
}
