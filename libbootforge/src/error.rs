//! Error types for libbootforge.

use thiserror::Error;

/// Central error enum for all BootForge USB operations.
#[derive(Error, Debug)]
pub enum BootforgeError {
    #[error("USB subsystem unavailable")]
    UsbUnavailable,

    #[error("USB scan failed: {0}")]
    UsbScanFailed(String),

    #[error("failed to read descriptor: {0}")]
    DescriptorReadFailed(String),

    #[error("failed to open device: {0}")]
    DeviceOpenFailed(String),

    #[error("JSON serialization failed: {0}")]
    JsonSerializationFailed(String),

    #[error("evidence chain invalid: {0}")]
    EvidenceChainInvalid(String),

    #[error("USB error: {0}")]
    UsbError(#[from] rusb::Error),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, BootforgeError>;
