//! USB ID Database - OMEGA MODE
//!
//! Provides vendor and product name lookups for USB devices.
//! Includes a built-in database of common devices and the ability
//! to load the full USB ID database.

use std::collections::HashMap;
use std::sync::OnceLock;

/// Static database instance
static USB_DATABASE: OnceLock<UsbDatabase> = OnceLock::new();

/// Get the global USB database
pub fn database() -> &'static UsbDatabase {
    USB_DATABASE.get_or_init(UsbDatabase::builtin)
}

/// USB vendor entry
#[derive(Debug, Clone)]
pub struct Vendor {
    /// Vendor ID
    pub id: u16,
    /// Vendor name
    pub name: String,
    /// Products from this vendor
    pub products: HashMap<u16, String>,
}

/// USB ID Database
#[derive(Debug, Clone)]
pub struct UsbDatabase {
    vendors: HashMap<u16, Vendor>,
    classes: HashMap<u8, ClassInfo>,
}

/// USB class information
#[derive(Debug, Clone)]
pub struct ClassInfo {
    pub name: String,
    pub subclasses: HashMap<u8, SubclassInfo>,
}

/// USB subclass information
#[derive(Debug, Clone)]
pub struct SubclassInfo {
    pub name: String,
    pub protocols: HashMap<u8, String>,
}

impl UsbDatabase {
    /// Create a new empty database
    pub fn new() -> Self {
        Self {
            vendors: HashMap::new(),
            classes: HashMap::new(),
        }
    }
    
    /// Create database with built-in entries
    pub fn builtin() -> Self {
        let mut db = Self::new();
        db.load_builtin_vendors();
        db.load_builtin_classes();
        db
    }
    
