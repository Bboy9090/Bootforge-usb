//! USB Control Transfer Utilities
//!
//! Provides helpers for common control transfer operations.

use super::{DeviceHandle, DEFAULT_TIMEOUT};
use crate::errors::UsbError;
use std::time::Duration;

/// Standard USB request types
pub mod request_type {
    /// Direction: Device to Host (IN)
    pub const DIR_IN: u8 = 0x80;
    /// Direction: Host to Device (OUT)
    pub const DIR_OUT: u8 = 0x00;
    
    /// Type: Standard
    pub const TYPE_STANDARD: u8 = 0x00;
    /// Type: Class
    pub const TYPE_CLASS: u8 = 0x20;
    /// Type: Vendor
    pub const TYPE_VENDOR: u8 = 0x40;
    
    /// Recipient: Device
    pub const RECIP_DEVICE: u8 = 0x00;
    /// Recipient: Interface
    pub const RECIP_INTERFACE: u8 = 0x01;
    /// Recipient: Endpoint
    pub const RECIP_ENDPOINT: u8 = 0x02;
    /// Recipient: Other
    pub const RECIP_OTHER: u8 = 0x03;
    
    /// Build a request type byte
    pub const fn build(direction: u8, req_type: u8, recipient: u8) -> u8 {
        direction | req_type | recipient
    }
}

/// Standard USB requests
pub mod request {
    /// GET_STATUS
    pub const GET_STATUS: u8 = 0x00;
    /// CLEAR_FEATURE
    pub const CLEAR_FEATURE: u8 = 0x01;
    /// SET_FEATURE
    pub const SET_FEATURE: u8 = 0x03;
    /// SET_ADDRESS
    pub const SET_ADDRESS: u8 = 0x05;
    /// GET_DESCRIPTOR
    pub const GET_DESCRIPTOR: u8 = 0x06;
    /// SET_DESCRIPTOR
    pub const SET_DESCRIPTOR: u8 = 0x07;
    /// GET_CONFIGURATION
    pub const GET_CONFIGURATION: u8 = 0x08;
    /// SET_CONFIGURATION
    pub const SET_CONFIGURATION: u8 = 0x09;
    /// GET_INTERFACE
    pub const GET_INTERFACE: u8 = 0x0A;
    /// SET_INTERFACE
    pub const SET_INTERFACE: u8 = 0x0B;
    /// SYNCH_FRAME
    pub const SYNCH_FRAME: u8 = 0x0C;
}

/// Descriptor types
pub mod descriptor_type {
    /// Device descriptor
    pub const DEVICE: u8 = 0x01;
    /// Configuration descriptor
    pub const CONFIGURATION: u8 = 0x02;
    /// String descriptor
    pub const STRING: u8 = 0x03;
    /// Interface descriptor
    pub const INTERFACE: u8 = 0x04;
    /// Endpoint descriptor
    pub const ENDPOINT: u8 = 0x05;
    /// Device Qualifier descriptor
    pub const DEVICE_QUALIFIER: u8 = 0x06;
    /// Other Speed Configuration descriptor
    pub const OTHER_SPEED_CONFIG: u8 = 0x07;
    /// Interface Power descriptor
    pub const INTERFACE_POWER: u8 = 0x08;
    /// BOS descriptor
    pub const BOS: u8 = 0x0F;
    /// HID descriptor
    pub const HID: u8 = 0x21;
    /// HID Report descriptor
    pub const HID_REPORT: u8 = 0x22;
}

/// Control transfer helper
pub struct ControlTransfer<'a> {
    handle: &'a DeviceHandle,
    timeout: Duration,
}

impl<'a> ControlTransfer<'a> {
    /// Create a new control transfer helper
    pub fn new(handle: &'a DeviceHandle) -> Self {
        Self {
            handle,
            timeout: DEFAULT_TIMEOUT,
        }
    }
    
