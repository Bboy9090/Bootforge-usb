# BootForge USB - OMEGA TRANSCENDENT MODE 🔱⚡

> **The Ultimate Cross-Platform USB Device Library**

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Build](https://img.shields.io/badge/build-passing-brightgreen.svg)]()
[![Tests](https://img.shields.io/badge/tests-99%20passed-brightgreen.svg)]()
[![Clippy](https://img.shields.io/badge/clippy-0%20warnings-brightgreen.svg)]()

BootForge USB is the **most comprehensive USB library for Rust**, featuring complete device enumeration, real-time hotplug monitoring, protocol implementations, and advanced device control capabilities.

## ✨ Feature Summary

| Category | Features |
|----------|----------|
| **Enumeration** | Full device discovery, descriptor parsing, platform enrichment |
| **Descriptors** | Configuration, Interface, Endpoint, BOS, SuperSpeed, USB4 |
| **Protocols** | ADB, Fastboot, MTP, PTP, CDC (Serial), DFU |
| **Hotplug** | Real-time events, debouncing, reconnection correlation |
| **HID** | Complete report descriptor parsing, usage tables |
| **Database** | 500+ vendor/product name lookups, class definitions |
| **Query API** | Rich filtering, presets, device search |
| **Control** | Reset, power cycle, hub port control, driver binding |
| **Permissions** | Cross-platform helpers, udev rule generation |
| **Caching** | TTL-based caching, LRU eviction |

## 🚀 Quick Start

```toml
[dependencies]
bootforge-usb = "0.2"
```

### Basic Enumeration

```rust
use bootforge_usb::enumerate_all;

fn main() -> anyhow::Result<()> {
    let devices = enumerate_all()?;
    
    for device in &devices {
        println!("Found: {:04X}:{:04X} - {} {}",
            device.vendor_id, device.product_id,
            device.manufacturer.as_deref().unwrap_or("Unknown"),
            device.product.as_deref().unwrap_or("Device"));
    }
    
    Ok(())
}
```

### Device Query API

```rust
use bootforge_usb::{DeviceQuery, presets, database};

// Find Android devices
let android = DeviceQuery::new()
    .vendor_id(0x18D1)  // Google
    .filter(&devices);

// Use presets
let serial_adapters = presets::serial_adapters().filter(&devices);
let game_controllers = presets::game_controllers().filter(&devices);

// Look up device names
let db = database();
for device in &devices {
    let desc = db.device_description(device.vendor_id, device.product_id);
    println!("{}", desc);
}
```

### Real-Time Hotplug Monitoring

```rust
use bootforge_usb::{PlatformWatcher, DeviceWatcher, DeviceEvent};

let watcher = PlatformWatcher::new()?;

for event in watcher.events() {
    match event {
        DeviceEvent::Added(device) => println!("+ {}", device),
        DeviceEvent::Removed(device) => println!("- {}", device),
        DeviceEvent::Reconnected { device, .. } => println!("↺ {}", device),
        _ => {}
    }
}
```

### Protocol Communication

```rust
use bootforge_usb::{DeviceHandle, AdbClient, FastbootClient};

// ADB Communication
let handle = DeviceHandle::open(0x18D1, 0x4EE1)?;
let mut adb = AdbClient::new(&handle, 0x81, 0x01);
adb.connect("host::bootforge")?;
let output = adb.shell("getprop ro.product.model")?;
println!("Model: {}", output);

// Fastboot Communication
let mut fb = FastbootClient::new(&handle, 0x81, 0x01);
let info = fb.get_device_info()?;
println!("Product: {:?}", info.product);
```

### HID Report Descriptor Parsing

```rust
use bootforge_usb::ReportDescriptor;

let report_data = get_hid_report_descriptor(device)?;
let desc = ReportDescriptor::parse(&report_data)?;

println!("Device Type: {}", desc.device_type());
println!("Input Fields: {}", desc.input_fields.len());

for field in &desc.input_fields {
    println!("  {} bits @ offset {}", field.total_bits(), field.bit_offset);
}
```

### Device Control

```rust
use bootforge_usb::{DeviceControl, HubControl, PermissionHelper};

// Reset a device
DeviceControl::reset(0x1234, 0x5678)?;

// Power cycle a hub port
HubControl::power_off_port(hub_vid, hub_pid, 1)?;
std::thread::sleep(Duration::from_secs(1));
HubControl::power_on_port(hub_vid, hub_pid, 1)?;

// Check permissions
let status = PermissionHelper::check(&device);
if !status.has_access() {
    println!("{}", PermissionHelper::get_instructions(&device, &status));
}
```

## 📦 Module Overview

```
bootforge_usb
├── enumerate        # Device enumeration (libusb + platform-specific)
├── watcher          # Hotplug monitoring (Linux udev, Windows, macOS)
├── descriptors      # Full USB descriptor parsing
│   ├── bos          # Binary Object Store (USB 2.1+)
│   ├── superspeed   # USB 3.0+ capabilities
│   ├── power_delivery  # USB-PD status
│   └── alternate_modes # DisplayPort, Thunderbolt
├── communication    # Device I/O layer
│   ├── control      # Control transfers
│   ├── bulk         # Bulk transfers with retry
│   ├── interrupt    # Interrupt transfers, HID helpers
│   └── session      # Session management
├── protocols        # Protocol implementations
│   ├── adb          # Android Debug Bridge
│   ├── fastboot     # Android bootloader
│   ├── mtp          # Media Transfer Protocol
│   ├── ptp          # Picture Transfer Protocol
│   ├── cdc          # USB Serial (ACM, ECM, NCM)
│   └── dfu          # Device Firmware Upgrade
├── hid              # HID report descriptor parser
├── database         # USB ID database
├── query            # Device filtering & search
├── cache            # Device info caching
├── permissions      # Permission helpers
└── control          # Device/hub control
```

## 🔌 Supported Protocols

| Protocol | Detection | Communication | Description |
|----------|-----------|---------------|-------------|
| **ADB** | ✅ | ✅ Full | Android Debug Bridge |
| **Fastboot** | ✅ | ✅ Full | Android bootloader |
| **MTP** | ✅ | ✅ Full | Media Transfer Protocol |
| **PTP** | ✅ | ✅ Full | Picture Transfer Protocol |
| **CDC-ACM** | ✅ | ✅ Full | USB Serial |
| **DFU** | ✅ | ✅ Full | Device Firmware Upgrade |
| **HID** | ✅ | ✅ Full | Human Interface Device |

## 🖥️ Platform Support

| Feature | Linux | macOS | Windows |
|---------|:-----:|:-----:|:-------:|
| Enumeration | ✅ | ✅ | ✅ |
| Hotplug | ✅ (udev) | ✅ (IOKit) | ✅ (DevNotify) |
| Port Path | ✅ | ✅ | ✅ |
| Driver Status | ✅ | ✅ | ✅ |
| Power Control | ✅ | ⚠️ | ⚠️ |
| Hub Control | ✅ | ✅ | ✅ |

## 📊 USB Capabilities

- **USB 1.x/2.0**: Full support, LPM detection
- **USB 3.0/3.1/3.2**: SuperSpeed/SuperSpeed+ capabilities
- **USB4**: Basic detection and tunneling modes
- **USB-PD**: Power Delivery status and contracts
- **Alt Modes**: DisplayPort, Thunderbolt detection
- **Type-C**: Orientation and alternate mode status

## 🔧 Query Presets

```rust
use bootforge_usb::presets;

presets::android_devices()    // Google, Samsung, etc.
presets::apple_devices()      // All Apple VID
presets::serial_adapters()    // FTDI, CH340, CP210x
presets::storage_devices()    // Mass storage class
presets::hid_devices()        // HID class
presets::audio_devices()      // Audio class
presets::video_devices()      // Video class
presets::dev_boards()         // Arduino, Raspberry Pi, etc.
presets::security_keys()      // YubiKey, Nitrokey, etc.
presets::game_controllers()   // Xbox, PlayStation, Switch
```

## 🧪 Testing

```bash
cargo test                    # Run all 99 tests
cargo clippy                  # Lint (0 warnings)
cargo doc --open              # Generate documentation
```

## 📝 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

---

**BootForge USB** - *The Ultimate USB Library for Rust* 🔱⚡
