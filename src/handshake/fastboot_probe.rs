use crate::model::UsbDeviceRecord;

/// Check if a device is a Fastboot device
pub fn is_fastboot_device(device: &UsbDeviceRecord) -> bool {
    // Fastboot devices typically use class 0xFF (Vendor Specific)
    // Common Fastboot vendor IDs include Google (0x18d1)
    
    // Check for Google Fastboot interface
    if device.id.vid == 0x18d1 {
        // Google fastboot PIDs
        if matches!(device.id.pid, 0x4ee0 | 0xd00d | 0x0d02) {
            return true;
        }
    }

    // Check for Qualcomm EDL/Fastboot mode
    if device.id.vid == 0x05c6 && device.id.pid == 0x9008 {
        return true;
    }

    // Check product string
    if let Some(ref product) = device.descriptor.product {
        let product_lower = product.to_lowercase();
        if product_lower.contains("fastboot") || product_lower.contains("bootloader") {
            return true;
        }
    }

    // Check tags
    device.has_tag("fastboot")
}
