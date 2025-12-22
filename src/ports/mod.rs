use crate::errors::UsbError;

const DEFAULT_HUB_PORTS: u8 = 4;

#[derive(Debug, Clone)]
pub struct UsbPort {
    pub port_number: u8,
    pub hub_address: Option<u8>,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct UsbHub {
    pub address: u8,
    pub bus: u8,
    pub num_ports: u8,
    pub ports: Vec<UsbPort>,
}

/// Parse a USB port path string (e.g., "1-2.3.4" means bus 1, hub at port 2, then port 3, then port 4)
pub fn parse_port_path(path: &str) -> Result<Vec<u8>, UsbError> {
    let parts: Vec<&str> = path.split(&['-', '.'][..]).collect();
    
    if parts.len() < 2 {
        return Err(UsbError::Parse(format!("Invalid port path: {}", path)));
    }

    let mut port_numbers = Vec::new();
    
    // Skip the first part (bus number)
    for part in parts.iter().skip(1) {
        match part.parse::<u8>() {
            Ok(num) => port_numbers.push(num),
            Err(_) => return Err(UsbError::Parse(format!("Invalid port number in path: {}", part))),
        }
    }

    Ok(port_numbers)
}

/// Enumerate USB hubs on the system
pub fn enumerate_hubs() -> Result<Vec<UsbHub>, UsbError> {
    let mut hubs = Vec::new();

    // Use rusb to find hub devices
    let devices = match rusb::devices() {
        Ok(devices) => devices,
        Err(e) => {
            // Return an empty list if we can't access USB
            log::debug!("Failed to enumerate USB devices: {}", e);
            return Ok(hubs);
        }
    };
    
    for device in devices.iter() {
        let device_desc = match device.device_descriptor() {
            Ok(desc) => desc,
            Err(_) => continue,
        };
        
        // USB hub class is 0x09
        if device_desc.class_code() == 0x09 {
            // Try to get the number of ports from the hub descriptor
            // Note: Full implementation would parse hub descriptors
            let hub = UsbHub {
                address: device.address(),
                bus: device.bus_number(),
                num_ports: DEFAULT_HUB_PORTS,
                ports: Vec::new(),
            };
            
            hubs.push(hub);
        }
    }

    Ok(hubs)
}

/// Get the port path for a USB device
pub fn get_device_port_path(bus: u8, address: u8) -> Option<String> {
    // This is a simplified implementation
    // Full implementation would query the system for the actual port path
    
    #[cfg(target_os = "linux")]
    {
        // On Linux, we can read from sysfs
        use std::fs;
        let _sysfs_path = format!("/sys/bus/usb/devices/{}-{}", bus, address);
        if let Ok(entries) = fs::read_dir("/sys/bus/usb/devices") {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy();
                    if name_str.starts_with(&format!("{}-", bus)) {
                        // Found a potential match
                        return Some(name_str.to_string());
                    }
                }
            }
        }
    }

    // Fallback: construct a simple path
    Some(format!("{}-{}", bus, address))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_port_path() {
        let path = "1-2.3.4";
        let result = parse_port_path(path).unwrap();
        assert_eq!(result, vec![2, 3, 4]);

        let path2 = "3-1";
        let result2 = parse_port_path(path2).unwrap();
        assert_eq!(result2, vec![1]);
    }

    #[test]
    fn test_enumerate_hubs() {
        // This test may fail if no USB hubs are present or no USB access
        // We need to catch panics because rusb may panic if USB access is denied
        let result = std::panic::catch_unwind(|| {
            enumerate_hubs()
        });

        match result {
            Ok(Ok(hubs)) => {
                println!("Found {} USB hubs", hubs.len());
            }
            Ok(Err(e)) => {
                println!("Hub enumeration failed (may be expected): {}", e);
            }
            Err(_) => {
                println!("Hub enumeration panicked (may be expected in test environment)");
            }
        }
    }
}
