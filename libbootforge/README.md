# libbootforge

libbootforge is a low-level USB device detection library designed for hardware discovery, repair workflows, and device preparation.

## Features

The library provides structured access to USB device information including:

- **USB device scanning** - Enumerate all connected USB devices
- **Vendor and product identifiers** - VID/PID for device identification
- **Device descriptors** - USB device, configuration, and string descriptors
- **Device mode detection** - Automatic detection of DFU, Recovery, Fastboot, ADB, and other special modes
- **Platform classification** - Apple, Android, Generic USB device categorization
- **Transport detection** - USB 2.0 vs USB 3.0 identification
- **Device fingerprinting** - Identify device family (iPhone, iPad, Android Phone, etc.) with confidence levels
- **Workflow recommendations** - Suggest appropriate repair/inspection workflows
- **Device connection events** - Monitor for USB device connect/disconnect events
- **JSON output** - Export device information in JSON format
- **CLI tool** - Comprehensive command-line interface with filtering and watch mode

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
        ├── device fingerprinting
        ├── workflow recommendations
        └── device event monitoring
```

## Usage

### Library API

#### Basic Device Scanning

```rust
use libbootforge::scan_devices;

// Scan all connected USB devices
let devices = scan_devices()?;

for device in devices {
    println!("Device: {:04x}:{:04x}", device.vendor_id, device.product_id);
    println!("  Platform: {:?}", device.platform);
    println!("  Mode: {:?}", device.mode);
    println!("  Family: {:?}", device.fingerprint.family);
    println!("  Workflow: {:?}", device.recommended_workflow);
}
```

#### JSON Output

```rust
use libbootforge::scan_devices_json;

// Get device list as JSON
let json = scan_devices_json()?;
println!("{}", json);
```

#### Device Mode Detection

```rust
use libbootforge::scan_devices;

let devices = scan_devices()?;

for device in devices {
    match device.mode {
        DeviceMode::Recovery => println!("Device in recovery mode"),
        DeviceMode::Dfu => println!("Device in DFU mode"),
        DeviceMode::Fastboot => println!("Device in fastboot mode"),
        DeviceMode::Adb => println!("Device in ADB mode"),
        _ => {}
    }
}
```

#### Event Monitoring

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

### CLI Usage

The `bootforge-cli` tool provides command-line access to all library features:

#### Basic Usage

```bash
# List all USB devices
bootforge-cli

# Output in JSON format
bootforge-cli --json

# Write JSON to file
bootforge-cli --json-file devices.json
```

#### Filtering

```bash
# Show only Apple devices
bootforge-cli --apple

# Show only Android devices
bootforge-cli --android

# Filter by device mode
bootforge-cli --mode recovery
bootforge-cli --mode dfu

# Filter by vendor ID (hex)
bootforge-cli --vendor 05ac
```

#### Watch Mode

```bash
# Monitor for device connections/disconnections in real-time
bootforge-cli --watch

# Watch for Apple devices only
bootforge-cli --watch --apple

# Watch for devices in recovery mode
bootforge-cli --watch --mode recovery
```

#### Examples

```bash
# List all devices in human-readable format
cargo run --example list_devices

# List all devices in JSON format
cargo run --example list_devices_json

# Run CLI tool
cargo run --bin bootforge-cli

# Filter for Apple devices in recovery mode
cargo run --bin bootforge-cli -- --apple --mode recovery

# Watch for any device changes
cargo run --bin bootforge-cli -- --watch
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
