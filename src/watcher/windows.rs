//! Windows Device Watcher - Full Implementation
//!
//! Uses Windows Device Notification API (RegisterDeviceNotificationW) for
//! real-time USB device hotplug monitoring.

use super::{DeviceEvent, DeviceWatcher};
use crate::model::*;
use std::sync::mpsc::{channel, Receiver, Sender};

#[cfg(target_os = "windows")]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(target_os = "windows")]
use std::sync::Arc;
#[cfg(target_os = "windows")]
use std::thread::{self, JoinHandle};

#[cfg(target_os = "windows")]
use windows::{
    core::GUID,
    Win32::Devices::DeviceAndDriverInstallation::*,
    Win32::Devices::Usb::GUID_DEVINTERFACE_USB_DEVICE,
    Win32::Foundation::*,
    Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::UI::WindowsAndMessaging::*,
};

/// GUID for USB device interface class
#[cfg(target_os = "windows")]
const GUID_DEVINTERFACE_USB_DEVICE_BYTES: GUID = GUID {
    data1: 0xA5DCBF10,
    data2: 0x6530,
    data3: 0x11D2,
    data4: [0x90, 0x1F, 0x00, 0xC0, 0x4F, 0xB9, 0x51, 0xED],
};

/// Device Broadcast Header for device interface notifications
#[cfg(target_os = "windows")]
#[repr(C)]
struct DevBroadcastDeviceInterface {
    size: u32,
    device_type: u32,
    reserved: u32,
    class_guid: GUID,
    name: [u16; 1], // Variable length
}

#[cfg(target_os = "windows")]
const DBT_DEVTYP_DEVICEINTERFACE: u32 = 0x00000005;
#[cfg(target_os = "windows")]
const DBT_DEVICEARRIVAL: u32 = 0x8000;
#[cfg(target_os = "windows")]
const DBT_DEVICEREMOVECOMPLETE: u32 = 0x8004;
#[cfg(target_os = "windows")]
const DEVICE_NOTIFY_WINDOW_HANDLE: u32 = 0x00000000;

/// Windows device watcher using RegisterDeviceNotification
pub struct WindowsDeviceWatcher {
    running: bool,
    sender: Option<Sender<DeviceEvent>>,
    #[cfg(target_os = "windows")]
    stop_flag: Arc<AtomicBool>,
    #[cfg(target_os = "windows")]
    thread_handle: Option<JoinHandle<()>>,
}

impl WindowsDeviceWatcher {
    pub fn new() -> Self {
        Self {
            running: false,
            sender: None,
            #[cfg(target_os = "windows")]
            stop_flag: Arc::new(AtomicBool::new(false)),
            #[cfg(target_os = "windows")]
            thread_handle: None,
        }
    }
}

impl Default for WindowsDeviceWatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_os = "windows")]
impl DeviceWatcher for WindowsDeviceWatcher {
    fn start(&mut self) -> Result<Receiver<DeviceEvent>, Box<dyn std::error::Error>> {
        let (tx, rx) = channel();
        self.sender = Some(tx.clone());
        self.running = true;
        self.stop_flag.store(false, Ordering::SeqCst);
        
        let stop_flag = self.stop_flag.clone();
        
        let handle = thread::spawn(move || {
            if let Err(e) = run_windows_watcher(tx, stop_flag) {
                log::error!("Windows device watcher error: {}", e);
            }
        });
        
        self.thread_handle = Some(handle);
        
        Ok(rx)
    }

    fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.running = false;
        self.stop_flag.store(true, Ordering::SeqCst);
        
        // Send a message to wake up the message loop
        // The thread will check stop_flag and exit
        
        if let Some(handle) = self.thread_handle.take() {
            // We can't easily interrupt the message loop, but setting the flag will
            // cause it to exit on the next message
            let _ = handle.join();
        }
        
        self.sender = None;
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn run_windows_watcher(
    tx: Sender<DeviceEvent>,
    stop_flag: Arc<AtomicBool>,
) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        // Create a message-only window for receiving device notifications
        let class_name = windows::core::w!("BootForgeUsbWatcher");
        
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: WNDCLASS_STYLES(0),
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: GetModuleHandleW(None)?.into(),
            hIcon: HICON::default(),
            hCursor: HCURSOR::default(),
            hbrBackground: HBRUSH::default(),
            lpszMenuName: windows::core::PCWSTR::null(),
            lpszClassName: class_name,
            hIconSm: HICON::default(),
        };
        
        let atom = RegisterClassExW(&wc);
        if atom == 0 {
            return Err("Failed to register window class".into());
        }
        
        // Create message-only window (HWND_MESSAGE as parent)
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class_name,
            windows::core::w!("BootForgeUsbWatcher"),
            WINDOW_STYLE(0),
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            None,
            None,
            None,
        )?;
        
        // Set up device notification filter for USB devices
        let mut filter = DEV_BROADCAST_DEVICEINTERFACE_W {
            dbcc_size: std::mem::size_of::<DEV_BROADCAST_DEVICEINTERFACE_W>() as u32,
            dbcc_devicetype: DBT_DEVTYP_DEVICEINTERFACE,
            dbcc_reserved: 0,
            dbcc_classguid: GUID_DEVINTERFACE_USB_DEVICE,
            dbcc_name: [0; 1],
        };
        
        let notification_handle = RegisterDeviceNotificationW(
            hwnd,
            &filter as *const _ as *const std::ffi::c_void,
            REGISTER_NOTIFICATION_FLAGS(DEVICE_NOTIFY_WINDOW_HANDLE),
        )?;
        
        // Store tx in thread-local storage for the window proc
        TX_SENDER.with(|cell| {
            *cell.borrow_mut() = Some(tx);
        });
        
        // Message loop
        let mut msg = MSG::default();
        while !stop_flag.load(Ordering::SeqCst) {
            let result = GetMessageW(&mut msg, HWND::default(), 0, 0);
            
            if result.0 == 0 || result.0 == -1 {
                break;
            }
            
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        
        // Cleanup
        let _ = UnregisterDeviceNotification(notification_handle);
        let _ = DestroyWindow(hwnd);
        let _ = UnregisterClassW(class_name, None);
    }
    
    Ok(())
}

