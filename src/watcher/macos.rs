//! macOS Device Watcher - Full Implementation
//!
//! Uses IOKit notifications (IOServiceAddMatchingNotification) for
//! real-time USB device hotplug monitoring.

use super::{DeviceEvent, DeviceWatcher};
use crate::model::*;
use std::sync::mpsc::{channel, Receiver, Sender};

#[cfg(target_os = "macos")]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(target_os = "macos")]
use std::sync::Arc;
#[cfg(target_os = "macos")]
use std::thread::{self, JoinHandle};

#[cfg(target_os = "macos")]
use core_foundation::{
    base::{kCFAllocatorDefault, CFType, TCFType},
    dictionary::{CFDictionary, CFDictionaryRef, CFMutableDictionary},
    number::{CFNumber, CFNumberRef},
    runloop::{kCFRunLoopDefaultMode, CFRunLoop, CFRunLoopRef, CFRunLoopRun, CFRunLoopStop},
    string::{CFString, CFStringRef},
};

#[cfg(target_os = "macos")]
use io_kit_sys::{
    kIOMasterPortDefault,
    kIOMatchedNotification,
    kIOTerminatedNotification,
    ret::kIOReturnSuccess,
    types::{io_iterator_t, io_object_t, io_service_t, IONotificationPortRef},
    IOIteratorNext,
    IONotificationPortCreate,
    IONotificationPortDestroy,
    IONotificationPortGetRunLoopSource,
    IOObjectRelease,
    IORegistryEntryCreateCFProperties,
    IOServiceAddMatchingNotification,
    IOServiceMatching,
};

#[cfg(target_os = "macos")]
use std::ffi::c_void;
#[cfg(target_os = "macos")]
use std::ptr;

/// macOS device watcher using IOKit notifications
pub struct MacOSDeviceWatcher {
    running: bool,
    sender: Option<Sender<DeviceEvent>>,
    #[cfg(target_os = "macos")]
    stop_flag: Arc<AtomicBool>,
    #[cfg(target_os = "macos")]
    thread_handle: Option<JoinHandle<()>>,
    #[cfg(target_os = "macos")]
    runloop_ref: Arc<std::sync::Mutex<Option<CFRunLoopRef>>>,
}

impl MacOSDeviceWatcher {
    pub fn new() -> Self {
        Self {
            running: false,
            sender: None,
            #[cfg(target_os = "macos")]
            stop_flag: Arc::new(AtomicBool::new(false)),
            #[cfg(target_os = "macos")]
            thread_handle: None,
            #[cfg(target_os = "macos")]
            runloop_ref: Arc::new(std::sync::Mutex::new(None)),
        }
    }
}

impl Default for MacOSDeviceWatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_os = "macos")]
struct WatcherContext {
    tx: Sender<DeviceEvent>,
    is_removal: bool,
}

#[cfg(target_os = "macos")]
impl DeviceWatcher for MacOSDeviceWatcher {
    fn start(&mut self) -> Result<Receiver<DeviceEvent>, Box<dyn std::error::Error>> {
        let (tx, rx) = channel();
        self.sender = Some(tx.clone());
        self.running = true;
        self.stop_flag.store(false, Ordering::SeqCst);
        
        let stop_flag = self.stop_flag.clone();
        let runloop_ref = self.runloop_ref.clone();
        
        let handle = thread::spawn(move || {
            if let Err(e) = run_macos_watcher(tx, stop_flag, runloop_ref) {
                log::error!("macOS device watcher error: {}", e);
            }
        });
        
        self.thread_handle = Some(handle);
        
        Ok(rx)
    }

    fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.running = false;
        self.stop_flag.store(true, Ordering::SeqCst);
        
        // Stop the run loop
        if let Ok(guard) = self.runloop_ref.lock() {
            if let Some(runloop) = *guard {
                unsafe {
                    CFRunLoopStop(runloop);
                }
            }
        }
        
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
        
        self.sender = None;
        Ok(())
    }
}

