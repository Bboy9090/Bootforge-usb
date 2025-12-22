use crate::model::UsbDeviceRecord;

/// Check if a device is an Apple device (iPhone, iPad, iPod)
pub fn is_apple_device(device: &UsbDeviceRecord) -> bool {
    // Apple's vendor ID
    const APPLE_VENDOR_ID: u16 = 0x05ac;

    if device.id.vid != APPLE_VENDOR_ID {
        return false;
    }

    // Common Apple device PIDs for iPhones, iPads, and iPods
    // This is not exhaustive but covers many common models
    let apple_device_pids = [
        0x1290, 0x1291, 0x1292, 0x1293, // iPhone
        0x12a0, 0x12a1, 0x12a2, 0x12a3, // iPad
        0x1294, 0x1297, 0x129a, 0x129c, // iPod
        0x12ab, 0x12ac, // Various iOS devices
    ];

    if apple_device_pids.contains(&device.id.pid) {
        return true;
    }

    // Check manufacturer string
    if let Some(ref manufacturer) = device.descriptor.manufacturer {
        if manufacturer.to_lowercase().contains("apple") {
            return true;
        }
    }

    // Check product string for common Apple device names
    if let Some(ref product) = device.descriptor.product {
        let product_lower = product.to_lowercase();
        if product_lower.contains("iphone") 
            || product_lower.contains("ipad") 
            || product_lower.contains("ipod") {
            return true;
        }
    }

    // Check tags
    device.has_tag("apple")
}
