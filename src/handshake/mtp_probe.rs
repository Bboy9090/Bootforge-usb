use crate::model::UsbDeviceRecord;

/// Check if a device is an MTP (Media Transfer Protocol) device
pub fn is_mtp_device(device: &UsbDeviceRecord) -> bool {
    // MTP devices typically use class 0x06 (Imaging) or 0xFF (Vendor Specific)
    // with specific subclass and protocol values
    
    // Check for MTP class code (6) with subclass 1 and protocol 1
    if let Some(class) = device.descriptor.device_class {
        if class == 0x06 {
            if let Some(subclass) = device.descriptor.device_subclass {
                if let Some(protocol) = device.descriptor.device_protocol {
                    if subclass == 0x01 && protocol == 0x01 {
                        return true;
                    }
                }
            }
        }
    }

    // Check product string for MTP keywords
    if let Some(ref product) = device.descriptor.product {
        let product_lower = product.to_lowercase();
        if product_lower.contains("mtp") || product_lower.contains("media transfer") {
            return true;
        }
    }

    // Many Android devices in MTP mode
    if let Some(ref manufacturer) = device.descriptor.manufacturer {
        let manufacturer_lower = manufacturer.to_lowercase();
        if manufacturer_lower.contains("android") {
            if let Some(class) = device.descriptor.device_class {
                // Android MTP devices often use vendor-specific class
                if class == 0x00 || class == 0xFF {
                    return true;
                }
            }
        }
    }

    // Check tags
    device.has_tag("mtp")
}
