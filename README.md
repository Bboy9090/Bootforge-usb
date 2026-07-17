# BootForge

**BootForge** is a cross-platform USB device detection and enumeration tool for diagnostic and read-only device analysis. Built with Rust, it provides the foundational layer for the Blue Phoenix OS / Bobby's World device ecosystem.

## Overview

BootForge serves as the USB hardware discovery layer, providing:
- USB device detection and descriptor reading
- Vendor ID/Product ID identification
- Protocol classification (ADB, Fastboot, MTP, Apple)
- Platform-specific device path resolution
- **Diagnostic and read-only operations only** — no device modification

### Architecture

```
    Bobby's Workshop (Public UX)
              ↓
    ForgeWorks Core (Compliance Engine)
              ↓
    BootForge USB (Device Detection)
              ↓
          Hardware
```

## Quick Start

### Prerequisites

- Rust 1.70+ (2021 edition)
- libusb (platform-specific installation)
- Git

### Building

```bash
# Clone repository
git clone https://github.com/Bboy9090/Bootforge-usb.git
cd Bootforge-usb

# Build entire workspace
cargo build --release

# Build libbootforge only
cargo build -p libbootforge --release

# Build CLI tool
cargo build --bin bootforge-cli --release
```

### Testing

```bash
# Run all tests
cargo test

# Run libbootforge tests only
cargo test -p libbootforge

# Run with USB hardware (requires connected devices)
cargo test -- --ignored --test-threads=1
```

### Running

```bash
# List connected USB devices
cargo run --bin bootforge-cli

# Or use the built binary
./target/release/bootforge-cli
```

### Phoenix Key Interface

Phoenix Key is the desktop product layer powered by `libbootforge`. The browser preview uses a clearly labeled sample device; the Tauri desktop build calls the real read-only USB scanner.

```bash
cd apps/workshop-ui

# Install locked frontend dependencies
npm ci

# Run the browser preview
npm run dev

# Verify the production frontend
npm run build

# Run the native desktop shell (requires the Tauri v1 prerequisites)
npm run desktop:dev

# Produce platform installers
npm run desktop:build
```

Windows MSI and NSIS packages are also built by `.github/workflows/windows-desktop.yml` and uploaded as the `phoenix-key-windows-installers` workflow artifact.

## Core Components

### libbootforge

Low-level USB device detection library providing:
- Device scanning and enumeration
- Descriptor extraction (vendor/product IDs, serial numbers)
- Device mode classification (DFU, recovery, bootloader, normal)
- Device fingerprinting with confidence levels
- Session logging and audit trails

**Location**: `libbootforge/`

### ForgeWorks Services

Compliance and analysis microservices:
- **device-analysis**: Capability analysis and modification classification
- **ownership-verification**: Confidence-based attestation engine
- **legal-classification**: Jurisdiction-aware status labeling
- **audit-logging**: Immutable, hash-chained activity trail
- **authority-routing**: OEM, carrier, court system pathways
- **auth**: SAML/OIDC authentication
- **metrics**: Performance and compliance metrics

**Location**: `services/*/`

### Applications

- **workshop-ui**: React + Tauri desktop application for device discovery UI
- **forgeworks-core**: Tauri-based compliance engine frontend

**Location**: `apps/*/`

## Usage Examples

```rust
use libbootforge::detect::scanner::DeviceScanner;

// Scan for USB devices
let scanner = DeviceScanner::new();
let devices = scanner.scan()?;

for device in devices {
    println!("Device: {} ({:04x}:{:04x})",
        device.descriptor.product_name,
        device.descriptor.vendor_id,
        device.descriptor.product_id
    );
}
```

## Packaging

See `packaging/README.md` for platform-specific packaging instructions:
- Windows MSIX packaging
- Blue Phoenix OS integration
- Cross-platform distribution

## Health Checks

```bash
# Verify USB detection and safe mode
./scripts/healthcheck.sh

# Run smoke tests (build + entrypoints)
./scripts/smoke-test.sh
```

## Documentation

- **[Product Requirements](docs/PRD.md)**: MVP features and scope
- **[Roadmap](docs/ROADMAP.md)**: Future development plans
- **[USB Discovery Model](docs/USB_DISCOVERY_MODEL.md)**: Technical architecture
- **[Safe Write Policy](docs/SAFE_WRITE_POLICY.md)**: Read-only guarantees
- **[Release Checklist](docs/RELEASE_CHECKLIST.md)**: Pre-release validation

## Project Structure

```
Bootforge-usb/
├── libbootforge/          # Core USB detection library
├── services/              # ForgeWorks compliance services
├── apps/                  # User-facing applications
├── docs/                  # Documentation
├── scripts/               # Build and health check scripts
├── packaging/             # Platform packaging configs
├── firmware/              # ForgeCore hardware tests
├── manufacturing/         # Hardware BOM and QA
└── governance/            # Compliance policies
```

## Platform Support

- **Linux**: Full support (requires libusb-1.0)
- **macOS**: Full support (built-in IOKit support)
- **Windows**: Full support (WinUSB/libusb-win32)

## Design Principles

1. **Read-Only First**: Only reads device descriptors, never modifies hardware
2. **Platform Neutral**: Cross-platform support via rusb/libusb
3. **Compliance-First**: Ownership, consent, jurisdiction verification
4. **Audit Everything**: Immutable audit trails for all operations

## Known Limitations (MVP)

- No destructive disk operations
- No device firmware modification
- No bootloader unlocking or bypass operations
- Read-only USB enumeration and analysis only

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Run CI checks locally
cargo build && cargo test && cargo fmt --check
```

## Security

See [SECURITY.md](SECURITY.md) for security policy and vulnerability reporting.

## License

Dual-licensed under MIT or Apache-2.0. See [LICENSE](LICENSE) for details.
