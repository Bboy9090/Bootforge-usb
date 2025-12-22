use super::{DeviceEvent, DeviceWatcher};
use std::sync::mpsc::{channel, Receiver, Sender};

#[cfg(feature = "udev")]
use std::thread;

#[cfg(feature = "udev")]
use crate::model::*;

pub struct LinuxDeviceWatcher {
    running: bool,
    sender: Option<Sender<DeviceEvent>>,
}

impl LinuxDeviceWatcher {
    pub fn new() -> Self {
        Self {
            running: false,
            sender: None,
        }
    }
}

impl Default for LinuxDeviceWatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl DeviceWatcher for LinuxDeviceWatcher {
    fn start(&mut self) -> Result<Receiver<DeviceEvent>, Box<dyn std::error::Error>> {
        let (tx, rx) = channel();
        self.sender = Some(tx.clone());
        self.running = true;

        #[cfg(feature = "udev")]
        {
            thread::spawn(move || {
                // Use udev to monitor USB device changes
                let context = match udev::Context::new() {
                    Ok(ctx) => ctx,
                    Err(e) => {
                        eprintln!("Failed to create udev context: {}", e);
                        return;
                    }
                };

                let mut monitor = match udev::MonitorBuilder::new(&context) {
                    Ok(builder) => match builder
                        .match_subsystem("usb")
                        .and_then(|b| b.listen()) {
                        Ok(mon) => mon,
                        Err(e) => {
                            eprintln!("Failed to create udev monitor: {}", e);
                            return;
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to create monitor builder: {}", e);
                        return;
                    }
                };

                for event in monitor.iter() {
                    let action = event.action().map(|s| s.to_string_lossy().to_string());
                    
                    if let Some(device) = event.device() {
                        let vid = device.attribute_value("idVendor")
                            .and_then(|v| u16::from_str_radix(&v.to_string_lossy(), 16).ok());
                        let pid = device.attribute_value("idProduct")
                            .and_then(|v| u16::from_str_radix(&v.to_string_lossy(), 16).ok());

                        if let (Some(vid), Some(pid)) = (vid, pid) {
                            let record = UsbDeviceRecord {
                                id: UsbId::new(vid, pid),
                                location: UsbLocation {
                                    bus: device.attribute_value("busnum")
                                        .and_then(|v| v.to_str().and_then(|s| s.parse().ok())),
                                    address: device.attribute_value("devnum")
                                        .and_then(|v| v.to_str().and_then(|s| s.parse().ok())),
                                    port_path: device.devpath().map(|p| p.to_string_lossy().to_string()),
                                },
                                descriptor: UsbDescriptorSummary {
                                    manufacturer: device.attribute_value("manufacturer")
                                        .and_then(|v| v.to_str().map(|s| s.to_string())),
                                    product: device.attribute_value("product")
                                        .and_then(|v| v.to_str().map(|s| s.to_string())),
                                    serial_number: device.attribute_value("serial")
                                        .and_then(|v| v.to_str().map(|s| s.to_string())),
                                    device_class: device.attribute_value("bDeviceClass")
                                        .and_then(|v| u8::from_str_radix(&v.to_string_lossy(), 16).ok()),
                                    device_subclass: device.attribute_value("bDeviceSubClass")
                                        .and_then(|v| u8::from_str_radix(&v.to_string_lossy(), 16).ok()),
                                    device_protocol: device.attribute_value("bDeviceProtocol")
                                        .and_then(|v| u8::from_str_radix(&v.to_string_lossy(), 16).ok()),
                                    usb_version: device.attribute_value("version")
                                        .and_then(|v| v.to_str().map(|s| s.to_string())),
                                },
                                driver: DriverStatus::Unknown,
                                health: LinkHealth::Good,
                                tags: Vec::new(),
                                raw_data: None,
                            };

                            let event = match action.as_deref() {
                                Some("add") => DeviceEvent::Added(record),
                                Some("remove") => DeviceEvent::Removed(record),
                                Some("change") => DeviceEvent::Changed(record),
                                _ => continue,
                            };

                            if tx.send(event).is_err() {
                                break;
                            }
                        }
                    }
                }
            });
        }

        #[cfg(not(feature = "udev"))]
        {
            log::warn!("Linux device watching requires udev feature to be enabled");
        }

        Ok(rx)
    }

    fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.running = false;
        self.sender = None;
        Ok(())
    }
}
