use super::{DeviceEvent, DeviceWatcher};
use crate::model::*;
use std::sync::mpsc::{channel, Receiver, Sender};

pub struct MacOSDeviceWatcher {
    running: bool,
    sender: Option<Sender<DeviceEvent>>,
}

impl MacOSDeviceWatcher {
    pub fn new() -> Self {
        Self {
            running: false,
            sender: None,
        }
    }
}

impl Default for MacOSDeviceWatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl DeviceWatcher for MacOSDeviceWatcher {
    fn start(&mut self) -> Result<Receiver<DeviceEvent>, Box<dyn std::error::Error>> {
        let (tx, rx) = channel();
        self.sender = Some(tx.clone());
        self.running = true;

        // Note: Full macOS implementation would use IOKit notifications
        // This is a placeholder that demonstrates the interface
        log::warn!("macOS device watching not fully implemented yet");

        Ok(rx)
    }

    fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.running = false;
        self.sender = None;
        Ok(())
    }
}
