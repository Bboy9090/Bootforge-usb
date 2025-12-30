//! MTP (Media Transfer Protocol) Implementation
//!
//! Implements the MTP protocol for file transfer with media devices.

use crate::communication::{DeviceHandle, BulkTransfer};
use crate::errors::UsbError;
use super::UsbProtocol;
use std::time::Duration;

/// MTP operation codes
pub mod operation {
    /// Get device info
    pub const GET_DEVICE_INFO: u16 = 0x1001;
    /// Open session
    pub const OPEN_SESSION: u16 = 0x1002;
    /// Close session
    pub const CLOSE_SESSION: u16 = 0x1003;
    /// Get storage IDs
    pub const GET_STORAGE_IDS: u16 = 0x1004;
    /// Get storage info
    pub const GET_STORAGE_INFO: u16 = 0x1005;
    /// Get number of objects
    pub const GET_NUM_OBJECTS: u16 = 0x1006;
    /// Get object handles
    pub const GET_OBJECT_HANDLES: u16 = 0x1007;
    /// Get object info
    pub const GET_OBJECT_INFO: u16 = 0x1008;
    /// Get object
    pub const GET_OBJECT: u16 = 0x1009;
    /// Get thumbnail
    pub const GET_THUMB: u16 = 0x100A;
    /// Delete object
    pub const DELETE_OBJECT: u16 = 0x100B;
    /// Send object info
    pub const SEND_OBJECT_INFO: u16 = 0x100C;
    /// Send object
    pub const SEND_OBJECT: u16 = 0x100D;
    /// Format store
    pub const FORMAT_STORE: u16 = 0x100F;
    /// Reset device
    pub const RESET_DEVICE: u16 = 0x1010;
    /// Get device property description
    pub const GET_DEVICE_PROP_DESC: u16 = 0x1014;
    /// Get device property value
    pub const GET_DEVICE_PROP_VALUE: u16 = 0x1015;
    /// Set device property value
    pub const SET_DEVICE_PROP_VALUE: u16 = 0x1016;
    /// Move object
    pub const MOVE_OBJECT: u16 = 0x1019;
    /// Copy object
    pub const COPY_OBJECT: u16 = 0x101A;
    /// Get partial object
    pub const GET_PARTIAL_OBJECT: u16 = 0x101B;
}

/// MTP response codes
pub mod response {
    /// OK
    pub const OK: u16 = 0x2001;
    /// General error
    pub const GENERAL_ERROR: u16 = 0x2002;
    /// Session not open
    pub const SESSION_NOT_OPEN: u16 = 0x2003;
    /// Invalid transaction ID
    pub const INVALID_TRANSACTION_ID: u16 = 0x2004;
    /// Operation not supported
    pub const OPERATION_NOT_SUPPORTED: u16 = 0x2005;
    /// Parameter not supported
    pub const PARAMETER_NOT_SUPPORTED: u16 = 0x2006;
    /// Incomplete transfer
    pub const INCOMPLETE_TRANSFER: u16 = 0x2007;
    /// Invalid storage ID
    pub const INVALID_STORAGE_ID: u16 = 0x2008;
    /// Invalid object handle
    pub const INVALID_OBJECT_HANDLE: u16 = 0x2009;
    /// Store full
    pub const STORE_FULL: u16 = 0x200C;
    /// Object write protected
    pub const OBJECT_WRITE_PROTECTED: u16 = 0x200D;
    /// Store read only
    pub const STORE_READ_ONLY: u16 = 0x200E;
    /// Access denied
    pub const ACCESS_DENIED: u16 = 0x200F;
    /// Invalid parent
    pub const INVALID_PARENT: u16 = 0x201A;
    /// Invalid parameter
    pub const INVALID_PARAMETER: u16 = 0x201D;
    /// Session already open
    pub const SESSION_ALREADY_OPEN: u16 = 0x201E;
}

/// MTP container types
pub mod container_type {
    /// Command block
    pub const COMMAND: u16 = 1;
    /// Data block
    pub const DATA: u16 = 2;
    /// Response block
    pub const RESPONSE: u16 = 3;
    /// Event block
    pub const EVENT: u16 = 4;
}

/// MTP container header (12 bytes minimum)
#[derive(Debug, Clone)]
pub struct MtpContainer {
    /// Container length
    pub length: u32,
    /// Container type
    pub container_type: u16,
    /// Operation code or response code
    pub code: u16,
    /// Transaction ID
    pub transaction_id: u32,
    /// Parameters (up to 5)
    pub parameters: Vec<u32>,
    /// Payload data
    pub payload: Vec<u8>,
}

