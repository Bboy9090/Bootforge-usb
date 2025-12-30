//! CDC (Communications Device Class) Implementation
//!
//! Provides support for USB serial/modem devices (ACM, ECM, NCM).

use crate::communication::{DeviceHandle, BulkTransfer};
use crate::errors::UsbError;
use super::UsbProtocol;
use std::time::Duration;

/// CDC subclass codes
pub mod subclass {
    /// Direct Line Control Model
    pub const DLCM: u8 = 0x01;
    /// Abstract Control Model (modem/serial)
    pub const ACM: u8 = 0x02;
    /// Telephone Control Model
    pub const TCM: u8 = 0x03;
    /// Multi-Channel Control Model
    pub const MCCM: u8 = 0x04;
    /// CAPI Control Model
    pub const CAPI: u8 = 0x05;
    /// Ethernet Networking Control Model
    pub const ECM: u8 = 0x06;
    /// ATM Networking Control Model
    pub const ATM: u8 = 0x07;
    /// Wireless Handset Control Model
    pub const WHCM: u8 = 0x08;
    /// Device Management
    pub const DM: u8 = 0x09;
    /// Mobile Direct Line Model
    pub const MDLM: u8 = 0x0A;
    /// OBEX
    pub const OBEX: u8 = 0x0B;
    /// Ethernet Emulation Model
    pub const EEM: u8 = 0x0C;
    /// Network Control Model
    pub const NCM: u8 = 0x0D;
}

/// CDC class requests
pub mod request {
    /// Send encapsulated command
    pub const SEND_ENCAPSULATED_COMMAND: u8 = 0x00;
    /// Get encapsulated response
    pub const GET_ENCAPSULATED_RESPONSE: u8 = 0x01;
    /// Set comm feature
    pub const SET_COMM_FEATURE: u8 = 0x02;
    /// Get comm feature
    pub const GET_COMM_FEATURE: u8 = 0x03;
    /// Clear comm feature
    pub const CLEAR_COMM_FEATURE: u8 = 0x04;
    /// Set line coding
    pub const SET_LINE_CODING: u8 = 0x20;
    /// Get line coding
    pub const GET_LINE_CODING: u8 = 0x21;
    /// Set control line state
    pub const SET_CONTROL_LINE_STATE: u8 = 0x22;
    /// Send break
    pub const SEND_BREAK: u8 = 0x23;
}

/// CDC notifications
pub mod notification {
    /// Network connection
    pub const NETWORK_CONNECTION: u8 = 0x00;
    /// Response available
    pub const RESPONSE_AVAILABLE: u8 = 0x01;
    /// Serial state
    pub const SERIAL_STATE: u8 = 0x20;
}

/// Line coding structure (7 bytes)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineCoding {
    /// Baud rate
    pub baud_rate: u32,
    /// Stop bits: 0=1, 1=1.5, 2=2
    pub stop_bits: u8,
    /// Parity: 0=None, 1=Odd, 2=Even, 3=Mark, 4=Space
    pub parity: u8,
    /// Data bits: 5, 6, 7, 8, 16
    pub data_bits: u8,
}

impl LineCoding {
    /// Create a new line coding with common defaults
    pub fn new(baud_rate: u32) -> Self {
        Self {
            baud_rate,
            stop_bits: 0,  // 1 stop bit
            parity: 0,     // No parity
            data_bits: 8,  // 8 data bits
        }
    }
    
    /// Create with full parameters
    pub fn with_params(baud_rate: u32, data_bits: u8, parity: u8, stop_bits: u8) -> Self {
        Self {
            baud_rate,
            stop_bits,
            parity,
            data_bits,
        }
    }
    
    /// Common baud rates
    pub const BAUD_9600: u32 = 9600;
    pub const BAUD_19200: u32 = 19200;
    pub const BAUD_38400: u32 = 38400;
    pub const BAUD_57600: u32 = 57600;
    pub const BAUD_115200: u32 = 115200;
    pub const BAUD_230400: u32 = 230400;
    pub const BAUD_460800: u32 = 460800;
    pub const BAUD_921600: u32 = 921600;
    
    /// Serialize to bytes
    pub fn to_bytes(&self) -> [u8; 7] {
        let baud = self.baud_rate.to_le_bytes();
        [baud[0], baud[1], baud[2], baud[3], self.stop_bits, self.parity, self.data_bits]
    }
    
