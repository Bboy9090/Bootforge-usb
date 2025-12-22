use crate::errors::UsbError;
use crate::model::UsbDeviceRecord;

pub trait UsbEnumerator: Send + Sync {
    fn enumerate(&self) -> Result<Vec<UsbDeviceRecord>, UsbError>;

    fn get_device(&self, vid: u16, pid: u16) -> Result<Option<UsbDeviceRecord>, UsbError> {
        Ok(self.enumerate()?.into_iter().find(|d| d.id.vid == vid && d.id.pid == pid))
    }

    fn is_connected(&self, vid: u16, pid: u16) -> Result<bool, UsbError> {
        Ok(self.get_device(vid, pid)?.is_some())
    }
}