impl MtpContainer {
    /// Create a command container
    pub fn command(code: u16, transaction_id: u32, parameters: Vec<u32>) -> Self {
        let length = 12 + (parameters.len() * 4) as u32;
        Self {
            length,
            container_type: container_type::COMMAND,
            code,
            transaction_id,
            parameters,
            payload: Vec::new(),
        }
    }
    
    /// Create a data container
    pub fn data(code: u16, transaction_id: u32, payload: Vec<u8>) -> Self {
        let length = 12 + payload.len() as u32;
        Self {
            length,
            container_type: container_type::DATA,
            code,
            transaction_id,
            parameters: Vec::new(),
            payload,
        }
    }
    
    /// Serialize to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.length as usize);
        
        buf.extend_from_slice(&self.length.to_le_bytes());
        buf.extend_from_slice(&self.container_type.to_le_bytes());
        buf.extend_from_slice(&self.code.to_le_bytes());
        buf.extend_from_slice(&self.transaction_id.to_le_bytes());
        
        for param in &self.parameters {
            buf.extend_from_slice(&param.to_le_bytes());
        }
        
        buf.extend_from_slice(&self.payload);
        
        buf
    }
    
    /// Parse from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() < 12 {
            return Err("Container too short");
        }
        
        let length = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let container_type = u16::from_le_bytes([data[4], data[5]]);
        let code = u16::from_le_bytes([data[6], data[7]]);
        let transaction_id = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        
        // Parse parameters (for response containers)
        let mut parameters = Vec::new();
        let mut offset = 12;
        
        if container_type == container_type::RESPONSE || container_type == container_type::COMMAND {
            while offset + 4 <= data.len() && offset + 4 <= length as usize && parameters.len() < 5 {
                let param = u32::from_le_bytes([
                    data[offset],
                    data[offset + 1],
                    data[offset + 2],
                    data[offset + 3],
                ]);
                parameters.push(param);
                offset += 4;
            }
        }
        
        // Rest is payload
        let payload = if offset < data.len() {
            data[offset..].to_vec()
        } else {
            Vec::new()
        };
        
        Ok(Self {
            length,
            container_type,
            code,
            transaction_id,
            parameters,
            payload,
        })
    }
    
    /// Check if this is an OK response
    pub fn is_ok(&self) -> bool {
        self.container_type == container_type::RESPONSE && self.code == response::OK
    }
}

/// MTP storage information
#[derive(Debug, Clone)]
pub struct MtpStorageInfo {
    /// Storage ID
    pub storage_id: u32,
    /// Storage type (Fixed ROM, Removable ROM, Fixed RAM, Removable RAM)
    pub storage_type: u16,
    /// Filesystem type
    pub filesystem_type: u16,
    /// Access capability (read-write, read-only, read-only with delete)
    pub access_capability: u16,
    /// Maximum capacity in bytes
    pub max_capacity: u64,
    /// Free space in bytes
    pub free_space: u64,
    /// Free space in objects
    pub free_space_objects: u32,
    /// Storage description
    pub description: String,
    /// Volume label
    pub volume_label: String,
}

/// MTP object information
#[derive(Debug, Clone)]
pub struct MtpObjectInfo {
    /// Object handle
    pub handle: u32,
    /// Storage ID
    pub storage_id: u32,
    /// Object format
    pub format: u16,
    /// Protection status
    pub protection_status: u16,
    /// Object compressed size
    pub compressed_size: u32,
    /// Thumbnail format
    pub thumb_format: u16,
    /// Thumbnail compressed size
    pub thumb_compressed_size: u32,
    /// Thumbnail width
    pub thumb_width: u32,
    /// Thumbnail height
    pub thumb_height: u32,
    /// Image width
    pub image_width: u32,
    /// Image height
    pub image_height: u32,
    /// Image bit depth
    pub image_bit_depth: u32,
    /// Parent object handle
    pub parent: u32,
    /// Association type
    pub association_type: u16,
    /// Association description
    pub association_desc: u32,
    /// Sequence number
    pub sequence_number: u32,
    /// Filename
    pub filename: String,
    /// Date created
    pub date_created: String,
    /// Date modified
    pub date_modified: String,
    /// Keywords
    pub keywords: String,
}