    /// Parse from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() < 7 {
            return Err("Line coding data too short");
        }
        
        Ok(Self {
            baud_rate: u32::from_le_bytes([data[0], data[1], data[2], data[3]]),
            stop_bits: data[4],
            parity: data[5],
            data_bits: data[6],
        })
    }
    
    /// Get stop bits description
    pub fn stop_bits_name(&self) -> &'static str {
        match self.stop_bits {
            0 => "1",
            1 => "1.5",
            2 => "2",
            _ => "?",
        }
    }
    
    /// Get parity description
    pub fn parity_name(&self) -> &'static str {
        match self.parity {
            0 => "None",
            1 => "Odd",
            2 => "Even",
            3 => "Mark",
            4 => "Space",
            _ => "?",
        }
    }
}

impl Default for LineCoding {
    fn default() -> Self {
        Self::new(Self::BAUD_115200)
    }
}

/// Control line state bits
pub mod control_line {
    /// DTR (Data Terminal Ready)
    pub const DTR: u16 = 0x0001;
    /// RTS (Request To Send)
    pub const RTS: u16 = 0x0002;
}

/// Serial state bits (from notification)
pub mod serial_state {
    /// DCD (Data Carrier Detect)
    pub const DCD: u16 = 0x0001;
    /// DSR (Data Set Ready)
    pub const DSR: u16 = 0x0002;
    /// Break
    pub const BREAK: u16 = 0x0004;
    /// Ring signal
    pub const RING: u16 = 0x0008;
    /// Framing error
    pub const FRAMING_ERROR: u16 = 0x0010;
    /// Parity error
    pub const PARITY_ERROR: u16 = 0x0020;
    /// Overrun error
    pub const OVERRUN: u16 = 0x0040;
}

/// CDC ACM (Abstract Control Model) client - USB Serial
pub struct CdcAcmClient<'a> {
    handle: &'a DeviceHandle,
    control_interface: u8,
    data_interface: u8,
    ep_in: u8,
    ep_out: u8,
    ep_notify: Option<u8>,
    line_coding: LineCoding,
    control_line_state: u16,
    timeout: Duration,
    connected: bool,
}

impl<'a> CdcAcmClient<'a> {
    /// CDC class code
    pub const CLASS: u8 = 0x02;
    /// CDC Data class code
    pub const DATA_CLASS: u8 = 0x0A;
    
    /// Create a new CDC ACM client
    pub fn new(
        handle: &'a DeviceHandle,
        control_interface: u8,
        data_interface: u8,
        ep_in: u8,
        ep_out: u8,
    ) -> Self {
        Self {
            handle,
            control_interface,
            data_interface,
            ep_in: ep_in | 0x80,
            ep_out: ep_out & 0x7F,
            ep_notify: None,
            line_coding: LineCoding::default(),
            control_line_state: 0,
            timeout: Duration::from_secs(1),
            connected: false,
        }
    }
    
    /// Set notification endpoint
    pub fn with_notify_endpoint(mut self, ep: u8) -> Self {
        self.ep_notify = Some(ep | 0x80);
        self
    }
    
    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    /// Open the serial port
    pub fn open(&mut self, baud_rate: u32) -> Result<(), UsbError> {
        // Set line coding
        self.line_coding = LineCoding::new(baud_rate);
        self.set_line_coding(&self.line_coding.clone())?;
        
        // Enable DTR and RTS
        self.set_control_line_state(control_line::DTR | control_line::RTS)?;
        
        self.connected = true;
        Ok(())
    }
    
    /// Close the serial port
    pub fn close(&mut self) -> Result<(), UsbError> {
        // Disable DTR and RTS
        self.set_control_line_state(0)?;
        self.connected = false;
        Ok(())
    }
    
    /// Set line coding (baud rate, data bits, etc.)
    pub fn set_line_coding(&mut self, coding: &LineCoding) -> Result<(), UsbError> {
        let request_type = 0x21; // Host to device, class, interface
        
        self.handle.control_write(
            request_type,
            request::SET_LINE_CODING,
            0,
            self.control_interface as u16,
            &coding.to_bytes(),
            self.timeout,
        )?;
        
        self.line_coding = *coding;
        Ok(())
    }
    
    /// Get current line coding
    pub fn get_line_coding(&self) -> Result<LineCoding, UsbError> {
        let request_type = 0xA1; // Device to host, class, interface
        let mut buf = [0u8; 7];
        
        self.handle.control_read(
            request_type,
            request::GET_LINE_CODING,
            0,
            self.control_interface as u16,
            &mut buf,
            self.timeout,
        )?;
        
        LineCoding::from_bytes(&buf).map_err(|e| UsbError::Parse(e.to_string()))
    }
    
    /// Set control line state (DTR, RTS)
    pub fn set_control_line_state(&mut self, state: u16) -> Result<(), UsbError> {
        let request_type = 0x21; // Host to device, class, interface
        
        self.handle.control_write(
            request_type,
            request::SET_CONTROL_LINE_STATE,
            state,
            self.control_interface as u16,
            &[],
            self.timeout,
        )?;
        
        self.control_line_state = state;
        Ok(())
    }
    