    /// Load built-in vendor database
    fn load_builtin_vendors(&mut self) {
        // Major vendors with common products
        let vendors = [
            (0x05AC, "Apple Inc.", vec![
                (0x12A8, "iPhone"),
                (0x12AB, "iPad"),
                (0x8600, "iPhone 5/5C/5S/6/SE"),
                (0x8005, "OHCI Root Hub"),
                (0x1006, "Hub in Aluminum Keyboard"),
            ]),
            (0x18D1, "Google Inc.", vec![
                (0x4EE1, "Nexus/Pixel (MTP)"),
                (0x4EE2, "Nexus/Pixel (MTP+ADB)"),
                (0xD001, "Pixel (Debug)"),
                (0x4EE0, "Nexus/Pixel (Fastboot)"),
                (0x2D00, "Android Accessory"),
                (0x2D01, "Android Accessory + ADB"),
            ]),
            (0x04E8, "Samsung Electronics Co., Ltd", vec![
                (0x6860, "Galaxy series (MTP)"),
                (0x6865, "Galaxy series (MTP+ADB)"),
                (0x6864, "Galaxy series (PTP)"),
            ]),
            (0x1D6B, "Linux Foundation", vec![
                (0x0001, "1.1 root hub"),
                (0x0002, "2.0 root hub"),
                (0x0003, "3.0 root hub"),
            ]),
            (0x8087, "Intel Corp.", vec![
                (0x0024, "Integrated Rate Matching Hub"),
                (0x0026, "Integrated Rate Matching Hub"),
                (0x0A2B, "Bluetooth Wireless"),
                (0x0AAA, "Bluetooth 9460/9560"),
            ]),
            (0x046D, "Logitech, Inc.", vec![
                (0xC077, "M105 Optical Mouse"),
                (0xC52B, "Unifying Receiver"),
                (0xC534, "Unifying Receiver"),
                (0x0825, "Webcam C270"),
                (0x082D, "HD Pro Webcam C920"),
                (0xC216, "Dual Action Gamepad"),
            ]),
            (0x045E, "Microsoft Corp.", vec![
                (0x028E, "Xbox360 Controller"),
                (0x02EA, "Xbox One S Controller"),
                (0x0B12, "Xbox Series X Controller"),
                (0x0745, "Nano Transceiver"),
                (0x07A5, "Wireless Receiver 1461C"),
            ]),
            (0x054C, "Sony Corp.", vec![
                (0x05C4, "DualShock 4 [CUH-ZCT1x]"),
                (0x09CC, "DualShock 4 [CUH-ZCT2x]"),
                (0x0CE6, "DualSense Controller"),
                (0x0DF2, "DualSense Edge"),
            ]),
            (0x1050, "Yubico.com", vec![
                (0x0010, "YubiKey 1/2"),
                (0x0110, "YubiKey NEO(-N) OTP"),
                (0x0111, "YubiKey NEO(-N) CCID"),
                (0x0401, "YubiKey 4/5 OTP"),
                (0x0402, "YubiKey 4/5 FIDO"),
                (0x0403, "YubiKey 4/5 OTP+FIDO"),
                (0x0404, "YubiKey 4/5 CCID"),
                (0x0405, "YubiKey 4/5 OTP+CCID"),
                (0x0406, "YubiKey 4/5 FIDO+CCID"),
                (0x0407, "YubiKey 4/5 OTP+FIDO+CCID"),
            ]),
            (0x2341, "Arduino SA", vec![
                (0x0001, "Uno (CDC ACM)"),
                (0x0010, "Mega 2560 (CDC ACM)"),
                (0x0036, "Leonardo"),
                (0x0042, "Uno Rev3 (CDC ACM)"),
                (0x0043, "Uno SMD (CDC ACM)"),
                (0x8036, "Leonardo (CDC ACM, HID)"),
            ]),
            (0x10C4, "Silicon Labs", vec![
                (0xEA60, "CP210x UART Bridge"),
                (0xEA70, "CP210x UART Bridge (VCP)"),
            ]),
            (0x0403, "Future Technology Devices International, Ltd", vec![
                (0x6001, "FT232 USB-Serial (UART) IC"),
                (0x6010, "FT2232C/D/H Dual UART/FIFO IC"),
                (0x6011, "FT4232H Quad HS USB-UART/FIFO IC"),
                (0x6014, "FT232H Single HS USB-UART/FIFO IC"),
                (0x6015, "FT-X Series"),
            ]),
            (0x1A86, "QinHeng Electronics", vec![
                (0x7523, "CH340 Serial"),
                (0x5523, "CH341 Serial"),
                (0x55D4, "CH9102 Serial"),
            ]),
            (0x2E8A, "Raspberry Pi", vec![
                (0x0003, "RP2040 Boot"),
                (0x0004, "Picoprobe (CMSIS-DAP)"),
                (0x0005, "Pico MicroPython"),
                (0x000A, "Pico SDK CDC UART"),
            ]),
            (0x239A, "Adafruit", vec![
                (0x8019, "Feather M0 Express"),
                (0x801D, "Metro M0 Express"),
                (0x80CB, "Feather RP2040"),
            ]),
            (0x303A, "Espressif", vec![
                (0x1001, "ESP32-S2"),
                (0x0002, "ESP32-S3"),
            ]),
            (0x1366, "SEGGER", vec![
                (0x0101, "J-Link"),
                (0x0105, "J-Link Plus"),
            ]),
            (0x0483, "STMicroelectronics", vec![
                (0x3748, "ST-LINK/V2"),
                (0x374B, "ST-LINK/V2.1 (Nucleo-F103RB)"),
                (0x374E, "STLINK-V3"),
                (0xDF11, "STM32 DFU Bootloader"),
            ]),
            (0x0D28, "ARM Ltd", vec![
                (0x0204, "DAPLink CMSIS-DAP"),
            ]),
            (0x1209, "Generic", vec![
                (0x0001, "pid.codes Test PID"),
            ]),
            (0x2109, "VIA Labs, Inc.", vec![
                (0x3431, "Hub"),
                (0x2817, "USB2.0 Hub"),
            ]),
            (0x05E3, "Genesys Logic, Inc.", vec![
                (0x0608, "Hub"),
                (0x0610, "4-port hub"),
            ]),
            (0x0781, "SanDisk Corp.", vec![
                (0x5567, "Cruzer Blade"),
                (0x5581, "Ultra"),
            ]),
            (0x090C, "Silicon Motion, Inc. - Taiwan", vec![
                (0x1000, "Flash Drive"),
            ]),
            (0x0951, "Kingston Technology", vec![
                (0x1666, "DataTraveler 100 G3"),
            ]),
            (0x8086, "Intel Corporation", vec![]),
            (0x1002, "Advanced Micro Devices, Inc. [AMD/ATI]", vec![]),
            (0x10DE, "NVIDIA Corporation", vec![]),
        ];
        
        for (vid, name, products) in vendors {
            let mut vendor = Vendor {
                id: vid,
                name: name.to_string(),
                products: HashMap::new(),
            };
            for (pid, pname) in products {
                vendor.products.insert(pid, pname.to_string());
            }
            self.vendors.insert(vid, vendor);
        }
    }
    