    /// Set the timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    /// Get device descriptor
    pub fn get_device_descriptor(&self) -> Result<Vec<u8>, UsbError> {
        let mut buf = vec![0u8; 18];
        let request_type = request_type::build(
            request_type::DIR_IN,
            request_type::TYPE_STANDARD,
            request_type::RECIP_DEVICE,
        );
        
        let bytes = self.handle.control_read(
            request_type,
            request::GET_DESCRIPTOR,
            (descriptor_type::DEVICE as u16) << 8,
            0,
            &mut buf,
            self.timeout,
        )?;
        
        buf.truncate(bytes);
        Ok(buf)
    }
    
    /// Get configuration descriptor
    pub fn get_configuration_descriptor(&self, index: u8) -> Result<Vec<u8>, UsbError> {
        // First, read just the header to get total length
        let mut header = vec![0u8; 9];
        let request_type = request_type::build(
            request_type::DIR_IN,
            request_type::TYPE_STANDARD,
            request_type::RECIP_DEVICE,
        );
        
        self.handle.control_read(
            request_type,
            request::GET_DESCRIPTOR,
            ((descriptor_type::CONFIGURATION as u16) << 8) | (index as u16),
            0,
            &mut header,
            self.timeout,
        )?;
        
        // Get total length
        let total_length = u16::from_le_bytes([header[2], header[3]]) as usize;
        
        // Read full descriptor
        let mut buf = vec![0u8; total_length];
        let bytes = self.handle.control_read(
            request_type,
            request::GET_DESCRIPTOR,
            ((descriptor_type::CONFIGURATION as u16) << 8) | (index as u16),
            0,
            &mut buf,
            self.timeout,
        )?;
        
        buf.truncate(bytes);
        Ok(buf)
    }
    
    /// Get string descriptor
    pub fn get_string_descriptor(&self, index: u8, lang_id: u16) -> Result<String, UsbError> {
        let mut buf = vec![0u8; 255];
        let request_type = request_type::build(
            request_type::DIR_IN,
            request_type::TYPE_STANDARD,
            request_type::RECIP_DEVICE,
        );
        
        let bytes = self.handle.control_read(
            request_type,
            request::GET_DESCRIPTOR,
            ((descriptor_type::STRING as u16) << 8) | (index as u16),
            lang_id,
            &mut buf,
            self.timeout,
        )?;
        
        if bytes < 2 {
            return Err(UsbError::Parse("String descriptor too short".into()));
        }
        
        // String descriptors are in UTF-16LE
        let chars: Vec<u16> = buf[2..bytes]
            .chunks(2)
            .filter_map(|chunk| {
                if chunk.len() == 2 {
                    Some(u16::from_le_bytes([chunk[0], chunk[1]]))
                } else {
                    None
                }
            })
            .collect();
        
        Ok(String::from_utf16_lossy(&chars))
    }
    
    /// Get supported language IDs
    pub fn get_language_ids(&self) -> Result<Vec<u16>, UsbError> {
        let mut buf = vec![0u8; 255];
        let request_type = request_type::build(
            request_type::DIR_IN,
            request_type::TYPE_STANDARD,
            request_type::RECIP_DEVICE,
        );
        
        let bytes = self.handle.control_read(
            request_type,
            request::GET_DESCRIPTOR,
            (descriptor_type::STRING as u16) << 8,
            0,
            &mut buf,
            self.timeout,
        )?;
        
        if bytes < 4 {
            return Ok(vec![0x0409]); // Default to US English
        }
        
        let lang_ids: Vec<u16> = buf[2..bytes]
            .chunks(2)
            .filter_map(|chunk| {
                if chunk.len() == 2 {
                    Some(u16::from_le_bytes([chunk[0], chunk[1]]))
                } else {
                    None
                }
            })
            .collect();
        
        Ok(lang_ids)
    }
    