    /// Set DTR
    pub fn set_dtr(&mut self, active: bool) -> Result<(), UsbError> {
        let state = if active {
            self.control_line_state | control_line::DTR
        } else {
            self.control_line_state & !control_line::DTR
        };
        self.set_control_line_state(state)
    }
    
    /// Set RTS
    pub fn set_rts(&mut self, active: bool) -> Result<(), UsbError> {
        let state = if active {
            self.control_line_state | control_line::RTS
        } else {
            self.control_line_state & !control_line::RTS
        };
        self.set_control_line_state(state)
    }
    
    /// Send break signal
    pub fn send_break(&self, duration_ms: u16) -> Result<(), UsbError> {
        let request_type = 0x21;
        
        self.handle.control_write(
            request_type,
            request::SEND_BREAK,
            duration_ms,
            self.control_interface as u16,
            &[],
            self.timeout,
        )?;
        
        Ok(())
    }
    
    /// Read data
    pub fn read(&self, buf: &mut [u8]) -> Result<usize, UsbError> {
        let bulk = BulkTransfer::new(self.handle).with_timeout(self.timeout);
        bulk.read(self.ep_in, buf)
    }
    
    /// Write data
    pub fn write(&self, buf: &[u8]) -> Result<usize, UsbError> {
        let bulk = BulkTransfer::new(self.handle).with_timeout(self.timeout);
        bulk.write(self.ep_out, buf)
    }
    
    /// Write a string
    pub fn write_str(&self, s: &str) -> Result<usize, UsbError> {
        self.write(s.as_bytes())
    }
    
    /// Write a line (with newline)
    pub fn write_line(&self, s: &str) -> Result<usize, UsbError> {
        let line = format!("{}\r\n", s);
        self.write(line.as_bytes())
    }
    
    /// Read a line
    pub fn read_line(&self) -> Result<String, UsbError> {
        let mut result = Vec::new();
        let mut buf = [0u8; 1];
        
        loop {
            let n = self.read(&mut buf)?;
            if n == 0 {
                break;
            }
            
            if buf[0] == b'\n' {
                // Remove trailing \r if present
                if result.last() == Some(&b'\r') {
                    result.pop();
                }
                break;
            }
            
            result.push(buf[0]);
        }
        
        String::from_utf8(result).map_err(|e| UsbError::Parse(e.to_string()))
    }
    
    /// Get current baud rate
    pub fn baud_rate(&self) -> u32 {
        self.line_coding.baud_rate
    }
    
    /// Set baud rate
    pub fn set_baud_rate(&mut self, baud_rate: u32) -> Result<(), UsbError> {
        let mut coding = self.line_coding;
        coding.baud_rate = baud_rate;
        self.set_line_coding(&coding)
    }
}

impl UsbProtocol for CdcAcmClient<'_> {
    fn name(&self) -> &'static str {
        "CDC-ACM"
    }
    
    fn is_connected(&self) -> bool {
        self.connected
    }
    
    fn version(&self) -> Option<String> {
        None
    }
}

/// CDC network client (ECM/NCM) - USB Ethernet
pub struct CdcNetworkInfo {
    /// MAC address
    pub mac_address: [u8; 6],
    /// Max segment size
    pub max_segment_size: u16,
    /// Number of power filters
    pub num_power_filters: u8,
    /// Whether the network is connected
    pub connected: bool,
}

impl CdcNetworkInfo {
    /// Format MAC address as string
    pub fn mac_string(&self) -> String {
        format!(
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.mac_address[0],
            self.mac_address[1],
            self.mac_address[2],
            self.mac_address[3],
            self.mac_address[4],
            self.mac_address[5],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_coding() {
        let coding = LineCoding::new(115200);
        assert_eq!(coding.baud_rate, 115200);
        assert_eq!(coding.data_bits, 8);
        assert_eq!(coding.parity, 0);
        assert_eq!(coding.stop_bits, 0);
        
        let bytes = coding.to_bytes();
        let parsed = LineCoding::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.baud_rate, 115200);
    }

    #[test]
    fn test_line_coding_names() {
        let coding = LineCoding::new(9600);
        assert_eq!(coding.stop_bits_name(), "1");
        assert_eq!(coding.parity_name(), "None");
    }

    #[test]
    fn test_mac_string() {
        let info = CdcNetworkInfo {
            mac_address: [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF],
            max_segment_size: 1500,
            num_power_filters: 0,
            connected: false,
        };
        assert_eq!(info.mac_string(), "AA:BB:CC:DD:EE:FF");
    }
}
