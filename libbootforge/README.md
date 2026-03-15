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
- **Known device profiles** - Built-in database of common devices with expected behaviors
- **Session logging** - Track device connection/disconnection events over time
- **Scan reports** - Generate exportable device scan reports
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

# Watch and save session log
bootforge-cli --watch --session-log session.json
```

#### Report Generation

```bash
# Generate and save scan report
bootforge-cli --report-file report.json

# Generate filtered report
bootforge-cli --apple --report-file apple_devices.json
```

#### Session Logging

```bash
# Create session log from single scan
bootforge-cli --session-log scan_session.json

# Watch mode with session logging
bootforge-cli --watch --session-log live_session.json
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

## Known Device Profiles

libbootforge includes a built-in database of known devices with expected characteristics:

- **Apple Devices**: DFU mode (0x05ac:0x1227), Recovery mode (0x05ac:0x1281), Normal mode (0x05ac:0x12a8)
- **Android Devices**: Google Fastboot (0x18d1:0x4ee7), Google ADB (0x18d1:0x4ee1)
- **Samsung Devices**: ADB (0x04e8:0x6860), Bootloader/Odin (0x04e8:0x685d)
- **Storage Devices**: SanDisk USB drives (0x0781:*)
- **Peripherals**: Logitech devices (0x046d:*)

When a device matches a known profile, the CLI displays the profile name and expected workflow.

## Session Logging

Session logging tracks device events over time, creating a history of what happened during a scan or watch session.

**Single Scan Session:**
```bash
bootforge-cli --session-log scan.json
```

Creates a session log with `Rescanned` events for all detected devices.

**Watch Mode Session:**
```bash
bootforge-cli --watch --session-log session.json
```

Records `Connected` and `Disconnected` events in real-time as devices are added or removed.

**Session Log Format:**
```json
{
  "session_id": "session_20240315T123045Z",
  "started_at": "2024-03-15T12:30:45Z",
  "ended_at": "2024-03-15T12:35:10Z",
  "events": [
    {
      "timestamp": "2024-03-15T12:30:50Z",
      "event_type": "Connected",
      "device": { ... }
    }
  ]
}
```

## Report Export

Generate structured scan reports with device snapshots:

```bash
bootforge-cli --report-file report.json
```

**Report Format:**
```json
{
  "generated_at": "2024-03-15T12:30:45Z",
  "total_devices": 3,
  "devices": [ ... ]
}
```

Reports can be filtered before export:
```bash
bootforge-cli --apple --mode recovery --report-file recovery_report.json
```

libbootforge provides intelligent device fingerprinting and workflow recommendations:

- **Device Family Identification**: Automatically identifies device families (iPhone, iPad, Android Phone, Android Tablet, USB Storage, Peripheral)
- **Confidence Levels**: Assigns confidence levels (High, Medium, Low) to fingerprints based on available information
- **Workflow Recommendations**: Suggests appropriate inspection or repair workflows based on device characteristics

Example output:

```
Bus 001 Device 002 ID 05ac:1281
  Vendor ID              : 05ac
  Product ID             : 1281
  Vendor Name            : Apple
  Manufacturer           : Apple Inc.
  Product                : iPhone
  Platform               : Apple
  Transport              : Usb2
  Mode                   : Recovery
  Fingerprint Family     : IPhone
  Model Hint             : iPhone
  Fingerprint Confidence : High
  Recommended Workflow   : AppleRecoveryWorkflow
```

**Note**: Workflow recommendations are advisory only and not guarantees. Always verify device state before performing any operations.

## Supported Vendors

libbootforge includes built-in vendor identification for:

- **Apple** (0x05ac)
- **Google** (0x18d1)
- **Samsung** (0x04e8)
- **Sony** (0x0fce)
- **OnePlus** (0x2a70)
- **Huawei** (0x12d1)
- **OPPO** (0x22d9)
- **Xiaomi** (0x2717)
- **Realtek** (0x0bda)
- **SanDisk** (0x0781)
- **Logitech** (0x046d)

## Supported Device Modes

- **Normal** - Standard operating mode
- **Recovery** - Device recovery mode (Apple, Android)
- **DFU** - Device Firmware Update mode (Apple)
- **Bootloader** - Bootloader mode (Android)
- **Fastboot** - Fastboot mode (Android)
- **ADB** - Android Debug Bridge mode
- **MassStorage** - USB mass storage mode

## Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| Linux | ✅ Supported | Includes sysfs path enrichment |
| macOS | ✅ Supported | IOKit enrichment planned |
| Windows | ✅ Supported | SetupAPI enrichment planned |

**Note**: Some device information may require elevated permissions on macOS and Linux.

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
# Format code
cargo fmt

# Run tests (non-USB tests work in CI)
cargo test

# Check for errors
cargo check

# Run all tests including USB hardware tests
cargo test -- --ignored --test-threads=1
```

### Running Examples

```bash
# List devices in human-readable format
cargo run --example list_devices

# List devices in JSON format
cargo run --example list_devices_json
```

### Running CLI

```bash
# Basic usage
cargo run --bin bootforge-cli

# With filtering
cargo run --bin bootforge-cli -- --apple
cargo run --bin bootforge-cli -- --mode recovery
cargo run --bin bootforge-cli -- --vendor 05ac

# JSON output
cargo run --bin bootforge-cli -- --json
cargo run --bin bootforge-cli -- --json-file devices.json

# Watch mode
cargo run --bin bootforge-cli -- --watch
```

Note: Tests requiring USB hardware access are marked with `#[ignore]` to allow CI testing.

## License

MIT OR Apache-2.0
