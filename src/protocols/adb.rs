//! ADB (Android Debug Bridge) Protocol Implementation
//!
//! Implements the ADB protocol for communicating with Android devices.

use crate::communication::{DeviceHandle, BulkTransfer};
use crate::errors::UsbError;
use super::UsbProtocol;
use std::time::Duration;

/// ADB protocol constants
pub mod constants {
    /// ADB protocol version
    pub const VERSION: u32 = 0x0100_0000;
    /// Maximum ADB data payload
    pub const MAX_PAYLOAD: u32 = 1024 * 1024; // 1MB
    
    /// ADB command: CNXN (connect)
    pub const CMD_CNXN: u32 = 0x4e58_4e43; // "CNXN" in little-endian
    /// ADB command: OPEN
    pub const CMD_OPEN: u32 = 0x4e45_504f; // "OPEN"
    /// ADB command: OKAY
    pub const CMD_OKAY: u32 = 0x5941_4b4f; // "OKAY"
    /// ADB command: CLSE (close)
    pub const CMD_CLSE: u32 = 0x4553_4c43; // "CLSE"
    /// ADB command: WRTE (write)
    pub const CMD_WRTE: u32 = 0x4554_5257; // "WRTE"
    /// ADB command: AUTH
    pub const CMD_AUTH: u32 = 0x4854_5541; // "AUTH"
    /// ADB command: STLS (TLS)
    pub const CMD_STLS: u32 = 0x534c_5453; // "STLS"
    
    /// AUTH type: Token
    pub const AUTH_TOKEN: u32 = 1;
    /// AUTH type: Signature
    pub const AUTH_SIGNATURE: u32 = 2;
    /// AUTH type: RSA Public Key
    pub const AUTH_RSAPUBLICKEY: u32 = 3;
}

/// ADB message header (24 bytes)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct AdbMessage {
    /// Command identifier
    pub command: u32,
    /// First argument
    pub arg0: u32,
    /// Second argument
    pub arg1: u32,
    /// Data payload length
    pub data_length: u32,
    /// CRC32 of data
    pub data_crc32: u32,
    /// Magic (command XOR 0xFFFFFFFF)
    pub magic: u32,
}

impl AdbMessage {
    /// Create a new ADB message
    pub fn new(command: u32, arg0: u32, arg1: u32, data: &[u8]) -> Self {
        Self {
            command,
            arg0,
            arg1,
            data_length: data.len() as u32,
            data_crc32: Self::crc32(data),
            magic: command ^ 0xFFFF_FFFF,
        }
    }
    
    /// Create a CNXN message
    pub fn connect(system_identity: &str) -> (Self, Vec<u8>) {
        let data = format!("host::{}\0", system_identity).into_bytes();
        (
            Self::new(constants::CMD_CNXN, constants::VERSION, constants::MAX_PAYLOAD, &data),
            data,
        )
    }
    
    /// Create an OPEN message
    pub fn open(local_id: u32, destination: &str) -> (Self, Vec<u8>) {
        let data = format!("{}\0", destination).into_bytes();
        (
            Self::new(constants::CMD_OPEN, local_id, 0, &data),
            data,
        )
    }
    
    /// Create a WRTE message
    pub fn write(local_id: u32, remote_id: u32, data: &[u8]) -> Self {
        Self::new(constants::CMD_WRTE, local_id, remote_id, data)
    }
    
    /// Create an OKAY message
    pub fn okay(local_id: u32, remote_id: u32) -> Self {
        Self::new(constants::CMD_OKAY, local_id, remote_id, &[])
    }
    
    /// Create a CLSE message
    pub fn close(local_id: u32, remote_id: u32) -> Self {
        Self::new(constants::CMD_CLSE, local_id, remote_id, &[])
    }
    
    /// Calculate CRC32 (ADB uses simple sum)
    fn crc32(data: &[u8]) -> u32 {
        data.iter().map(|&b| b as u32).sum()
    }
    
    /// Serialize to bytes
    pub fn to_bytes(&self) -> [u8; 24] {
        let mut buf = [0u8; 24];
        buf[0..4].copy_from_slice(&self.command.to_le_bytes());
        buf[4..8].copy_from_slice(&self.arg0.to_le_bytes());
        buf[8..12].copy_from_slice(&self.arg1.to_le_bytes());
        buf[12..16].copy_from_slice(&self.data_length.to_le_bytes());
        buf[16..20].copy_from_slice(&self.data_crc32.to_le_bytes());
        buf[20..24].copy_from_slice(&self.magic.to_le_bytes());
        buf
    }
    
    /// Parse from bytes
    pub fn from_bytes(buf: &[u8]) -> Result<Self, &'static str> {
        if buf.len() < 24 {
            return Err("Buffer too short for ADB message");
        }
        
        let command = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let magic = u32::from_le_bytes([buf[20], buf[21], buf[22], buf[23]]);
        
