//! DFU (Device Firmware Upgrade) Implementation
//!
//! Implements the USB DFU protocol for firmware updates.

use crate::communication::DeviceHandle;
use crate::errors::UsbError;
use super::UsbProtocol;
use std::time::Duration;

/// DFU class requests
pub mod request {
    /// Detach from application mode
    pub const DETACH: u8 = 0;
    /// Download firmware
    pub const DNLOAD: u8 = 1;
    /// Upload firmware
    pub const UPLOAD: u8 = 2;
    /// Get DFU status
    pub const GETSTATUS: u8 = 3;
    /// Clear status
    pub const CLRSTATUS: u8 = 4;
    /// Get state
    pub const GETSTATE: u8 = 5;
    /// Abort operation
    pub const ABORT: u8 = 6;
}

/// DFU states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DfuState {
    /// App is in idle state
    AppIdle = 0,
    /// App is waiting for detach
    AppDetach = 1,
    /// DFU mode idle
    DfuIdle = 2,
    /// Download is in sync
    DfuDnloadSync = 3,
    /// Download is busy
    DfuDnBusy = 4,
    /// Download is idle (waiting for more data)
    DfuDnloadIdle = 5,
    /// Manifestation is in sync
    DfuManifestSync = 6,
    /// Manifestation is in progress
    DfuManifest = 7,
    /// Manifestation is waiting for reset
    DfuManifestWaitReset = 8,
    /// Upload is idle
    DfuUploadIdle = 9,
    /// Error occurred
    DfuError = 10,
}

impl DfuState {
    /// Parse from byte
    pub fn from_byte(b: u8) -> Self {
        match b {
            0 => Self::AppIdle,
            1 => Self::AppDetach,
            2 => Self::DfuIdle,
            3 => Self::DfuDnloadSync,
            4 => Self::DfuDnBusy,
            5 => Self::DfuDnloadIdle,
            6 => Self::DfuManifestSync,
            7 => Self::DfuManifest,
            8 => Self::DfuManifestWaitReset,
            9 => Self::DfuUploadIdle,
            _ => Self::DfuError,
        }
    }
    
    /// Get state name
    pub fn name(&self) -> &'static str {
        match self {
            Self::AppIdle => "appIDLE",
            Self::AppDetach => "appDETACH",
            Self::DfuIdle => "dfuIDLE",
            Self::DfuDnloadSync => "dfuDNLOAD-SYNC",
            Self::DfuDnBusy => "dfuDNBUSY",
            Self::DfuDnloadIdle => "dfuDNLOAD-IDLE",
            Self::DfuManifestSync => "dfuMANIFEST-SYNC",
            Self::DfuManifest => "dfuMANIFEST",
            Self::DfuManifestWaitReset => "dfuMANIFEST-WAIT-RESET",
            Self::DfuUploadIdle => "dfuUPLOAD-IDLE",
            Self::DfuError => "dfuERROR",
        }
    }
    
    /// Check if in DFU mode (not application mode)
    pub fn is_dfu_mode(&self) -> bool {
        !matches!(self, Self::AppIdle | Self::AppDetach)
    }
}

/// DFU status codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DfuStatus {
    /// No error
    Ok = 0x00,
    /// File error
    ErrTarget = 0x01,
    /// File is not targeted for this device
    ErrFile = 0x02,
    /// Firmware write failed
    ErrWrite = 0x03,
    /// Firmware erase failed
    ErrErase = 0x04,
    /// Erase check failed
    ErrCheckErased = 0x05,
    /// Program failed
    ErrProg = 0x06,
    /// Verify failed
    ErrVerify = 0x07,
    /// Address out of range
    ErrAddress = 0x08,
    /// Received DFU_DNLOAD with nonzero length in a state other than dfuDNLOAD-IDLE
    ErrNotdone = 0x09,
    /// Firmware corrupt
    ErrFirmware = 0x0A,
    /// Vendor-specific error
    ErrVendor = 0x0B,
    /// USB reset detected
    ErrUsbr = 0x0C,
    /// POR detected
    ErrPor = 0x0D,
    /// Unknown error
    ErrUnknown = 0x0E,
    /// Stalled
    ErrStalledPkt = 0x0F,
}

