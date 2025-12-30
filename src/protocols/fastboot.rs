//! Fastboot Protocol Implementation
//!
//! Implements the Fastboot protocol for Android bootloader communication.

use crate::communication::{DeviceHandle, BulkTransfer};
use crate::errors::UsbError;
use super::UsbProtocol;
use std::time::Duration;

/// Fastboot response types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FastbootResponse {
    /// Operation succeeded
    Okay(String),
    /// Device is busy, please wait
    Info(String),
    /// Operation failed
    Fail(String),
    /// Data transfer expected
    Data(u32),
}

impl FastbootResponse {
    /// Parse a fastboot response
    pub fn parse(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() < 4 {
            return Err("Response too short");
        }
        
        let prefix = std::str::from_utf8(&data[..4]).map_err(|_| "Invalid UTF-8")?;
        let message = std::str::from_utf8(&data[4..])
            .unwrap_or("")
            .trim()
            .to_string();
        
        match prefix {
            "OKAY" => Ok(Self::Okay(message)),
            "INFO" => Ok(Self::Info(message)),
            "FAIL" => Ok(Self::Fail(message)),
            "DATA" => {
                // DATA is followed by 8 hex digits
                if data.len() >= 12 {
                    let size_str = std::str::from_utf8(&data[4..12]).map_err(|_| "Invalid size")?;
                    let size = u32::from_str_radix(size_str, 16).map_err(|_| "Invalid size")?;
                    Ok(Self::Data(size))
                } else {
                    Err("DATA response too short")
                }
            }
            _ => Err("Unknown response prefix"),
        }
    }
    
    /// Check if response is successful
    pub fn is_okay(&self) -> bool {
        matches!(self, Self::Okay(_))
    }
    
    /// Check if response is an error
    pub fn is_fail(&self) -> bool {
        matches!(self, Self::Fail(_))
    }
}

/// Fastboot variable info
#[derive(Debug, Clone)]
pub struct FastbootVariable {
    /// Variable name
    pub name: String,
    /// Variable value
    pub value: String,
}

/// Fastboot partition info
#[derive(Debug, Clone)]
pub struct FastbootPartition {
    /// Partition name
    pub name: String,
    /// Partition size in bytes
    pub size: Option<u64>,
    /// Partition type
    pub partition_type: Option<String>,
}

/// Fastboot client
pub struct FastbootClient<'a> {
    handle: &'a DeviceHandle,
    ep_in: u8,
    ep_out: u8,
    connected: bool,
    timeout: Duration,
}

impl<'a> FastbootClient<'a> {
    /// Fastboot interface class
    pub const CLASS: u8 = 0xFF;
    /// Fastboot interface subclass
    pub const SUBCLASS: u8 = 0x42;
    /// Fastboot interface protocol
    pub const PROTOCOL: u8 = 0x03;
    
