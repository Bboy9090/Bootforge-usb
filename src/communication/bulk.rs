//! USB Bulk Transfer Utilities
//!
//! Provides helpers for bulk transfer operations with retry logic and chunking.

use super::{DeviceHandle, TransferResult, DEFAULT_TIMEOUT, MAX_RETRIES};
use crate::errors::UsbError;
use std::time::{Duration, Instant};

/// Bulk transfer helper with advanced features
pub struct BulkTransfer<'a> {
    handle: &'a DeviceHandle,
    timeout: Duration,
    max_retries: u32,
    chunk_size: Option<usize>,
}

impl<'a> BulkTransfer<'a> {
    /// Create a new bulk transfer helper
    pub fn new(handle: &'a DeviceHandle) -> Self {
        Self {
            handle,
            timeout: DEFAULT_TIMEOUT,
            max_retries: MAX_RETRIES,
            chunk_size: None,
        }
    }
    
    /// Set the timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    /// Set the maximum retry count
    pub fn with_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }
    
    /// Set chunk size for large transfers
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = Some(size);
        self
    }
    
    /// Read data from a bulk IN endpoint
    pub fn read(&self, endpoint: u8, buf: &mut [u8]) -> Result<usize, UsbError> {
        let mut last_error = None;
        
        for attempt in 0..=self.max_retries {
            match self.handle.bulk_read(endpoint | 0x80, buf, self.timeout) {
                Ok(bytes) => return Ok(bytes),
                Err(e) => {
                    if attempt < self.max_retries && is_retryable(&e) {
                        std::thread::sleep(Duration::from_millis(10 * (attempt as u64 + 1)));
                        last_error = Some(e);
                        continue;
                    }
                    return Err(e);
                }
            }
        }
        
        Err(last_error.unwrap_or(UsbError::Unknown("Max retries exceeded".into())))
    }
    
    /// Write data to a bulk OUT endpoint
    pub fn write(&self, endpoint: u8, buf: &[u8]) -> Result<usize, UsbError> {
        if let Some(chunk_size) = self.chunk_size {
            self.write_chunked(endpoint, buf, chunk_size)
        } else {
            self.write_single(endpoint, buf)
        }
    }
    
    /// Write data in a single transfer
    fn write_single(&self, endpoint: u8, buf: &[u8]) -> Result<usize, UsbError> {
        let mut last_error = None;
        
        for attempt in 0..=self.max_retries {
            match self.handle.bulk_write(endpoint & 0x7F, buf, self.timeout) {
                Ok(bytes) => return Ok(bytes),
                Err(e) => {
                    if attempt < self.max_retries && is_retryable(&e) {
                        std::thread::sleep(Duration::from_millis(10 * (attempt as u64 + 1)));
                        last_error = Some(e);
                        continue;
                    }
                    return Err(e);
                }
            }
        }
        
        Err(last_error.unwrap_or(UsbError::Unknown("Max retries exceeded".into())))
    }
    
    /// Write data in chunks
    fn write_chunked(&self, endpoint: u8, buf: &[u8], chunk_size: usize) -> Result<usize, UsbError> {
        let mut total_written = 0;
        
        for chunk in buf.chunks(chunk_size) {
            let written = self.write_single(endpoint, chunk)?;
            total_written += written;
            
            if written < chunk.len() {
                // Short write, stop here
                break;
            }
        }
        
        Ok(total_written)
    }
    
    /// Read exact number of bytes (may require multiple transfers)
    pub fn read_exact(&self, endpoint: u8, buf: &mut [u8]) -> Result<(), UsbError> {
        let mut total_read = 0;
        let chunk_size = self.chunk_size.unwrap_or(buf.len());
        
        while total_read < buf.len() {
            let remaining = buf.len() - total_read;
            let to_read = remaining.min(chunk_size);
            
            let read = self.read(endpoint, &mut buf[total_read..total_read + to_read])?;
            
            if read == 0 {
                return Err(UsbError::Unknown("Short read".into()));
            }
            
            total_read += read;
        }
        
        Ok(())
    }
    
    /// Read with result info including timing
    pub fn read_with_result(&self, endpoint: u8, buf: &mut [u8]) -> TransferResult {
        let start = Instant::now();
        
        match self.read(endpoint, buf) {
            Ok(bytes) => TransferResult::success(bytes, start.elapsed()),
            Err(e) => TransferResult::failure(e.to_string(), start.elapsed()),
        }
    }
    
    /// Write with result info including timing
    pub fn write_with_result(&self, endpoint: u8, buf: &[u8]) -> TransferResult {
        let start = Instant::now();
        
        match self.write(endpoint, buf) {
            Ok(bytes) => TransferResult::success(bytes, start.elapsed()),
            Err(e) => TransferResult::failure(e.to_string(), start.elapsed()),
        }
    }
}