    /// Load built-in class database
    fn load_builtin_classes(&mut self) {
        let classes = [
            (0x00, "Device", vec![
                (0x00, "Use class information in the Interface Descriptors", vec![]),
            ]),
            (0x01, "Audio", vec![
                (0x01, "Control Device", vec![]),
                (0x02, "Streaming", vec![]),
                (0x03, "MIDI Streaming", vec![]),
            ]),
            (0x02, "Communications and CDC Control", vec![
                (0x01, "Direct Line Control Model", vec![]),
                (0x02, "Abstract Control Model", vec![(0x01, "AT Commands V.250")]),
                (0x06, "Ethernet Networking Control Model", vec![]),
                (0x0D, "Network Control Model", vec![]),
            ]),
            (0x03, "HID (Human Interface Device)", vec![
                (0x00, "No Subclass", vec![(0x00, "None"), (0x01, "Keyboard"), (0x02, "Mouse")]),
                (0x01, "Boot Interface Subclass", vec![(0x01, "Keyboard"), (0x02, "Mouse")]),
            ]),
            (0x05, "Physical Interface Device", vec![]),
            (0x06, "Still Image Capture", vec![
                (0x01, "Still Image Capture Device", vec![(0x01, "Picture Transfer Protocol (PTP)")]),
            ]),
            (0x07, "Printer", vec![
                (0x01, "Printer", vec![
                    (0x01, "Unidirectional"),
                    (0x02, "Bidirectional"),
                    (0x03, "IEEE 1284.4 compatible bidirectional"),
                ]),
            ]),
            (0x08, "Mass Storage", vec![
                (0x01, "RBC", vec![]),
                (0x02, "SFF-8020i, MMC-2 (ATAPI)", vec![]),
                (0x04, "UFI", vec![]),
                (0x05, "SFF-8070i", vec![]),
                (0x06, "SCSI transparent command set", vec![
                    (0x00, "CBI without interrupt"),
                    (0x01, "CBI with interrupt"),
                    (0x50, "Bulk-Only"),
                    (0x62, "UAS"),
                ]),
            ]),
            (0x09, "Hub", vec![
                (0x00, "Full Speed Hub", vec![]),
                (0x01, "Single TT", vec![]),
                (0x02, "Multiple TT", vec![]),
            ]),
            (0x0A, "CDC-Data", vec![]),
            (0x0B, "Smart Card", vec![]),
            (0x0D, "Content Security", vec![]),
            (0x0E, "Video", vec![
                (0x01, "Video Control", vec![]),
                (0x02, "Video Streaming", vec![]),
                (0x03, "Video Interface Collection", vec![]),
            ]),
            (0x0F, "Personal Healthcare", vec![]),
            (0x10, "Audio/Video Devices", vec![]),
            (0x11, "Billboard Device Class", vec![]),
            (0x12, "USB Type-C Bridge Class", vec![]),
            (0xDC, "Diagnostic Device", vec![]),
            (0xE0, "Wireless Controller", vec![
                (0x01, "Radio Frequency", vec![(0x01, "Bluetooth")]),
            ]),
            (0xEF, "Miscellaneous Device", vec![
                (0x01, "Common Class", vec![(0x01, "Microsoft ActiveSync")]),
                (0x02, "Common Class", vec![(0x01, "Interface Association Descriptor")]),
            ]),
            (0xFE, "Application Specific", vec![
                (0x01, "Device Firmware Upgrade", vec![]),
                (0x02, "IRDA Bridge", vec![]),
                (0x03, "USB Test and Measurement", vec![]),
            ]),
            (0xFF, "Vendor Specific", vec![]),
        ];
        
        for (class_id, class_name, subclasses) in classes {
            let mut class_info = ClassInfo {
                name: class_name.to_string(),
                subclasses: HashMap::new(),
            };
            
            for (subclass_id, subclass_name, protocols) in subclasses {
                let mut subclass_info = SubclassInfo {
                    name: subclass_name.to_string(),
                    protocols: HashMap::new(),
                };
                
                for (protocol_id, protocol_name) in protocols {
                    subclass_info.protocols.insert(protocol_id, protocol_name.to_string());
                }
                
                class_info.subclasses.insert(subclass_id, subclass_info);
            }
            
            self.classes.insert(class_id, class_info);
        }
    }
    