        // Validate magic
        if command ^ 0xFFFF_FFFF != magic {
            return Err("Invalid ADB message magic");
        }
        
        Ok(Self {
            command,
            arg0: u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]),
            arg1: u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]),
            data_length: u32::from_le_bytes([buf[12], buf[13], buf[14], buf[15]]),
            data_crc32: u32::from_le_bytes([buf[16], buf[17], buf[18], buf[19]]),
            magic,
        })
    }
    
    /// Get command name
    pub fn command_name(&self) -> &'static str {
        match self.command {
            constants::CMD_CNXN => "CNXN",
            constants::CMD_OPEN => "OPEN",
            constants::CMD_OKAY => "OKAY",
            constants::CMD_CLSE => "CLSE",
            constants::CMD_WRTE => "WRTE",
            constants::CMD_AUTH => "AUTH",
            constants::CMD_STLS => "STLS",
            _ => "UNKNOWN",
        }
    }
}

/// ADB connection state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdbState {
    /// Not connected
    Disconnected,
    /// Waiting for connection response
    Connecting,
    /// Waiting for authentication
    Authenticating,
    /// Connected and ready
    Connected,
    /// Error state
    Error(String),
}

/// ADB stream state
#[derive(Debug, Clone)]
pub struct AdbStream {
    /// Local stream ID
    pub local_id: u32,
    /// Remote stream ID
    pub remote_id: u32,
    /// Stream destination (e.g., "shell:", "sync:")
    pub destination: String,
    /// Whether the stream is open
    pub is_open: bool,
}

/// ADB device client
pub struct AdbClient<'a> {
    handle: &'a DeviceHandle,
    ep_in: u8,
    ep_out: u8,
    state: AdbState,
    device_banner: Option<String>,
    next_local_id: u32,
    streams: Vec<AdbStream>,
    max_payload: u32,
    timeout: Duration,
}

impl<'a> AdbClient<'a> {
    /// ADB interface class
    pub const CLASS: u8 = 0xFF;
    /// ADB interface subclass
    pub const SUBCLASS: u8 = 0x42;
    /// ADB interface protocol
    pub const PROTOCOL: u8 = 0x01;
    
    /// Create a new ADB client
    pub fn new(handle: &'a DeviceHandle, ep_in: u8, ep_out: u8) -> Self {
        Self {
            handle,
            ep_in: ep_in | 0x80,
            ep_out: ep_out & 0x7F,
            state: AdbState::Disconnected,
            device_banner: None,
            next_local_id: 1,
            streams: Vec::new(),
            max_payload: 4096,
            timeout: Duration::from_secs(5),
        }
    }
    
    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    /// Get connection state
    pub fn state(&self) -> &AdbState {
        &self.state
    }
    
    /// Get device banner (product info)
    pub fn device_banner(&self) -> Option<&str> {
        self.device_banner.as_deref()
    }
    
    /// Connect to the ADB device
    pub fn connect(&mut self, system_identity: &str) -> Result<(), UsbError> {
        self.state = AdbState::Connecting;
        
        // Send CNXN message
        let (msg, data) = AdbMessage::connect(system_identity);
        self.send_message(&msg, &data)?;
        
        // Receive response
        let (response, response_data) = self.recv_message()?;
        
        match response.command {
            constants::CMD_CNXN => {
                // Connected!
                self.max_payload = response.arg1;
                self.device_banner = Some(
                    String::from_utf8_lossy(&response_data).trim_end_matches('\0').to_string()
                );
                self.state = AdbState::Connected;
                Ok(())
            }
            constants::CMD_AUTH => {
                // Authentication required
                self.state = AdbState::Authenticating;
                Err(UsbError::Unknown("ADB authentication required".into()))
            }
            _ => {
                self.state = AdbState::Error(format!("Unexpected response: {}", response.command_name()));
                Err(UsbError::Unknown(format!("Unexpected ADB response: {}", response.command_name())))
            }
        }
    }
    
    /// Open a stream (shell, sync, etc.)
    pub fn open_stream(&mut self, destination: &str) -> Result<u32, UsbError> {
        if self.state != AdbState::Connected {
            return Err(UsbError::Unknown("Not connected".into()));
        }
        
        let local_id = self.next_local_id;
        self.next_local_id += 1;
        
        let (msg, data) = AdbMessage::open(local_id, destination);
        self.send_message(&msg, &data)?;
        
        // Wait for OKAY or CLSE
        let (response, _) = self.recv_message()?;
        
        match response.command {
            constants::CMD_OKAY => {
                self.streams.push(AdbStream {
                    local_id,
                    remote_id: response.arg0,
                    destination: destination.to_string(),
                    is_open: true,
                });
                Ok(local_id)
            }
            constants::CMD_CLSE => {
                Err(UsbError::Unknown(format!("Stream rejected: {}", destination)))
            }
            _ => {
                Err(UsbError::Unknown(format!("Unexpected response: {}", response.command_name())))
            }
        }
    }
    
