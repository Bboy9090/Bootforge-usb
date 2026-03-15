//! Example: List all USB devices in human-readable format

use libbootforge::scan_devices;

fn main() {
    match scan_devices() {
        Ok(devices) => {
            if devices.is_empty() {
                println!("No USB devices found.");
                return;
            }

            println!("USB Devices Found:\n");
            for device in devices {
                println!("Bus {:03} Device {:03} ID {:04x}:{:04x}",
                    device.bus_number,
                    device.address,
                    device.vendor_id,
                    device.product_id
                );
                println!("  Vendor ID              : {:04x}", device.vendor_id);
                println!("  Product ID             : {:04x}", device.product_id);
                println!("  Vendor Name            : {}",
                    device.vendor_name.as_deref().unwrap_or("Unknown")
                );
                println!("  Manufacturer           : {}",
                    device.manufacturer.as_deref().unwrap_or("Unknown")
                );
                println!("  Product                : {}",
                    device.product_name.as_deref().unwrap_or("Unknown")
                );
                println!("  Serial                 : {}",
                    device.serial_number.as_deref().unwrap_or("Unknown")
                );
                println!("  Platform               : {:?}", device.platform);
                println!("  Transport              : {:?}", device.transport);
                println!("  Mode                   : {:?}", device.mode);
                println!("  Fingerprint Family     : {:?}", device.fingerprint.family);
                println!("  Model Hint             : {}",
                    device.fingerprint.model_hint.as_deref().unwrap_or("Unknown")
                );
                println!("  Fingerprint Confidence : {:?}", device.fingerprint.confidence);
                println!("  Recommended Workflow   : {:?}", device.recommended_workflow);
                println!();
            }
        }
        Err(e) => {
            eprintln!("Error scanning devices: {}", e);
            std::process::exit(1);
        }
    }
}