impl DfuStatus {
    /// Parse from byte
    pub fn from_byte(b: u8) -> Self {
        match b {
            0x00 => Self::Ok,
            0x01 => Self::ErrTarget,
            0x02 => Self::ErrFile,
            0x03 => Self::ErrWrite,
            0x04 => Self::ErrErase,
            0x05 => Self::ErrCheckErased,
            0x06 => Self::ErrProg,
            0x07 => Self::ErrVerify,
            0x08 => Self::ErrAddress,
            0x09 => Self::ErrNotdone,
            0x0A => Self::ErrFirmware,
            0x0B => Self::ErrVendor,
            0x0C => Self::ErrUsbr,
            0x0D => Self::ErrPor,
            0x0E => Self::ErrUnknown,
            _ => Self::ErrStalledPkt,
        }
    }
    
    /// Get status name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::ErrTarget => "errTARGET",
            Self::ErrFile => "errFILE",
            Self::ErrWrite => "errWRITE",
            Self::ErrErase => "errERASE",
            Self::ErrCheckErased => "errCHECK_ERASED",
            Self::ErrProg => "errPROG",
            Self::ErrVerify => "errVERIFY",
            Self::ErrAddress => "errADDRESS",
            Self::ErrNotdone => "errNOTDONE",
            Self::ErrFirmware => "errFIRMWARE",
            Self::ErrVendor => "errVENDOR",
            Self::ErrUsbr => "errUSBR",
            Self::ErrPor => "errPOR",
            Self::ErrUnknown => "errUNKNOWN",
            Self::ErrStalledPkt => "errSTALLEDPKT",
        }
    }
    
    /// Check if no error
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok)
    }
}

/// DFU GET_STATUS response (6 bytes)
#[derive(Debug, Clone)]
pub struct DfuStatusResponse {
    /// Status code
    pub status: DfuStatus,
    /// Poll timeout in milliseconds
    pub poll_timeout: u32,
    /// Current state
    pub state: DfuState,
    /// String index for status description
    pub i_string: u8,
}

impl DfuStatusResponse {
    /// Parse from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() < 6 {
            return Err("Status response too short");
        }
        
        Ok(Self {
            status: DfuStatus::from_byte(data[0]),
            poll_timeout: u32::from_le_bytes([data[1], data[2], data[3], 0]),
            state: DfuState::from_byte(data[4]),
            i_string: data[5],
        })
    }
}

/// DFU functional descriptor
#[derive(Debug, Clone)]
pub struct DfuFunctionalDescriptor {
    /// Will detach (bit 0)
    pub will_detach: bool,
    /// Manifestation tolerant (bit 1)
    pub manifestation_tolerant: bool,
    /// Can upload (bit 2)
    pub can_upload: bool,
    /// Can download (bit 3)
    pub can_download: bool,
    /// Detach timeout
    pub detach_timeout: u16,
    /// Transfer size
    pub transfer_size: u16,
    /// DFU version
    pub dfu_version: u16,
}

impl DfuFunctionalDescriptor {
    /// Parse from bytes (9 bytes for DFU descriptor)
    pub fn from_bytes(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() < 9 {
            return Err("Functional descriptor too short");
        }
        
        // data[0] = length
        // data[1] = descriptor type (0x21 for DFU)
        let attributes = data[2];
        
        Ok(Self {
            will_detach: (attributes & 0x08) != 0,
            manifestation_tolerant: (attributes & 0x04) != 0,
            can_upload: (attributes & 0x02) != 0,
            can_download: (attributes & 0x01) != 0,
            detach_timeout: u16::from_le_bytes([data[3], data[4]]),
            transfer_size: u16::from_le_bytes([data[5], data[6]]),
            dfu_version: u16::from_le_bytes([data[7], data[8]]),
        })
    }
    
    /// Get DFU version string
    pub fn version_string(&self) -> String {
        format!("{}.{}", self.dfu_version >> 8, self.dfu_version & 0xFF)
    }
}

/// DFU client
pub struct DfuClient<'a> {
    handle: &'a DeviceHandle,
    interface: u8,
    transfer_size: u16,
    timeout: Duration,
    state: DfuState,
}

