use crate::errors::UsbError;
use crate::model::UsbDeviceRecord;

/// Trait for USB device enumeration implementations.
/// 
/// UsbEnumerator provides an abstraction for discovering USB devices.
/// Implementations may use different backends (libusb, platform-specific APIs)
/// but all produce `UsbDeviceRecord` results representing confirmed devices.
/// 
/// # Thread Safety
/// Enumeration is not thread-safe by default. Applications should call
/// enumerate() from a single thread or implement external synchronization.
/// Multiple concurrent enumerations may cause USB bus contention.
/// 
/// # Concurrency Model
/// - Multiple readers: Safe to read device records concurrently after enumeration
/// - Multiple writers: NOT safe to enumerate concurrently without external locks
/// - Mixed read/write: NOT safe to enumerate while operating on devices
/// 
/// See ARCHITECTURE.md for details on operation safety patterns.
pub trait UsbEnumerator: Send + Sync {
    /// Enumerate all USB devices and return confirmed device records.
    /// 
    /// This executes the detection pipeline and returns fully-identified devices.
    /// Errors may occur due to USB access permissions or hardware issues, but
    /// partial results may still be returned when possible.
    fn enumerate(&self) -> Result<Vec<UsbDeviceRecord>, UsbError>;

    /// Get a specific device by vendor and product ID.
    /// 
    /// This is a convenience method that enumerates all devices and filters
    /// by VID/PID. For tracking specific device instances, also check serial
    /// number or port path.
    fn get_device(&self, vid: u16, pid: u16) -> Result<Option<UsbDeviceRecord>, UsbError> {
        Ok(self.enumerate()?.into_iter().find(|d| d.id.vid == vid && d.id.pid == pid))
    }

    /// Check if a device with the given VID/PID is currently connected.
    /// 
    /// This is a lightweight check but still requires full enumeration.
    /// For continuous monitoring, use DeviceWatcher instead of polling.
    fn is_connected(&self, vid: u16, pid: u16) -> Result<bool, UsbError> {
        Ok(self.get_device(vid, pid)?.is_some())
    }
}
