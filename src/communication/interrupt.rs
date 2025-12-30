//! USB Interrupt Transfer Utilities
//!
//! Provides helpers for interrupt transfer operations, commonly used for HID devices.

use super::{DeviceHandle, TransferResult, DEFAULT_TIMEOUT};
use crate::errors::UsbError;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Interrupt transfer helper
pub struct InterruptTransfer<'a> {
    handle: &'a DeviceHandle,
    timeout: Duration,
}

impl<'a> InterruptTransfer<'a> {
    /// Create a new interrupt transfer helper
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
    
    /// Read data from an interrupt IN endpoint
    pub fn read(&self, endpoint: u8, buf: &mut [u8]) -> Result<usize, UsbError> {
        self.handle.interrupt_read(endpoint | 0x80, buf, self.timeout)
    }
    
    /// Write data to an interrupt OUT endpoint
    pub fn write(&self, endpoint: u8, buf: &[u8]) -> Result<usize, UsbError> {
        self.handle.interrupt_write(endpoint & 0x7F, buf, self.timeout)
    }
    
    /// Read with result info
    pub fn read_with_result(&self, endpoint: u8, buf: &mut [u8]) -> TransferResult {
        let start = Instant::now();
        
        match self.read(endpoint, buf) {
            Ok(bytes) => TransferResult::success(bytes, start.elapsed()),
            Err(e) => TransferResult::failure(e.to_string(), start.elapsed()),
        }
    }
    
    /// Try to read with a short timeout (non-blocking style)
    pub fn try_read(&self, endpoint: u8, buf: &mut [u8]) -> Option<usize> {
        let short_timeout = Duration::from_millis(1);
        self.handle.interrupt_read(endpoint | 0x80, buf, short_timeout).ok()
    }
}

/// Interrupt endpoint poller for continuous reading
pub struct InterruptPoller {
    endpoint: u8,
    buffer_size: usize,
    poll_interval: Duration,
    running: Arc<AtomicBool>,
}

impl InterruptPoller {
    /// Create a new interrupt poller
    pub fn new(endpoint: u8, buffer_size: usize) -> Self {
        Self {
            endpoint: endpoint | 0x80,
            buffer_size,
            poll_interval: Duration::from_millis(10),
            running: Arc::new(AtomicBool::new(false)),
        }
    }
    
    /// Set the poll interval
    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }
    
    /// Start polling (blocking)
    pub fn poll<F>(&self, handle: &DeviceHandle, mut callback: F) -> Result<(), UsbError>
    where
        F: FnMut(&[u8]) -> bool, // Return false to stop polling
    {
        self.running.store(true, Ordering::SeqCst);
        let mut buffer = vec![0u8; self.buffer_size];
        
        while self.running.load(Ordering::SeqCst) {
            match handle.interrupt_read(self.endpoint, &mut buffer, self.poll_interval) {
                Ok(bytes) => {
                    if !callback(&buffer[..bytes]) {
                        break;
                    }
                }
                Err(UsbError::UsbLib(rusb::Error::Timeout)) => {
                    // Timeout is expected, continue polling
                    continue;
                }
                Err(e) => {
                    self.running.store(false, Ordering::SeqCst);
                    return Err(e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Stop polling
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
    
    /// Check if polling is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

/// HID-specific interrupt helpers
pub mod hid {
    use super::*;
    
    /// HID report types
    #[derive(Debug, Clone, Copy)]
    pub enum ReportType {
        /// Input report
        Input = 0x01,
        /// Output report
        Output = 0x02,
        /// Feature report
        Feature = 0x03,
    }
    
    /// HID class requests
    pub mod request {
        /// GET_REPORT
        pub const GET_REPORT: u8 = 0x01;
        /// GET_IDLE
        pub const GET_IDLE: u8 = 0x02;
        /// GET_PROTOCOL
        pub const GET_PROTOCOL: u8 = 0x03;
        /// SET_REPORT
        pub const SET_REPORT: u8 = 0x09;
        /// SET_IDLE
        pub const SET_IDLE: u8 = 0x0A;
        /// SET_PROTOCOL
        pub const SET_PROTOCOL: u8 = 0x0B;
    }
    
    /// HID device helper
    pub struct HidDevice<'a> {
        handle: &'a DeviceHandle,
        interface: u8,
        timeout: Duration,
    }
    
    impl<'a> HidDevice<'a> {
        /// Create a new HID device helper
        pub fn new(handle: &'a DeviceHandle, interface: u8) -> Self {
            Self {
                handle,
                interface,
                timeout: Duration::from_secs(1),
            }
        }
        
        /// Set timeout
        pub fn with_timeout(mut self, timeout: Duration) -> Self {
            self.timeout = timeout;
            self
        }
        
        /// Get a HID report using control transfer
        pub fn get_report(&self, report_type: ReportType, report_id: u8, buf: &mut [u8]) -> Result<usize, UsbError> {
            let request_type = 0xA1; // Device to host, class, interface
            let value = ((report_type as u16) << 8) | (report_id as u16);
            
            self.handle.control_read(
                request_type,
                request::GET_REPORT,
                value,
                self.interface as u16,
                buf,
                self.timeout,
            )
        }
        
        /// Set a HID report using control transfer
        pub fn set_report(&self, report_type: ReportType, report_id: u8, buf: &[u8]) -> Result<usize, UsbError> {
            let request_type = 0x21; // Host to device, class, interface
            let value = ((report_type as u16) << 8) | (report_id as u16);
            
            self.handle.control_write(
                request_type,
                request::SET_REPORT,
                value,
                self.interface as u16,
                buf,
                self.timeout,
            )
        }
        
        /// Get idle rate
        pub fn get_idle(&self, report_id: u8) -> Result<u8, UsbError> {
            let mut buf = [0u8; 1];
            let request_type = 0xA1;
            
            self.handle.control_read(
                request_type,
                request::GET_IDLE,
                report_id as u16,
                self.interface as u16,
                &mut buf,
                self.timeout,
            )?;
            
            Ok(buf[0])
        }
        
        /// Set idle rate
        pub fn set_idle(&self, report_id: u8, duration: u8) -> Result<(), UsbError> {
            let request_type = 0x21;
            let value = ((duration as u16) << 8) | (report_id as u16);
            
            self.handle.control_write(
                request_type,
                request::SET_IDLE,
                value,
                self.interface as u16,
                &[],
                self.timeout,
            )?;
            
            Ok(())
        }
        
        /// Get protocol (0 = boot, 1 = report)
        pub fn get_protocol(&self) -> Result<u8, UsbError> {
            let mut buf = [0u8; 1];
            let request_type = 0xA1;
            
            self.handle.control_read(
                request_type,
                request::GET_PROTOCOL,
                0,
                self.interface as u16,
                &mut buf,
                self.timeout,
            )?;
            
            Ok(buf[0])
        }
        
        /// Set protocol
        pub fn set_protocol(&self, protocol: u8) -> Result<(), UsbError> {
            let request_type = 0x21;
            
            self.handle.control_write(
                request_type,
                request::SET_PROTOCOL,
                protocol as u16,
                self.interface as u16,
                &[],
                self.timeout,
            )?;
            
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interrupt_poller() {
        let poller = InterruptPoller::new(0x81, 64)
            .with_poll_interval(Duration::from_millis(100));
        
        assert!(!poller.is_running());
    }
}