    /// Write to a stream
    pub fn write_stream(&mut self, local_id: u32, data: &[u8]) -> Result<(), UsbError> {
        let stream = self.streams.iter()
            .find(|s| s.local_id == local_id && s.is_open)
            .ok_or_else(|| UsbError::Unknown("Stream not found".into()))?;
        
        let msg = AdbMessage::write(local_id, stream.remote_id, data);
        self.send_message(&msg, data)?;
        
        // Wait for OKAY
        let (response, _) = self.recv_message()?;
        
        if response.command == constants::CMD_OKAY {
            Ok(())
        } else {
            Err(UsbError::Unknown(format!("Write failed: {}", response.command_name())))
        }
    }
    
    /// Read from a stream
    pub fn read_stream(&mut self, local_id: u32) -> Result<Vec<u8>, UsbError> {
        let stream = self.streams.iter()
            .find(|s| s.local_id == local_id && s.is_open)
            .ok_or_else(|| UsbError::Unknown("Stream not found".into()))?;
        
        let (response, data) = self.recv_message()?;
        
        match response.command {
            constants::CMD_WRTE => {
                // Send OKAY acknowledgment
                let okay = AdbMessage::okay(local_id, stream.remote_id);
                self.send_message(&okay, &[])?;
                Ok(data)
            }
            constants::CMD_CLSE => {
                // Stream closed by remote
                if let Some(s) = self.streams.iter_mut().find(|s| s.local_id == local_id) {
                    s.is_open = false;
                }
                Err(UsbError::Unknown("Stream closed".into()))
            }
            _ => {
                Err(UsbError::Unknown(format!("Unexpected: {}", response.command_name())))
            }
        }
    }
    
    /// Close a stream
    pub fn close_stream(&mut self, local_id: u32) -> Result<(), UsbError> {
        // Find the remote_id and current state first
        let stream_info = self.streams.iter()
            .find(|s| s.local_id == local_id)
            .map(|s| (s.remote_id, s.is_open));
        
        if let Some((remote_id, is_open)) = stream_info {
            if is_open {
                let msg = AdbMessage::close(local_id, remote_id);
                self.send_message(&msg, &[])?;
                
                // Now update the stream
                if let Some(stream) = self.streams.iter_mut().find(|s| s.local_id == local_id) {
                    stream.is_open = false;
                }
            }
        }
        Ok(())
    }
    
    /// Execute a shell command
    pub fn shell(&mut self, command: &str) -> Result<String, UsbError> {
        let stream_id = self.open_stream(&format!("shell:{}", command))?;
        
        let mut output = Vec::new();
        while let Ok(data) = self.read_stream(stream_id) {
            output.extend_from_slice(&data);
        }
        
        Ok(String::from_utf8_lossy(&output).to_string())
    }
    
    /// Send an ADB message
    fn send_message(&self, msg: &AdbMessage, data: &[u8]) -> Result<(), UsbError> {
        let bulk = BulkTransfer::new(self.handle).with_timeout(self.timeout);
        
        // Send header
        bulk.write(self.ep_out, &msg.to_bytes())?;
        
        // Send data if present
        if !data.is_empty() {
            bulk.write(self.ep_out, data)?;
        }
        
        Ok(())
    }
    
    /// Receive an ADB message
    fn recv_message(&self) -> Result<(AdbMessage, Vec<u8>), UsbError> {
        let bulk = BulkTransfer::new(self.handle).with_timeout(self.timeout);
        
        // Receive header
        let mut header = [0u8; 24];
        bulk.read(self.ep_in, &mut header)?;
        
        let msg = AdbMessage::from_bytes(&header)
            .map_err(|e| UsbError::Parse(e.to_string()))?;
        
        // Receive data if present
        let data = if msg.data_length > 0 {
            let mut buf = vec![0u8; msg.data_length as usize];
            bulk.read(self.ep_in, &mut buf)?;
            buf
        } else {
            Vec::new()
        };
        
        Ok((msg, data))
    }
}

impl UsbProtocol for AdbClient<'_> {
    fn name(&self) -> &'static str {
        "ADB"
    }
    
    fn is_connected(&self) -> bool {
        self.state == AdbState::Connected
    }
    
    fn version(&self) -> Option<String> {
        Some(format!("0x{:08X}", constants::VERSION))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adb_message() {
        let msg = AdbMessage::new(constants::CMD_CNXN, 0x01000000, 4096, b"test");
        assert_eq!(msg.command_name(), "CNXN");
        assert_eq!(msg.data_length, 4);
        
        let bytes = msg.to_bytes();
        let parsed = AdbMessage::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.command, msg.command);
    }

    #[test]
    fn test_adb_connect_message() {
        let (msg, data) = AdbMessage::connect("host::test");
        assert_eq!(msg.command, constants::CMD_CNXN);
        assert!(data.starts_with(b"host::"));
    }
}
