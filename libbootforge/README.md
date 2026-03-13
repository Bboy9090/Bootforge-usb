# libbootforge

libbootforge is a low-level USB device detection library designed for hardware discovery, repair workflows, and device preparation.

## Features

The library provides structured access to USB device information including:

- **Vendor and product identifiers** - VID/PID for device identification
- **Device descriptors** - USB device, configuration, and string descriptors
- **Device mode detection** - Automatic detection of DFU, Recovery, and other special modes
- **Device connection events** - Monitor for USB device connect/disconnect events

libbootforge serves as the USB hardware discovery layer for the Bobby's Workshop device ecosystem.

## Architecture

libbootforge is the foundation layer in the device ecosystem:

```
Bobby's Workshop Platform
        │
        ▼
PhoenixCore
        │
        ▼
libbootforge
        │
        ├── USB detection
        ├── descriptor reading
        ├── device mode detection
        └── device event monitoring
```

## Usage

### Basic Device Enumeration

```rust
use libbootforge::enumerate_devices;

// Enumerate all connected USB devices
let devices = enumerate_devices()?;

for device in devices {
    println!("Device: {:04x}:{:04x}", device.vendor_id, device.product_id);
    if let Some(name) = device.product_name {
        println!("  Name: {}", name);
    }
    println!("  Mode: {:?}", device.device_mode);
}
```

### Device Mode Detection

```rust
use libbootforge::{DeviceMode, enumerate_devices};

let devices = enumerate_devices()?;

for device in devices {
    if device.device_mode.is_special_mode() {
        println!("Device in special mode: {:?}", device.device_mode);
    }
}
```

### Event Monitoring

```rust
use libbootforge::DeviceEventMonitor;
use std::time::Duration;

let mut monitor = DeviceEventMonitor::new()?;

loop {
    let events = monitor.wait_for_events(Duration::from_secs(1))?;

    for event in events {
        match event {
            DeviceEvent::Connected(device) => {
                println!("Device connected: {:04x}:{:04x}",
                    device.vendor_id, device.product_id);
            },
            DeviceEvent::Disconnected { vendor_id, product_id, .. } => {
                println!("Device disconnected: {:04x}:{:04x}",
                    vendor_id, product_id);
            },
        }
    }
}
```

## Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| Linux | ✅ Supported | Includes sysfs path enrichment |
| macOS | ✅ Supported | IOKit enrichment planned |
| Windows | ✅ Supported | SetupAPI enrichment planned |

## Compliance & Safety

libbootforge is **read-only**:
- ✅ Reads device descriptors
- ✅ Reads string descriptors
- ✅ Enumerates devices
- ❌ Does NOT modify devices
- ❌ Does NOT execute exploits
- ❌ Does NOT bypass security

## Dependencies

- `rusb` - Cross-platform USB library (libusb 1.0 wrapper)
- `serde` - Serialization support
- `thiserror` - Error handling

## Testing

```bash
# Run non-USB tests (work in CI)
cargo test --package libbootforge

# Run all tests including USB tests (requires USB hardware access)
cargo test --package libbootforge -- --ignored --test-threads=1
```

Note: Tests requiring USB hardware access are marked with `#[ignore]` to allow CI testing.

## License

MIT OR Apache-2.0
