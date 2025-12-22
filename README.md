# BootForge USB

A cross-platform Rust library for USB device enumeration and information gathering with advanced features including real-time hotplug monitoring, protocol detection, and port topology mapping.

## Overview

BootForge USB provides a unified interface for discovering and managing USB devices across Windows, macOS, and Linux platforms. It combines cross-platform enumeration using libusb (via rusb) with platform-specific APIs for enriching device information, and adds powerful features for device monitoring and protocol detection.

## Features

- **Cross-platform enumeration**: Works on Windows, macOS, and Linux
- **Detailed device information**: Vendor ID, Product ID, manufacturer, product name, serial number
- **Real-time hotplug monitoring**: Watch for USB device connection and disconnection events
- **Protocol detection**: Automatically detect ADB, Fastboot, Apple devices, MTP, and more
- **USB port topology**: Map USB hub connections and port paths
- **Driver status**: Query driver binding and health status
- **Platform-specific enrichment**: 
  - Windows: Device paths via SetupAPI
  - macOS: IORegistry paths via IOKit
  - Linux: sysfs paths and udev integration
- **Normalized data structures**: Consistent API across all platforms
- **Extensible design**: Trait-based architecture for custom implementations

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
bootforge-usb = "0.2"
```

### Basic enumeration example:

```rust
use bootforge_usb::enumerate_all;

fn main() -> anyhow::Result<()> {
    let devices = enumerate_all()?;
    
    for device in devices {
        println!("Device: {}", device);
        println!("  Vendor ID: {:04x}", device.vendor_id);
        println!("  Product ID: {:04x}", device.product_id);
        
        if let Some(manufacturer) = device.manufacturer {
            println!("  Manufacturer: {}", manufacturer);
        }
        
        if let Some(product) = device.product {
            println!("  Product: {}", product);
        }
    }
    
    Ok(())
}
```

### Device watching example:

```rust
use bootforge_usb::{DeviceWatcher, PlatformWatcher, DeviceEvent};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut watcher = PlatformWatcher::default();
    let receiver = watcher.start()?;

    loop {
        match receiver.recv_timeout(Duration::from_secs(1)) {
            Ok(DeviceEvent::Added(device)) => {
                println!("Device added: {}", device.id.as_hex_string());
            }
            Ok(DeviceEvent::Removed(device)) => {
                println!("Device removed: {}", device.id.as_hex_string());
            }
            Ok(DeviceEvent::Changed(device)) => {
                println!("Device changed: {}", device.id.as_hex_string());
            }
            _ => continue,
        }
    }
}
```

### Protocol detection example:

```rust
use bootforge_usb::{api::UsbEnumerator, classify_device_protocols, enumerate::FallbackEnumerator};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let enumerator = FallbackEnumerator::default();
    let devices = enumerator.enumerate()?;

    for device in devices {
        let protocols = classify_device_protocols(&device);
        println!("Device {} supports: {:?}", device.id.as_hex_string(), protocols);
    }

    Ok(())
}
```

## Architecture

### Core Components

- **`api.rs`**: Defines `UsbEnumerator` trait for custom implementations
- **`model.rs`**: Unified device model with `UsbDeviceRecord`, `DriverStatus`, `LinkHealth`
- **`errors.rs`**: Comprehensive error types for USB operations
- **`types.rs`**: Legacy data structures for backward compatibility
- **`enumerate/`**: Enumeration and enrichment modules
  - `mod.rs`: Cross-platform dispatcher and main `enumerate_all()` function
  - `common.rs`: `FallbackEnumerator` using rusb/libusb
  - `libusb.rs`: Base enumeration using rusb/libusb
  - `windows.rs`: Windows-specific enrichment
  - `macos.rs`: macOS-specific enrichment
  - `linux.rs`: Linux-specific enrichment using sysfs
- **`watcher/`**: Real-time device monitoring
  - `mod.rs`: `DeviceWatcher` trait and `DeviceEvent` types
  - `linux.rs`: udev-based monitoring (requires udev feature)
  - `windows.rs`: Windows device notification
  - `macos.rs`: IOKit notification monitoring
- **`handshake/`**: Protocol detection modules
  - `mod.rs`: Main protocol classification function
  - `adb_probe.rs`: ADB device detection
  - `fastboot_probe.rs`: Fastboot device detection
  - `apple_probe.rs`: Apple device detection
  - `mtp_probe.rs`: MTP device detection
- **`ports/`**: USB topology mapping
  - `mod.rs`: Hub enumeration and port path parsing

### Enumeration Flow

1. **Base Enumeration**: Uses libusb (rusb) to discover all USB devices
2. **Descriptor Reading**: Extracts basic information from USB device descriptors
3. **String Descriptors**: Attempts to read manufacturer, product, and serial number strings
4. **Platform Enrichment**: Applies OS-specific enrichment to add platform-specific paths and metadata
5. **Protocol Classification**: Detects device protocols (ADB, Fastboot, Apple, MTP)

## Platform Support

| Platform | Status | Implementation |
|----------|--------|----------------|
| Linux | ✅ Implemented | libusb + sysfs + udev |
| Windows | ✅ Implemented | libusb + SetupAPI |
| macOS | ✅ Implemented | libusb + IOKit |

## Examples

See the `examples/` directory for complete working examples:

- `list_devices.rs`: Basic device enumeration
- `watch_devices.rs`: Real-time hotplug monitoring
- `detect_protocols.rs`: Protocol detection demonstration

Run examples with:

```bash
cargo run --example list_devices
cargo run --example watch_devices
cargo run --example detect_protocols
```

## Requirements

- Rust 2021 edition or later
- libusb 1.0 or compatible (rusb dependency)
- Platform-specific requirements:
  - Linux: udev development libraries (optional, for hotplug monitoring)
  - Windows: Windows SDK
  - macOS: IOKit framework

## Development

Build the project:

```bash
cargo build
```

Run tests:

```bash
cargo test
```

Run clippy:

```bash
cargo clippy
```

Note: Some tests may require elevated permissions or USB devices connected to succeed.

## Features

- `udev` (Linux only): Enables udev-based hotplug monitoring on Linux

## License

MIT OR Apache-2.0

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.
