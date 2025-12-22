use thiserror::Error;

#[derive(Error, Debug)]
pub enum UsbError {
    #[error("Platform error: {0}")]
    Platform(String),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("USB library error: {0}")]
    UsbLib(#[from] rusb::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}