/// MTP device info
#[derive(Debug, Clone)]
pub struct MtpDeviceInfo {
    /// Standard version
    pub standard_version: u16,
    /// MTP vendor extension ID
    pub vendor_extension_id: u32,
    /// MTP version
    pub mtp_version: u16,
    /// MTP extensions
    pub extensions: String,
    /// Functional mode
    pub functional_mode: u16,
    /// Supported operations
    pub operations_supported: Vec<u16>,
    /// Supported events
    pub events_supported: Vec<u16>,
    /// Supported device properties
    pub device_properties_supported: Vec<u16>,
    /// Supported capture formats
    pub capture_formats: Vec<u16>,
    /// Supported playback formats
    pub playback_formats: Vec<u16>,
    /// Manufacturer
    pub manufacturer: String,
    /// Model
    pub model: String,
    /// Device version
    pub device_version: String,
    /// Serial number
    pub serial_number: String,
}

/// MTP client
pub struct MtpClient<'a> {
    handle: &'a DeviceHandle,
    ep_in: u8,
    ep_out: u8,
    #[allow(dead_code)]
    ep_int: u8, // For interrupt endpoint (events)
    session_id: u32,
    transaction_id: u32,
    timeout: Duration,
}

impl<'a> MtpClient<'a> {
    /// MTP interface class (Still Image)
    pub const CLASS: u8 = 0x06;
    /// MTP interface subclass
    pub const SUBCLASS: u8 = 0x01;
    /// MTP interface protocol
    pub const PROTOCOL: u8 = 0x01;
    
    /// Create a new MTP client
    pub fn new(handle: &'a DeviceHandle, ep_in: u8, ep_out: u8, ep_int: u8) -> Self {
        Self {
            handle,
            ep_in: ep_in | 0x80,
            ep_out: ep_out & 0x7F,
            ep_int: ep_int | 0x80,
            session_id: 0,
            transaction_id: 0,
            timeout: Duration::from_secs(5),
        }
    }
    
    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    /// Get next transaction ID
    fn next_transaction_id(&mut self) -> u32 {
        self.transaction_id += 1;
        self.transaction_id
    }
    
    /// Send a command
    fn send_command(&mut self, code: u16, parameters: Vec<u32>) -> Result<(), UsbError> {
        let tx_id = self.next_transaction_id();
        let container = MtpContainer::command(code, tx_id, parameters);
        
        let bulk = BulkTransfer::new(self.handle).with_timeout(self.timeout);
        bulk.write(self.ep_out, &container.to_bytes())?;
        
        Ok(())
    }
    
    /// Read response
    fn read_response(&self) -> Result<MtpContainer, UsbError> {
        let bulk = BulkTransfer::new(self.handle).with_timeout(self.timeout);
        
        let mut buf = vec![0u8; 512];
        let bytes = bulk.read(self.ep_in, &mut buf)?;
        
        MtpContainer::from_bytes(&buf[..bytes])
            .map_err(|e| UsbError::Parse(e.to_string()))
    }
    
    /// Read data phase
    fn read_data(&self) -> Result<Vec<u8>, UsbError> {
        let bulk = BulkTransfer::new(self.handle).with_timeout(self.timeout);
        
        // First read to get container header and initial data
        let mut buf = vec![0u8; 64 * 1024];
        let bytes = bulk.read(self.ep_in, &mut buf)?;
        
        let container = MtpContainer::from_bytes(&buf[..bytes])
            .map_err(|e| UsbError::Parse(e.to_string()))?;
        
        if container.container_type != container_type::DATA {
            return Err(UsbError::Unknown("Expected data container".into()));
        }
        
        // If we need more data, keep reading
        let total_length = container.length as usize;
        let mut data = buf[12..bytes].to_vec();
        
        while data.len() + 12 < total_length {
            let bytes = bulk.read(self.ep_in, &mut buf)?;
            data.extend_from_slice(&buf[..bytes]);
        }
        
        Ok(data)
    }
    
    /// Open a session
    pub fn open_session(&mut self) -> Result<(), UsbError> {
        self.session_id += 1;
        self.transaction_id = 0;
        
        self.send_command(operation::OPEN_SESSION, vec![self.session_id])?;
        
        let response = self.read_response()?;
        
        if response.is_ok() || response.code == response::SESSION_ALREADY_OPEN {
            Ok(())
        } else {
            Err(UsbError::Unknown(format!("Open session failed: 0x{:04X}", response.code)))
        }
    }
    
    /// Close the session
    pub fn close_session(&mut self) -> Result<(), UsbError> {
        self.send_command(operation::CLOSE_SESSION, vec![])?;
        
        let response = self.read_response()?;
        
        if response.is_ok() {
            self.session_id = 0;
            Ok(())
        } else {
            Err(UsbError::Unknown(format!("Close session failed: 0x{:04X}", response.code)))
        }
    }
    
