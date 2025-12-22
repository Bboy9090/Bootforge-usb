use crate::model::UsbDeviceRecord;

/// Check if a device is an ADB device
pub fn is_adb_device(device: &UsbDeviceRecord) -> bool {
    // ADB devices typically use class 0xFF (Vendor Specific)
    // Common ADB vendor IDs include Google (0x18d1), Samsung (0x04e8), etc.
    
    // Check for Google ADB interface
    if device.id.vid == 0x18d1 {
        // Google vendor ID with various ADB product IDs
        if matches!(device.id.pid, 0x4ee1..=0x4ee7) {
            return true;
        }
    }

    // Check for Samsung ADB
    if device.id.vid == 0x04e8 {
        // Samsung devices in ADB mode often use specific PIDs
        if device.id.pid == 0x6860 || device.id.pid == 0x6864 {
            return true;
        }
    }

    // Check class code for vendor-specific (often used by ADB)
    if let Some(class) = device.descriptor.device_class {
        if class == 0xFF {
            // Check if it has "adb" in the product name
            if let Some(ref product) = device.descriptor.product {
                if product.to_lowercase().contains("adb") {
                    return true;
                }
            }
        }
    }

    // Check tags
    device.has_tag("adb")
}
