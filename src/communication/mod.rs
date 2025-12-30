//! USB Communication Layer - OMEGA MODE
//!
//! This module provides safe abstractions for USB device communication including:
//! - Control transfers
//! - Bulk transfers
//! - Interrupt transfers
//! - Interface claiming
//! - Device handle management
//! - Transfer timeouts and retry logic

use crate::errors::UsbError;
use crate::model::UsbId;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub mod control;
pub mod bulk;
pub mod interrupt;
pub mod session;

pub use control::*;
pub use bulk::*;
pub use interrupt::*;
pub use session::*;

/// Default timeout for USB operations
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// Maximum retry count for transient errors
pub const MAX_RETRIES: u32 = 3;

/// USB transfer direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Host to device (OUT)
    Out,
    /// Device to host (IN)
    In,
}

/// USB transfer result
#[derive(Debug, Clone)]
pub struct TransferResult {
    /// Number of bytes actually transferred
    pub bytes_transferred: usize,
    /// Whether the transfer completed successfully
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Transfer duration
    pub duration: Duration,
}

impl TransferResult {
    /// Create a successful result
    pub fn success(bytes: usize, duration: Duration) -> Self {
        Self {
            bytes_transferred: bytes,
            success: true,
            error: None,
            duration,
        }
    }
    
    /// Create a failed result
    pub fn failure(error: impl Into<String>, duration: Duration) -> Self {
        Self {
            bytes_transferred: 0,
            success: false,
            error: Some(error.into()),
            duration,
        }
    }
}

/// USB device handle wrapper for safe communication
pub struct DeviceHandle {
    inner: rusb::DeviceHandle<rusb::Context>,
    id: UsbId,
    claimed_interfaces: Vec<u8>,
    kernel_drivers_detached: Vec<u8>,
}

impl DeviceHandle {
    /// Open a device by VID/PID
    pub fn open(vid: u16, pid: u16) -> Result<Self, UsbError> {
        let devices = rusb::devices()?;
        
        for device in devices.iter() {
            let desc = device.device_descriptor()?;
            if desc.vendor_id() == vid && desc.product_id() == pid {
                let handle = device.open()?;
                return Ok(Self {
                    inner: handle,
                    id: UsbId::new(vid, pid),
                    claimed_interfaces: Vec::new(),
                    kernel_drivers_detached: Vec::new(),
                });
            }
        }
        
        Err(UsbError::DeviceNotFound(format!("{:04X}:{:04X}", vid, pid)))
    }
    
    /// Open a device by bus and address
    pub fn open_by_address(bus: u8, address: u8) -> Result<Self, UsbError> {
        let devices = rusb::devices()?;
        
        for device in devices.iter() {
            if device.bus_number() == bus && device.address() == address {
                let desc = device.device_descriptor()?;
                let handle = device.open()?;
                return Ok(Self {
                    inner: handle,
                    id: UsbId::new(desc.vendor_id(), desc.product_id()),
                    claimed_interfaces: Vec::new(),
                    kernel_drivers_detached: Vec::new(),
                });
            }
        }
        
        Err(UsbError::DeviceNotFound(format!("Bus {} Address {}", bus, address)))
    }
    
    /// Get the device ID
    pub fn id(&self) -> &UsbId {
        &self.id
    }
    
    /// Claim an interface for exclusive access
    pub fn claim_interface(&mut self, interface: u8) -> Result<(), UsbError> {
        // Try to detach kernel driver if active
        if self.inner.kernel_driver_active(interface).unwrap_or(false) {
            self.inner.detach_kernel_driver(interface)?;
            self.kernel_drivers_detached.push(interface);
        }
        
        self.inner.claim_interface(interface)?;
        self.claimed_interfaces.push(interface);
        Ok(())
    }
    
    /// Release an interface
    pub fn release_interface(&mut self, interface: u8) -> Result<(), UsbError> {
        self.inner.release_interface(interface)?;
        self.claimed_interfaces.retain(|&i| i != interface);
        
        // Re-attach kernel driver if we detached it
        if self.kernel_drivers_detached.contains(&interface) {
            let _ = self.inner.attach_kernel_driver(interface);
            self.kernel_drivers_detached.retain(|&i| i != interface);
        }
        
        Ok(())
    }
    
    /// Set the active configuration
    pub fn set_configuration(&mut self, config: u8) -> Result<(), UsbError> {
        self.inner.set_active_configuration(config)?;
        Ok(())
    }
    