#[cfg(target_os = "macos")]
fn run_macos_watcher(
    tx: Sender<DeviceEvent>,
    _stop_flag: Arc<AtomicBool>,
    runloop_ref: Arc<std::sync::Mutex<Option<CFRunLoopRef>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        // Create notification port
        let notify_port = IONotificationPortCreate(kIOMasterPortDefault);
        if notify_port.is_null() {
            return Err("Failed to create IONotificationPort".into());
        }
        
        // Get the run loop source from the notification port
        let run_loop_source = IONotificationPortGetRunLoopSource(notify_port);
        
        // Get current run loop and add the source
        let run_loop = CFRunLoop::get_current();
        let run_loop_ref = run_loop.as_concrete_TypeRef();
        
        // Store run loop reference for stop()
        if let Ok(mut guard) = runloop_ref.lock() {
            *guard = Some(run_loop_ref);
        }
        
        core_foundation::runloop::CFRunLoopAddSource(
            run_loop_ref,
            run_loop_source,
            kCFRunLoopDefaultMode,
        );
        
        // Set up matching for USB devices - both IOUSBHostDevice (modern) and IOUSBDevice (legacy)
        for service_name in &["IOUSBHostDevice", "IOUSBDevice"] {
            let matching_dict = IOServiceMatching(service_name.as_ptr() as *const i8);
            if matching_dict.is_null() {
                continue;
            }
            
            // Need two copies of matching dict (one for add, one for remove)
            let matching_dict_remove = CFDictionary::<CFString, CFType>::wrap_under_get_rule(
                matching_dict as CFDictionaryRef
            ).to_untyped().as_CFTypeRef() as CFDictionaryRef;
            
            // Create context for callbacks
            let add_context = Box::new(WatcherContext {
                tx: tx.clone(),
                is_removal: false,
            });
            let remove_context = Box::new(WatcherContext {
                tx: tx.clone(),
                is_removal: true,
            });
            
            let mut add_iterator: io_iterator_t = 0;
            let mut remove_iterator: io_iterator_t = 0;
            
            // Register for device additions
            let result = IOServiceAddMatchingNotification(
                notify_port,
                kIOMatchedNotification,
                matching_dict,
                Some(device_callback),
                Box::into_raw(add_context) as *mut c_void,
                &mut add_iterator,
            );
            
            if result == kIOReturnSuccess {
                // Drain the iterator to arm the notification
                drain_iterator(add_iterator, &tx, false);
            }
            
            // Register for device removals
            let result = IOServiceAddMatchingNotification(
                notify_port,
                kIOTerminatedNotification,
                matching_dict_remove as *mut _,
                Some(device_callback),
                Box::into_raw(remove_context) as *mut c_void,
                &mut remove_iterator,
            );
            
            if result == kIOReturnSuccess {
                // Drain the iterator to arm the notification
                drain_iterator(remove_iterator, &tx, true);
            }
        }
        
        // Run the run loop
        CFRunLoopRun();
        
        // Cleanup
        IONotificationPortDestroy(notify_port);
    }
    
    Ok(())
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn device_callback(refcon: *mut c_void, iterator: io_iterator_t) {
    let context = refcon as *const WatcherContext;
    if context.is_null() {
        return;
    }
    
    let ctx = &*context;
    drain_iterator(iterator, &ctx.tx, ctx.is_removal);
}

#[cfg(target_os = "macos")]
unsafe fn drain_iterator(iterator: io_iterator_t, tx: &Sender<DeviceEvent>, is_removal: bool) {
    loop {
        let service = IOIteratorNext(iterator);
        if service == 0 {
            break;
        }
        
        if let Some(record) = create_device_record_from_service(service) {
            let event = if is_removal {
                DeviceEvent::Removed(record)
            } else {
                DeviceEvent::Added(record)
            };
            
            let _ = tx.send(event);
        }
        
        IOObjectRelease(service);
    }
}