    /// Get device info
    pub fn get_device_info(&mut self) -> Result<MtpDeviceInfo, UsbError> {
        self.send_command(operation::GET_DEVICE_INFO, vec![])?;
        
        let data = self.read_data()?;
        let response = self.read_response()?;
        
        if !response.is_ok() {
            return Err(UsbError::Unknown("Get device info failed".into()));
        }
        
        // Parse device info from data
        // This is a simplified parse - full parse would need MTP string handling
        Ok(MtpDeviceInfo {
            standard_version: if data.len() >= 2 {
                u16::from_le_bytes([data[0], data[1]])
            } else { 0 },
            vendor_extension_id: if data.len() >= 6 {
                u32::from_le_bytes([data[2], data[3], data[4], data[5]])
            } else { 0 },
            mtp_version: if data.len() >= 8 {
                u16::from_le_bytes([data[6], data[7]])
            } else { 0 },
            extensions: String::new(),
            functional_mode: 0,
            operations_supported: Vec::new(),
            events_supported: Vec::new(),
            device_properties_supported: Vec::new(),
            capture_formats: Vec::new(),
            playback_formats: Vec::new(),
            manufacturer: String::new(),
            model: String::new(),
            device_version: String::new(),
            serial_number: String::new(),
        })
    }
    
    /// Get storage IDs
    pub fn get_storage_ids(&mut self) -> Result<Vec<u32>, UsbError> {
        self.send_command(operation::GET_STORAGE_IDS, vec![])?;
        
        let data = self.read_data()?;
        let response = self.read_response()?;
        
        if !response.is_ok() {
            return Err(UsbError::Unknown("Get storage IDs failed".into()));
        }
        
        // Parse array of storage IDs
        if data.len() < 4 {
            return Ok(Vec::new());
        }
        
        let count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        let mut ids = Vec::with_capacity(count);
        
        for i in 0..count {
            let offset = 4 + i * 4;
            if offset + 4 <= data.len() {
                ids.push(u32::from_le_bytes([
                    data[offset],
                    data[offset + 1],
                    data[offset + 2],
                    data[offset + 3],
                ]));
            }
        }
        
        Ok(ids)
    }
    
    /// Get object handles
    pub fn get_object_handles(&mut self, storage_id: u32, parent: u32) -> Result<Vec<u32>, UsbError> {
        self.send_command(operation::GET_OBJECT_HANDLES, vec![storage_id, 0, parent])?;
        
        let data = self.read_data()?;
        let response = self.read_response()?;
        
        if !response.is_ok() {
            return Err(UsbError::Unknown("Get object handles failed".into()));
        }
        
        // Parse array of object handles
        if data.len() < 4 {
            return Ok(Vec::new());
        }
        
        let count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        let mut handles = Vec::with_capacity(count);
        
        for i in 0..count {
            let offset = 4 + i * 4;
            if offset + 4 <= data.len() {
                handles.push(u32::from_le_bytes([
                    data[offset],
                    data[offset + 1],
                    data[offset + 2],
                    data[offset + 3],
                ]));
            }
        }
        
        Ok(handles)
    }
    
    /// Get object (download file)
    pub fn get_object(&mut self, handle: u32) -> Result<Vec<u8>, UsbError> {
        self.send_command(operation::GET_OBJECT, vec![handle])?;
        
        let data = self.read_data()?;
        let response = self.read_response()?;
        
        if response.is_ok() {
            Ok(data)
        } else {
            Err(UsbError::Unknown("Get object failed".into()))
        }
    }
    
    /// Delete object
    pub fn delete_object(&mut self, handle: u32) -> Result<(), UsbError> {
        self.send_command(operation::DELETE_OBJECT, vec![handle])?;
        
        let response = self.read_response()?;
        
        if response.is_ok() {
            Ok(())
        } else {
            Err(UsbError::Unknown("Delete object failed".into()))
        }
    }
}

impl UsbProtocol for MtpClient<'_> {
    fn name(&self) -> &'static str {
        "MTP"
    }
    
    fn is_connected(&self) -> bool {
        self.session_id > 0
    }
    
    fn version(&self) -> Option<String> {
        Some("1.0".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_serialization() {
        let container = MtpContainer::command(operation::OPEN_SESSION, 1, vec![1]);
        let bytes = container.to_bytes();
        
        assert_eq!(bytes.len(), 16); // 12 + 4 (one parameter)
        
        let parsed = MtpContainer::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.code, operation::OPEN_SESSION);
        assert_eq!(parsed.transaction_id, 1);
    }

    #[test]
    fn test_response_codes() {
        assert_eq!(response::OK, 0x2001);
        assert_eq!(response::SESSION_NOT_OPEN, 0x2003);
    }
}