    /// Get BOS descriptor
    pub fn get_bos_descriptor(&self) -> Result<Vec<u8>, UsbError> {
        // First, read header to get total length
        let mut header = vec![0u8; 5];
        let request_type = request_type::build(
            request_type::DIR_IN,
            request_type::TYPE_STANDARD,
            request_type::RECIP_DEVICE,
        );
        
        self.handle.control_read(
            request_type,
            request::GET_DESCRIPTOR,
            (descriptor_type::BOS as u16) << 8,
            0,
            &mut header,
            self.timeout,
        )?;
        
        let total_length = u16::from_le_bytes([header[2], header[3]]) as usize;
        
        // Read full descriptor
        let mut buf = vec![0u8; total_length];
        let bytes = self.handle.control_read(
            request_type,
            request::GET_DESCRIPTOR,
            (descriptor_type::BOS as u16) << 8,
            0,
            &mut buf,
            self.timeout,
        )?;
        
        buf.truncate(bytes);
        Ok(buf)
    }
    
    /// Get HID report descriptor
    pub fn get_hid_report_descriptor(&self, interface: u16, length: u16) -> Result<Vec<u8>, UsbError> {
        let mut buf = vec![0u8; length as usize];
        let request_type = request_type::build(
            request_type::DIR_IN,
            request_type::TYPE_STANDARD,
            request_type::RECIP_INTERFACE,
        );
        
        let bytes = self.handle.control_read(
            request_type,
            request::GET_DESCRIPTOR,
            (descriptor_type::HID_REPORT as u16) << 8,
            interface,
            &mut buf,
            self.timeout,
        )?;
        
        buf.truncate(bytes);
        Ok(buf)
    }
    
    /// Set device configuration
    pub fn set_configuration(&self, config: u8) -> Result<(), UsbError> {
        let request_type = request_type::build(
            request_type::DIR_OUT,
            request_type::TYPE_STANDARD,
            request_type::RECIP_DEVICE,
        );
        
        self.handle.control_write(
            request_type,
            request::SET_CONFIGURATION,
            config as u16,
            0,
            &[],
            self.timeout,
        )?;
        
        Ok(())
    }
    
    /// Get device status
    pub fn get_device_status(&self) -> Result<u16, UsbError> {
        let mut buf = [0u8; 2];
        let request_type = request_type::build(
            request_type::DIR_IN,
            request_type::TYPE_STANDARD,
            request_type::RECIP_DEVICE,
        );
        
        self.handle.control_read(
            request_type,
            request::GET_STATUS,
            0,
            0,
            &mut buf,
            self.timeout,
        )?;
        
        Ok(u16::from_le_bytes(buf))
    }
    
    /// Send a vendor-specific control request (IN)
    pub fn vendor_read(
        &self,
        request: u8,
        value: u16,
        index: u16,
        buf: &mut [u8],
    ) -> Result<usize, UsbError> {
        let request_type = request_type::build(
            request_type::DIR_IN,
            request_type::TYPE_VENDOR,
            request_type::RECIP_DEVICE,
        );
        
        self.handle.control_read(request_type, request, value, index, buf, self.timeout)
    }
    
    /// Send a vendor-specific control request (OUT)
    pub fn vendor_write(
        &self,
        request: u8,
        value: u16,
        index: u16,
        buf: &[u8],
    ) -> Result<usize, UsbError> {
        let request_type = request_type::build(
            request_type::DIR_OUT,
            request_type::TYPE_VENDOR,
            request_type::RECIP_DEVICE,
        );
        
        self.handle.control_write(request_type, request, value, index, buf, self.timeout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_type_builder() {
        let rt = request_type::build(
            request_type::DIR_IN,
            request_type::TYPE_STANDARD,
            request_type::RECIP_DEVICE,
        );
        assert_eq!(rt, 0x80);
        
        let rt2 = request_type::build(
            request_type::DIR_OUT,
            request_type::TYPE_VENDOR,
            request_type::RECIP_INTERFACE,
        );
        assert_eq!(rt2, 0x41);
    }
}