#[cfg(target_os = "macos")]
unsafe fn create_device_record_from_service(service: io_service_t) -> Option<UsbDeviceRecord> {
    // Get device properties
    let mut props: CFDictionaryRef = ptr::null();
    let result = IORegistryEntryCreateCFProperties(
        service,
        &mut props,
        kCFAllocatorDefault,
        0,
    );
    
    if result != kIOReturnSuccess || props.is_null() {
        return None;
    }
    
    let props_dict = CFDictionary::<CFString, CFType>::wrap_under_create_rule(props);
    
    // Extract VID/PID
    let vid = get_number_property(&props_dict, "idVendor")? as u16;
    let pid = get_number_property(&props_dict, "idProduct")? as u16;
    
    // Get location ID
    let location_id = get_number_property(&props_dict, "locationID");
    
    // Get bus and address
    let bus = get_number_property(&props_dict, "USB Address").map(|v| v as u8);
    let address = get_number_property(&props_dict, "USB Device Speed").map(|v| v as u8);
    
    // Get strings
    let manufacturer = get_string_property(&props_dict, "USB Vendor Name");
    let product = get_string_property(&props_dict, "USB Product Name");
    let serial_number = get_string_property(&props_dict, "USB Serial Number");
    
    // Get class info
    let device_class = get_number_property(&props_dict, "bDeviceClass").map(|v| v as u8);
    let device_subclass = get_number_property(&props_dict, "bDeviceSubClass").map(|v| v as u8);
    let device_protocol = get_number_property(&props_dict, "bDeviceProtocol").map(|v| v as u8);
    
    // Format port path from location ID
    let port_path = location_id.map(|loc| format_location_path(loc as u32));
    
    Some(UsbDeviceRecord {
        id: UsbId::new(vid, pid),
        location: UsbLocation {
            bus,
            address,
            port_path,
        },
        descriptor: UsbDescriptorSummary {
            manufacturer,
            product,
            serial_number,
            device_class,
            device_subclass,
            device_protocol,
            usb_version: None,
        },
        driver: DriverStatus::Unknown,
        health: LinkHealth::Good,
        tags: vec!["macos".to_string()],
        raw_data: None,
    })
}

#[cfg(target_os = "macos")]
fn get_number_property(props: &CFDictionary<CFString, CFType>, key: &str) -> Option<i64> {
    let key_string = CFString::new(key);
    let value = props.find(&key_string)?;
    
    unsafe {
        let number = CFNumber::wrap_under_get_rule(value.as_CFTypeRef() as CFNumberRef);
        let mut result: i64 = 0;
        if number.to_i64(&mut result) {
            Some(result)
        } else {
            None
        }
    }
}

#[cfg(target_os = "macos")]
fn get_string_property(props: &CFDictionary<CFString, CFType>, key: &str) -> Option<String> {
    let key_string = CFString::new(key);
    let value = props.find(&key_string)?;
    
    unsafe {
        let string = CFString::wrap_under_get_rule(value.as_CFTypeRef() as CFStringRef);
        Some(string.to_string())
    }
}

#[cfg(target_os = "macos")]
fn format_location_path(location_id: u32) -> String {
    let bus = (location_id >> 24) & 0xFF;
    let mut ports = Vec::new();
    
    for i in (0..6).rev() {
        let port = (location_id >> (i * 4)) & 0xF;
        if port != 0 {
            ports.push(port);
        }
    }
    
    if ports.is_empty() {
        format!("bus-{}", bus)
    } else {
        format!(
            "bus-{}-{}",
            bus,
            ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(".")
        )
    }
}

#[cfg(not(target_os = "macos"))]
impl DeviceWatcher for MacOSDeviceWatcher {
    fn start(&mut self) -> Result<Receiver<DeviceEvent>, Box<dyn std::error::Error>> {
        let (tx, rx) = channel();
        self.sender = Some(tx);
        self.running = true;
        log::warn!("macOS device watching only available on macOS");
        Ok(rx)
    }

    fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.running = false;
        self.sender = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watcher_creation() {
        let watcher = MacOSDeviceWatcher::new();
        assert!(!watcher.running);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_location_path_formatting() {
        // Bus 20, port 1
        assert_eq!(format_location_path(0x14100000), "bus-20-1");
        // Bus 20, port 1, sub-port 2
        assert_eq!(format_location_path(0x14120000), "bus-20-2.1");
    }
}