impl<'a> DfuClient<'a> {
    /// DFU interface class
    pub const CLASS: u8 = 0xFE;
    /// DFU interface subclass
    pub const SUBCLASS: u8 = 0x01;
    /// DFU runtime protocol
    pub const PROTOCOL_RUNTIME: u8 = 0x01;
    /// DFU mode protocol
    pub const PROTOCOL_DFU: u8 = 0x02;
    
    /// Create a new DFU client
    pub fn new(handle: &'a DeviceHandle, interface: u8, transfer_size: u16) -> Self {
        Self {
            handle,
            interface,
            transfer_size,
            timeout: Duration::from_secs(10),
            state: DfuState::DfuIdle,
        }
    }
    
    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    /// Get current state
    pub fn state(&self) -> DfuState {
        self.state
    }
    
    /// Get DFU status
    pub fn get_status(&mut self) -> Result<DfuStatusResponse, UsbError> {
        let request_type = 0xA1; // Device to host, class, interface
        let mut buf = [0u8; 6];
        
        self.handle.control_read(
            request_type,
            request::GETSTATUS,
            0,
            self.interface as u16,
            &mut buf,
            self.timeout,
        )?;
        
        let status = DfuStatusResponse::from_bytes(&buf)
            .map_err(|e| UsbError::Parse(e.to_string()))?;
        
        self.state = status.state;
        Ok(status)
    }
    
    /// Get current DFU state
    pub fn get_state(&mut self) -> Result<DfuState, UsbError> {
        let request_type = 0xA1;
        let mut buf = [0u8; 1];
        
        self.handle.control_read(
            request_type,
            request::GETSTATE,
            0,
            self.interface as u16,
            &mut buf,
            self.timeout,
        )?;
        
        self.state = DfuState::from_byte(buf[0]);
        Ok(self.state)
    }
    
    /// Clear error status
    pub fn clear_status(&mut self) -> Result<(), UsbError> {
        let request_type = 0x21; // Host to device, class, interface
        
        self.handle.control_write(
            request_type,
            request::CLRSTATUS,
            0,
            self.interface as u16,
            &[],
            self.timeout,
        )?;
        
        Ok(())
    }
    
    /// Abort current operation
    pub fn abort(&mut self) -> Result<(), UsbError> {
        let request_type = 0x21;
        
        self.handle.control_write(
            request_type,
            request::ABORT,
            0,
            self.interface as u16,
            &[],
            self.timeout,
        )?;
        
        self.state = DfuState::DfuIdle;
        Ok(())
    }
    
    /// Detach from application mode
    pub fn detach(&self, timeout_ms: u16) -> Result<(), UsbError> {
        let request_type = 0x21;
        
        self.handle.control_write(
            request_type,
            request::DETACH,
            timeout_ms,
            self.interface as u16,
            &[],
            self.timeout,
        )?;
        
        Ok(())
    }
    
    /// Download a block of firmware
    fn download_block(&self, block_num: u16, data: &[u8]) -> Result<(), UsbError> {
        let request_type = 0x21;
        
        self.handle.control_write(
            request_type,
            request::DNLOAD,
            block_num,
            self.interface as u16,
            data,
            self.timeout,
        )?;
        
        Ok(())
    }
    
    /// Upload a block of firmware
    fn upload_block(&self, block_num: u16, buf: &mut [u8]) -> Result<usize, UsbError> {
        let request_type = 0xA1;
        
        self.handle.control_read(
            request_type,
            request::UPLOAD,
            block_num,
            self.interface as u16,
            buf,
            self.timeout,
        )
    }
    
    /// Wait for device to be ready
    fn wait_for_ready(&mut self) -> Result<(), UsbError> {
        loop {
            let status = self.get_status()?;
            
            if !status.status.is_ok() {
                return Err(UsbError::Unknown(format!(
                    "DFU error: {}",
                    status.status.name()
                )));
            }
            
            match status.state {
                DfuState::DfuDnloadSync | DfuState::DfuDnBusy | DfuState::DfuManifestSync => {
                    // Wait and poll again
                    std::thread::sleep(Duration::from_millis(status.poll_timeout as u64));
                }
                DfuState::DfuDnloadIdle | DfuState::DfuIdle | DfuState::DfuManifest => {
                    return Ok(());
                }
                DfuState::DfuError => {
                    return Err(UsbError::Unknown("DFU error state".into()));
                }
                _ => {
                    return Err(UsbError::Unknown(format!(
                        "Unexpected state: {}",
                        status.state.name()
                    )));
                }
            }
        }
    }
    