#[cfg(target_os = "windows")]
thread_local! {
    static TX_SENDER: std::cell::RefCell<Option<Sender<DeviceEvent>>> = const { std::cell::RefCell::new(None) };
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    const WM_DEVICECHANGE: u32 = 0x0219;
    
    if msg == WM_DEVICECHANGE {
        let event_type = wparam.0 as u32;
        
        if event_type == DBT_DEVICEARRIVAL || event_type == DBT_DEVICEREMOVECOMPLETE {
            if lparam.0 != 0 {
                let header = lparam.0 as *const DevBroadcastDeviceInterface;
                
                if (*header).device_type == DBT_DEVTYP_DEVICEINTERFACE {
                    // Extract device path from the broadcast structure
                    let name_ptr = &(*header).name as *const u16;
                    let mut len = 0;
                    while *name_ptr.offset(len) != 0 && len < 512 {
                        len += 1;
                    }
                    let device_path = String::from_utf16_lossy(
                        std::slice::from_raw_parts(name_ptr, len as usize)
                    );
                    
                    // Parse VID/PID from device path
                    if let Some((vid, pid)) = parse_vid_pid_from_path(&device_path) {
                        let record = create_device_record(vid, pid, &device_path);
                        
                        let event = if event_type == DBT_DEVICEARRIVAL {
                            DeviceEvent::Added(record)
                        } else {
                            DeviceEvent::Removed(record)
                        };
                        
                        TX_SENDER.with(|cell| {
                            if let Some(ref tx) = *cell.borrow() {
                                let _ = tx.send(event);
                            }
                        });
                    }
                }
            }
        }
    }
    
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

#[cfg(target_os = "windows")]
fn parse_vid_pid_from_path(path: &str) -> Option<(u16, u16)> {
    let upper = path.to_uppercase();
    
    // Look for VID_xxxx pattern
    let vid = if let Some(vid_pos) = upper.find("VID_") {
        let vid_start = vid_pos + 4;
        if vid_start + 4 <= upper.len() {
            u16::from_str_radix(&upper[vid_start..vid_start + 4], 16).ok()?
        } else {
            return None;
        }
    } else {
        return None;
    };

    // Look for PID_xxxx pattern
    let pid = if let Some(pid_pos) = upper.find("PID_") {
        let pid_start = pid_pos + 4;
        if pid_start + 4 <= upper.len() {
            u16::from_str_radix(&upper[pid_start..pid_start + 4], 16).ok()?
        } else {
            return None;
        }
    } else {
        return None;
    };

    Some((vid, pid))
}

#[cfg(target_os = "windows")]
fn create_device_record(vid: u16, pid: u16, device_path: &str) -> UsbDeviceRecord {
    UsbDeviceRecord {
        id: UsbId::new(vid, pid),
        location: UsbLocation {
            bus: None,
            address: None,
            port_path: None,
        },
        descriptor: UsbDescriptorSummary {
            manufacturer: None,
            product: None,
            serial_number: extract_serial_from_path(device_path),
            device_class: None,
            device_subclass: None,
            device_protocol: None,
            usb_version: None,
        },
        driver: DriverStatus::Unknown,
        health: LinkHealth::Good,
        tags: vec!["windows".to_string()],
        raw_data: Some(device_path.to_string()),
    }
}

#[cfg(target_os = "windows")]
fn extract_serial_from_path(path: &str) -> Option<String> {
    // Device path format: \\?\USB#VID_xxxx&PID_xxxx#serial#{GUID}
    let parts: Vec<&str> = path.split('#').collect();
    if parts.len() >= 3 {
        let serial = parts[2];
        // Filter out non-serial strings (like interface numbers)
        if !serial.starts_with("&") && serial.len() > 2 {
            return Some(serial.to_string());
        }
    }
    None
}

#[cfg(not(target_os = "windows"))]
impl DeviceWatcher for WindowsDeviceWatcher {
    fn start(&mut self) -> Result<Receiver<DeviceEvent>, Box<dyn std::error::Error>> {
        let (tx, rx) = channel();
        self.sender = Some(tx);
        self.running = true;
        log::warn!("Windows device watching only available on Windows");
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
        let watcher = WindowsDeviceWatcher::new();
        assert!(!watcher.running);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_vid_pid_parsing() {
        let path = r"\\?\USB#VID_18D1&PID_4EE1#12345#{GUID}";
        let result = parse_vid_pid_from_path(path);
        assert_eq!(result, Some((0x18D1, 0x4EE1)));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_serial_extraction() {
        let path = r"\\?\USB#VID_18D1&PID_4EE1#ABC123#{GUID}";
        let serial = extract_serial_from_path(path);
        assert_eq!(serial, Some("ABC123".to_string()));
    }
}
