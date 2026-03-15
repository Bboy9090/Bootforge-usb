//! Error types for libbootforge

use thiserror::Error;

/// Central error enum for all bootforge operations
#[derive(Error, Debug)]
pub enum BootforgeError {
    #[error("USB subsystem unavailable")]
    UsbUnavailable,

    #[error("USB scan failed: {0}")]
    UsbScanFailed(String),

    #[error("Failed to read descriptor: {0}")]
    DescriptorReadFailed(String),

    #[error("Failed to open device: {0}")]
    DeviceOpenFailed(String),

    #[error("JSON serialization failed: {0}")]
    JsonSerializationFailed(String),

    #[error("USB error: {0}")]
    UsbError(#[from] rusb::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, BootforgeError>;