    /// Create a new Fastboot client
    pub fn new(handle: &'a DeviceHandle, ep_in: u8, ep_out: u8) -> Self {
        Self {
            handle,
            ep_in: ep_in | 0x80,
            ep_out: ep_out & 0x7F,
            connected: false,
            timeout: Duration::from_secs(30),
        }
    }
    
    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    /// Connect to the fastboot device
    pub fn connect(&mut self) -> Result<(), UsbError> {
        // Try to get a variable to verify connection
        match self.get_var("product") {
            Ok(_) => {
                self.connected = true;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
    
    /// Send a command and get the response
    pub fn command(&self, cmd: &str) -> Result<FastbootResponse, UsbError> {
        let bulk = BulkTransfer::new(self.handle).with_timeout(self.timeout);
        
        // Send command
        bulk.write(self.ep_out, cmd.as_bytes())?;
        
        // Read responses until OKAY or FAIL
        loop {
            let mut buf = [0u8; 256];
            let bytes = bulk.read(self.ep_in, &mut buf)?;
            
            let response = FastbootResponse::parse(&buf[..bytes])
                .map_err(|e| UsbError::Parse(e.to_string()))?;
            
            match &response {
                FastbootResponse::Info(_) => {
                    // Info is intermediate, continue reading
                    continue;
                }
                _ => return Ok(response),
            }
        }
    }
    
    /// Send a command and collect all INFO messages
    pub fn command_with_info(&self, cmd: &str) -> Result<(FastbootResponse, Vec<String>), UsbError> {
        let bulk = BulkTransfer::new(self.handle).with_timeout(self.timeout);
        
        // Send command
        bulk.write(self.ep_out, cmd.as_bytes())?;
        
        let mut info_messages = Vec::new();
        
        // Read responses until OKAY or FAIL
        loop {
            let mut buf = [0u8; 256];
            let bytes = bulk.read(self.ep_in, &mut buf)?;
            
            let response = FastbootResponse::parse(&buf[..bytes])
                .map_err(|e| UsbError::Parse(e.to_string()))?;
            
            match &response {
                FastbootResponse::Info(msg) => {
                    info_messages.push(msg.clone());
                    continue;
                }
                _ => return Ok((response, info_messages)),
            }
        }
    }
    
    /// Get a variable value
    pub fn get_var(&self, name: &str) -> Result<String, UsbError> {
        let cmd = format!("getvar:{}", name);
        let response = self.command(&cmd)?;
        
        match response {
            FastbootResponse::Okay(value) => Ok(value),
            FastbootResponse::Fail(msg) => Err(UsbError::Unknown(msg)),
            _ => Err(UsbError::Unknown("Unexpected response".into())),
        }
    }
    
    /// Get all variables
    pub fn get_all_vars(&self) -> Result<Vec<FastbootVariable>, UsbError> {
        let (response, info) = self.command_with_info("getvar:all")?;
        
        if response.is_fail() {
            // Some devices don't support getvar:all
            return Ok(Vec::new());
        }
        
        let vars: Vec<FastbootVariable> = info.iter()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                if parts.len() == 2 {
                    Some(FastbootVariable {
                        name: parts[0].trim().to_string(),
                        value: parts[1].trim().to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();
        
        Ok(vars)
    }
    
    /// Get device info
    pub fn get_device_info(&self) -> Result<FastbootDeviceInfo, UsbError> {
        Ok(FastbootDeviceInfo {
            product: self.get_var("product").ok(),
            variant: self.get_var("variant").ok(),
            serialno: self.get_var("serialno").ok(),
            version_bootloader: self.get_var("version-bootloader").ok(),
            version_baseband: self.get_var("version-baseband").ok(),
            secure: self.get_var("secure").ok().map(|v| v == "yes"),
            unlocked: self.get_var("unlocked").ok().map(|v| v == "yes"),
            off_mode_charge: self.get_var("off-mode-charge").ok().map(|v| v == "1"),
            battery_voltage: self.get_var("battery-voltage").ok()
                .and_then(|v| v.trim_end_matches("mV").parse().ok()),
            battery_soc_ok: self.get_var("battery-soc-ok").ok().map(|v| v == "yes"),
            max_download_size: self.get_var("max-download-size").ok()
                .and_then(|v| {
                    if let Some(stripped) = v.strip_prefix("0x") {
                        u64::from_str_radix(stripped, 16).ok()
                    } else {
                        v.parse().ok()
                    }
                }),
        })
    }
    
    /// Flash a partition
    pub fn flash(&self, partition: &str, data: &[u8]) -> Result<(), UsbError> {
        // First, download the data
        self.download(data)?;
        
        // Then flash to partition
        let cmd = format!("flash:{}", partition);
        let response = self.command(&cmd)?;
        
        match response {
            FastbootResponse::Okay(_) => Ok(()),
            FastbootResponse::Fail(msg) => Err(UsbError::Unknown(format!("Flash failed: {}", msg))),
            _ => Err(UsbError::Unknown("Unexpected response".into())),
        }
    }
    
    /// Download data to device
    pub fn download(&self, data: &[u8]) -> Result<(), UsbError> {
        let bulk = BulkTransfer::new(self.handle).with_timeout(self.timeout);
        
        // Send download command with size
        let cmd = format!("download:{:08x}", data.len());
        bulk.write(self.ep_out, cmd.as_bytes())?;
        
        // Wait for DATA response
        let mut buf = [0u8; 256];
        let bytes = bulk.read(self.ep_in, &mut buf)?;
        
        let response = FastbootResponse::parse(&buf[..bytes])
            .map_err(|e| UsbError::Parse(e.to_string()))?;
        
        match response {
            FastbootResponse::Data(size) => {
                if size as usize != data.len() {
                    return Err(UsbError::Unknown("Size mismatch".into()));
                }
            }
            FastbootResponse::Fail(msg) => {
                return Err(UsbError::Unknown(format!("Download rejected: {}", msg)));
            }
            _ => return Err(UsbError::Unknown("Unexpected response".into())),
        }
        
        // Send data in chunks
        let chunk_size = 1024 * 1024; // 1MB chunks
        for chunk in data.chunks(chunk_size) {
            bulk.write(self.ep_out, chunk)?;
        }
        
        // Wait for final OKAY
        let bytes = bulk.read(self.ep_in, &mut buf)?;
        let response = FastbootResponse::parse(&buf[..bytes])
            .map_err(|e| UsbError::Parse(e.to_string()))?;
        
        match response {
            FastbootResponse::Okay(_) => Ok(()),
            FastbootResponse::Fail(msg) => Err(UsbError::Unknown(format!("Download failed: {}", msg))),
            _ => Err(UsbError::Unknown("Unexpected response".into())),
        }
    }
    
    /// Erase a partition
    pub fn erase(&self, partition: &str) -> Result<(), UsbError> {
        let cmd = format!("erase:{}", partition);
        let response = self.command(&cmd)?;
        
        match response {
            FastbootResponse::Okay(_) => Ok(()),
            FastbootResponse::Fail(msg) => Err(UsbError::Unknown(format!("Erase failed: {}", msg))),
            _ => Err(UsbError::Unknown("Unexpected response".into())),
        }
    }
    
    /// Reboot the device
    pub fn reboot(&self) -> Result<(), UsbError> {
        let response = self.command("reboot")?;
        
        match response {
            FastbootResponse::Okay(_) => Ok(()),
            FastbootResponse::Fail(msg) => Err(UsbError::Unknown(format!("Reboot failed: {}", msg))),
            _ => Err(UsbError::Unknown("Unexpected response".into())),
        }
    }
    
    /// Reboot to bootloader
    pub fn reboot_bootloader(&self) -> Result<(), UsbError> {
        let response = self.command("reboot-bootloader")?;
        
        match response {
            FastbootResponse::Okay(_) => Ok(()),
            FastbootResponse::Fail(msg) => Err(UsbError::Unknown(msg)),
            _ => Err(UsbError::Unknown("Unexpected response".into())),
        }
    }
    
    /// Continue boot
    pub fn continue_boot(&self) -> Result<(), UsbError> {
        let response = self.command("continue")?;
        
        match response {
            FastbootResponse::Okay(_) => Ok(()),
            FastbootResponse::Fail(msg) => Err(UsbError::Unknown(msg)),
            _ => Err(UsbError::Unknown("Unexpected response".into())),
        }
    }
    
    /// OEM command
    pub fn oem(&self, command: &str) -> Result<(FastbootResponse, Vec<String>), UsbError> {
        let cmd = format!("oem {}", command);
        self.command_with_info(&cmd)
    }
    
    /// Unlock bootloader
    pub fn oem_unlock(&self) -> Result<(), UsbError> {
        let response = self.command("oem unlock")?;
        
        match response {
            FastbootResponse::Okay(_) => Ok(()),
            FastbootResponse::Fail(msg) => Err(UsbError::Unknown(format!("Unlock failed: {}", msg))),
            _ => Err(UsbError::Unknown("Unexpected response".into())),
        }
    }
    
    /// Lock bootloader
    pub fn oem_lock(&self) -> Result<(), UsbError> {
        let response = self.command("oem lock")?;
        
        match response {
            FastbootResponse::Okay(_) => Ok(()),
            FastbootResponse::Fail(msg) => Err(UsbError::Unknown(format!("Lock failed: {}", msg))),
            _ => Err(UsbError::Unknown("Unexpected response".into())),
        }
    }
}

impl UsbProtocol for FastbootClient<'_> {
    fn name(&self) -> &'static str {
        "Fastboot"
    }
    
    fn is_connected(&self) -> bool {
        self.connected
    }
    
    fn version(&self) -> Option<String> {
        None // Fastboot doesn't have a protocol version
    }
}

/// Fastboot device information
#[derive(Debug, Clone, Default)]
pub struct FastbootDeviceInfo {
    /// Product name
    pub product: Option<String>,
    /// Product variant
    pub variant: Option<String>,
    /// Serial number
    pub serialno: Option<String>,
    /// Bootloader version
    pub version_bootloader: Option<String>,
    /// Baseband version
    pub version_baseband: Option<String>,
    /// Is secure boot enabled
    pub secure: Option<bool>,
    /// Is bootloader unlocked
    pub unlocked: Option<bool>,
    /// Off-mode charging enabled
    pub off_mode_charge: Option<bool>,
    /// Battery voltage in mV
    pub battery_voltage: Option<u32>,
    /// Is battery sufficient for flashing
    pub battery_soc_ok: Option<bool>,
    /// Maximum download size
    pub max_download_size: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_parse() {
        assert!(matches!(
            FastbootResponse::parse(b"OKAY"),
            Ok(FastbootResponse::Okay(_))
        ));
        
        assert!(matches!(
            FastbootResponse::parse(b"FAILerror"),
            Ok(FastbootResponse::Fail(msg)) if msg == "error"
        ));
        
        assert!(matches!(
            FastbootResponse::parse(b"DATA00001000"),
            Ok(FastbootResponse::Data(0x1000))
        ));
        
        assert!(matches!(
            FastbootResponse::parse(b"INFOmessage"),
            Ok(FastbootResponse::Info(msg)) if msg == "message"
        ));
    }

    #[test]
    fn test_device_info_default() {
        let info = FastbootDeviceInfo::default();
        assert!(info.product.is_none());
        assert!(info.unlocked.is_none());
    }
}