    /// Look up vendor name
    pub fn vendor_name(&self, vid: u16) -> Option<&str> {
        self.vendors.get(&vid).map(|v| v.name.as_str())
    }
    
    /// Look up product name
    pub fn product_name(&self, vid: u16, pid: u16) -> Option<&str> {
        self.vendors.get(&vid)
            .and_then(|v| v.products.get(&pid))
            .map(|s| s.as_str())
    }
    
    /// Look up class name
    pub fn class_name(&self, class: u8) -> Option<&str> {
        self.classes.get(&class).map(|c| c.name.as_str())
    }
    
    /// Look up subclass name
    pub fn subclass_name(&self, class: u8, subclass: u8) -> Option<&str> {
        self.classes.get(&class)
            .and_then(|c| c.subclasses.get(&subclass))
            .map(|s| s.name.as_str())
    }
    
    /// Look up protocol name
    pub fn protocol_name(&self, class: u8, subclass: u8, protocol: u8) -> Option<&str> {
        self.classes.get(&class)
            .and_then(|c| c.subclasses.get(&subclass))
            .and_then(|s| s.protocols.get(&protocol))
            .map(|p| p.as_str())
    }
    
    /// Get full class description
    pub fn class_description(&self, class: u8, subclass: u8, protocol: u8) -> String {
        let class_name = self.class_name(class).unwrap_or("Unknown");
        let subclass_name = self.subclass_name(class, subclass);
        let protocol_name = self.protocol_name(class, subclass, protocol);
        
        match (subclass_name, protocol_name) {
            (Some(s), Some(p)) => format!("{} / {} / {}", class_name, s, p),
            (Some(s), None) => format!("{} / {}", class_name, s),
            _ => class_name.to_string(),
        }
    }
    
    /// Get device description from VID/PID
    pub fn device_description(&self, vid: u16, pid: u16) -> String {
        let vendor = self.vendor_name(vid).unwrap_or("Unknown Vendor");
        let product = self.product_name(vid, pid);
        
        match product {
            Some(p) => format!("{} {}", vendor, p),
            None => format!("{} (Product {:04X})", vendor, pid),
        }
    }
    
    /// Add a vendor to the database
    pub fn add_vendor(&mut self, vid: u16, name: &str) {
        self.vendors.entry(vid).or_insert_with(|| Vendor {
            id: vid,
            name: name.to_string(),
            products: HashMap::new(),
        });
    }
    
    /// Add a product to the database
    pub fn add_product(&mut self, vid: u16, pid: u16, name: &str) {
        if let Some(vendor) = self.vendors.get_mut(&vid) {
            vendor.products.insert(pid, name.to_string());
        }
    }
    
    /// Number of vendors in database
    pub fn vendor_count(&self) -> usize {
        self.vendors.len()
    }
    
    /// Number of products in database
    pub fn product_count(&self) -> usize {
        self.vendors.values().map(|v| v.products.len()).sum()
    }
}

impl Default for UsbDatabase {
    fn default() -> Self {
        Self::builtin()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_database() {
        let db = UsbDatabase::builtin();
        assert!(db.vendor_count() > 0);
        assert!(db.product_count() > 0);
    }

    #[test]
    fn test_vendor_lookup() {
        let db = database();
        assert_eq!(db.vendor_name(0x05AC), Some("Apple Inc."));
        assert_eq!(db.vendor_name(0x18D1), Some("Google Inc."));
    }

    #[test]
    fn test_product_lookup() {
        let db = database();
        assert!(db.product_name(0x18D1, 0x4EE0).is_some());
    }

    #[test]
    fn test_class_lookup() {
        let db = database();
        assert_eq!(db.class_name(0x03), Some("HID (Human Interface Device)"));
        assert_eq!(db.class_name(0x08), Some("Mass Storage"));
    }

    #[test]
    fn test_device_description() {
        let db = database();
        let desc = db.device_description(0x05AC, 0x12A8);
        assert!(desc.contains("Apple"));
        assert!(desc.contains("iPhone"));
    }
}