    /// Set alternate setting for an interface
    pub fn set_alternate_setting(&mut self, interface: u8, setting: u8) -> Result<(), UsbError> {
        self.inner.set_alternate_setting(interface, setting)?;
        Ok(())
    }
    
    /// Get the underlying rusb handle (for advanced operations)
    pub fn raw_handle(&self) -> &rusb::DeviceHandle<rusb::Context> {
        &self.inner
    }
    
    /// Reset the device
    pub fn reset(&mut self) -> Result<(), UsbError> {
        self.inner.reset()?;
        Ok(())
    }
    
    /// Perform a control transfer (IN)
    pub fn control_read(
        &self,
        request_type: u8,
        request: u8,
        value: u16,
        index: u16,
        buf: &mut [u8],
        timeout: Duration,
    ) -> Result<usize, UsbError> {
        let bytes = self.inner.read_control(request_type, request, value, index, buf, timeout)?;
        Ok(bytes)
    }
    
    /// Perform a control transfer (OUT)
    pub fn control_write(
        &self,
        request_type: u8,
        request: u8,
        value: u16,
        index: u16,
        buf: &[u8],
        timeout: Duration,
    ) -> Result<usize, UsbError> {
        let bytes = self.inner.write_control(request_type, request, value, index, buf, timeout)?;
        Ok(bytes)
    }
    
    /// Perform a bulk read
    pub fn bulk_read(
        &self,
        endpoint: u8,
        buf: &mut [u8],
        timeout: Duration,
    ) -> Result<usize, UsbError> {
        let bytes = self.inner.read_bulk(endpoint, buf, timeout)?;
        Ok(bytes)
    }
    
    /// Perform a bulk write
    pub fn bulk_write(
        &self,
        endpoint: u8,
        buf: &[u8],
        timeout: Duration,
    ) -> Result<usize, UsbError> {
        let bytes = self.inner.write_bulk(endpoint, buf, timeout)?;
        Ok(bytes)
    }
    
    /// Perform an interrupt read
    pub fn interrupt_read(
        &self,
        endpoint: u8,
        buf: &mut [u8],
        timeout: Duration,
    ) -> Result<usize, UsbError> {
        let bytes = self.inner.read_interrupt(endpoint, buf, timeout)?;
        Ok(bytes)
    }
    
    /// Perform an interrupt write
    pub fn interrupt_write(
        &self,
        endpoint: u8,
        buf: &[u8],
        timeout: Duration,
    ) -> Result<usize, UsbError> {
        let bytes = self.inner.write_interrupt(endpoint, buf, timeout)?;
        Ok(bytes)
    }
    
    /// Clear a HALT/STALL condition on an endpoint
    pub fn clear_halt(&self, endpoint: u8) -> Result<(), UsbError> {
        self.inner.clear_halt(endpoint)?;
        Ok(())
    }
}

impl Drop for DeviceHandle {
    fn drop(&mut self) {
        // Release all claimed interfaces
        for &interface in &self.claimed_interfaces.clone() {
            let _ = self.release_interface(interface);
        }
    }
}

/// Thread-safe device handle pool
pub struct DevicePool {
    handles: Arc<Mutex<Vec<DeviceHandle>>>,
}

impl DevicePool {
    /// Create a new empty pool
    pub fn new() -> Self {
        Self {
            handles: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Add a device to the pool
    pub fn add(&self, handle: DeviceHandle) {
        let mut handles = self.handles.lock().unwrap();
        handles.push(handle);
    }
    
    /// Get a device from the pool by VID/PID
    pub fn get(&self, vid: u16, pid: u16) -> Option<DeviceHandle> {
        let mut handles = self.handles.lock().unwrap();
        let idx = handles.iter().position(|h| h.id.vid == vid && h.id.pid == pid)?;
        Some(handles.remove(idx))
    }
    
    /// Return a device to the pool
    pub fn return_handle(&self, handle: DeviceHandle) {
        self.add(handle);
    }
    
    /// Get the number of devices in the pool
    pub fn len(&self) -> usize {
        self.handles.lock().unwrap().len()
    }
    
    /// Check if the pool is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for DevicePool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_result() {
        let success = TransferResult::success(64, Duration::from_millis(10));
        assert!(success.success);
        assert_eq!(success.bytes_transferred, 64);
        
        let failure = TransferResult::failure("timeout", Duration::from_millis(5000));
        assert!(!failure.success);
        assert!(failure.error.is_some());
    }

    #[test]
    fn test_device_pool() {
        let pool = DevicePool::new();
        assert!(pool.is_empty());
    }
}