/// Stream-like bulk reader
pub struct BulkReader<'a> {
    handle: &'a DeviceHandle,
    endpoint: u8,
    timeout: Duration,
    buffer: Vec<u8>,
    buffer_pos: usize,
    buffer_len: usize,
}

impl<'a> BulkReader<'a> {
    /// Create a new bulk reader
    pub fn new(handle: &'a DeviceHandle, endpoint: u8, buffer_size: usize) -> Self {
        Self {
            handle,
            endpoint: endpoint | 0x80,
            timeout: DEFAULT_TIMEOUT,
            buffer: vec![0u8; buffer_size],
            buffer_pos: 0,
            buffer_len: 0,
        }
    }
    
    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    /// Read into buffer, returns bytes read
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, UsbError> {
        let mut written = 0;
        
        // First, drain any buffered data
        while written < buf.len() && self.buffer_pos < self.buffer_len {
            buf[written] = self.buffer[self.buffer_pos];
            written += 1;
            self.buffer_pos += 1;
        }
        
        // If we need more data, read from device
        while written < buf.len() {
            self.buffer_len = self.handle.bulk_read(
                self.endpoint,
                &mut self.buffer,
                self.timeout,
            )?;
            self.buffer_pos = 0;
            
            if self.buffer_len == 0 {
                break;
            }
            
            while written < buf.len() && self.buffer_pos < self.buffer_len {
                buf[written] = self.buffer[self.buffer_pos];
                written += 1;
                self.buffer_pos += 1;
            }
        }
        
        Ok(written)
    }
    
    /// Read a line (until \n or \r\n)
    pub fn read_line(&mut self) -> Result<String, UsbError> {
        let mut line = Vec::new();
        let mut byte = [0u8; 1];
        
        loop {
            let read = self.read(&mut byte)?;
            if read == 0 {
                break;
            }
            
            if byte[0] == b'\n' {
                // Remove trailing \r if present
                if line.last() == Some(&b'\r') {
                    line.pop();
                }
                break;
            }
            
            line.push(byte[0]);
        }
        
        String::from_utf8(line).map_err(|e| UsbError::Parse(e.to_string()))
    }
}

/// Stream-like bulk writer
pub struct BulkWriter<'a> {
    handle: &'a DeviceHandle,
    endpoint: u8,
    timeout: Duration,
    buffer: Vec<u8>,
    buffer_size: usize,
}

impl<'a> BulkWriter<'a> {
    /// Create a new bulk writer
    pub fn new(handle: &'a DeviceHandle, endpoint: u8, buffer_size: usize) -> Self {
        Self {
            handle,
            endpoint: endpoint & 0x7F,
            timeout: DEFAULT_TIMEOUT,
            buffer: Vec::with_capacity(buffer_size),
            buffer_size,
        }
    }
    
    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    /// Write data (may buffer)
    pub fn write(&mut self, buf: &[u8]) -> Result<usize, UsbError> {
        for &byte in buf {
            self.buffer.push(byte);
            
            if self.buffer.len() >= self.buffer_size {
                self.flush()?;
            }
        }
        
        Ok(buf.len())
    }
    
    /// Flush the buffer
    pub fn flush(&mut self) -> Result<(), UsbError> {
        if !self.buffer.is_empty() {
            self.handle.bulk_write(self.endpoint, &self.buffer, self.timeout)?;
            self.buffer.clear();
        }
        Ok(())
    }
    
    /// Write a line (appends \n)
    pub fn write_line(&mut self, line: &str) -> Result<(), UsbError> {
        self.write(line.as_bytes())?;
        self.write(b"\n")?;
        self.flush()
    }
}

impl Drop for BulkWriter<'_> {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

/// Check if an error is retryable
fn is_retryable(error: &UsbError) -> bool {
    match error {
        UsbError::UsbLib(e) => matches!(
            e,
            rusb::Error::Timeout | rusb::Error::Busy | rusb::Error::Interrupted
        ),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_retryable() {
        assert!(is_retryable(&UsbError::UsbLib(rusb::Error::Timeout)));
        assert!(is_retryable(&UsbError::UsbLib(rusb::Error::Busy)));
        assert!(!is_retryable(&UsbError::UsbLib(rusb::Error::NotFound)));
    }
}