    /// Download firmware to device
    pub fn download(&mut self, firmware: &[u8], progress: Option<&dyn Fn(usize, usize)>) -> Result<(), UsbError> {
        // Ensure we're in idle state
        if self.state != DfuState::DfuIdle {
            self.abort()?;
            self.get_status()?;
        }
        
        let total = firmware.len();
        let mut offset = 0;
        let mut block_num: u16 = 0;
        
        while offset < total {
            let chunk_size = (total - offset).min(self.transfer_size as usize);
            let chunk = &firmware[offset..offset + chunk_size];
            
            self.download_block(block_num, chunk)?;
            self.wait_for_ready()?;
            
            offset += chunk_size;
            block_num += 1;
            
            if let Some(cb) = progress {
                cb(offset, total);
            }
        }
        
        // Send empty download to signal end
        self.download_block(block_num, &[])?;
        
        // Wait for manifestation
        loop {
            let status = self.get_status()?;
            
            if !status.status.is_ok() {
                return Err(UsbError::Unknown(format!(
                    "Manifestation error: {}",
                    status.status.name()
                )));
            }
            
            match status.state {
                DfuState::DfuManifest | DfuState::DfuManifestSync => {
                    std::thread::sleep(Duration::from_millis(status.poll_timeout as u64));
                }
                DfuState::DfuManifestWaitReset | DfuState::DfuIdle => {
                    return Ok(());
                }
                _ => {
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        }
    }
    
    /// Upload firmware from device
    pub fn upload(&mut self, max_size: usize, progress: Option<&dyn Fn(usize, usize)>) -> Result<Vec<u8>, UsbError> {
        // Ensure we're in idle state
        if self.state != DfuState::DfuIdle {
            self.abort()?;
            self.get_status()?;
        }
        
        let mut firmware = Vec::new();
        let mut block_num: u16 = 0;
        let mut buf = vec![0u8; self.transfer_size as usize];
        
        loop {
            let bytes = self.upload_block(block_num, &mut buf)?;
            
            if bytes == 0 {
                break;
            }
            
            firmware.extend_from_slice(&buf[..bytes]);
            block_num += 1;
            
            if let Some(cb) = progress {
                cb(firmware.len(), max_size);
            }
            
            if firmware.len() >= max_size {
                break;
            }
            
            if bytes < self.transfer_size as usize {
                // Short read indicates end
                break;
            }
            
            self.get_status()?;
        }
        
        Ok(firmware)
    }
}

impl UsbProtocol for DfuClient<'_> {
    fn name(&self) -> &'static str {
        "DFU"
    }
    
    fn is_connected(&self) -> bool {
        self.state.is_dfu_mode()
    }
    
    fn version(&self) -> Option<String> {
        Some("1.1".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dfu_state() {
        assert_eq!(DfuState::from_byte(0), DfuState::AppIdle);
        assert_eq!(DfuState::from_byte(2), DfuState::DfuIdle);
        assert!(DfuState::DfuIdle.is_dfu_mode());
        assert!(!DfuState::AppIdle.is_dfu_mode());
    }

    #[test]
    fn test_dfu_status() {
        assert!(DfuStatus::Ok.is_ok());
        assert!(!DfuStatus::ErrTarget.is_ok());
        assert_eq!(DfuStatus::Ok.name(), "OK");
    }

    #[test]
    fn test_status_response() {
        let data = [0x00, 0x10, 0x00, 0x00, 0x02, 0x00];
        let response = DfuStatusResponse::from_bytes(&data).unwrap();
        assert!(response.status.is_ok());
        assert_eq!(response.poll_timeout, 16);
        assert_eq!(response.state, DfuState::DfuIdle);
    }

    #[test]
    fn test_functional_descriptor() {
        let data = [0x09, 0x21, 0x0B, 0x00, 0x10, 0x00, 0x01, 0x10, 0x01];
        let desc = DfuFunctionalDescriptor::from_bytes(&data).unwrap();
        assert!(desc.will_detach);
        assert!(desc.can_upload);
        assert!(desc.can_download);
        assert_eq!(desc.transfer_size, 256);
    }
}
