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

pub use adb::*;
pub use fastboot::*;
pub use mtp::*;
pub use ptp::*;
pub use cdc::*;
pub use dfu::*;

/// Protocol trait for common operations
pub trait UsbProtocol {
    /// Protocol name
    fn name(&self) -> &'static str;
    
    /// Check if the protocol is connected
    fn is_connected(&self) -> bool;
    
    /// Get the protocol version
    fn version(&self) -> Option<String>;
}
