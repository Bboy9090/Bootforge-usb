# USB Discovery Model
## BootForge Technical Architecture

**Version**: 3.0.0
**Last Updated**: 2026-05-23

---

## Overview

BootForge's USB discovery model provides a layered architecture for detecting, classifying, and analyzing USB devices in a read-only, non-invasive manner. This document describes the technical architecture and design decisions.

---

## Architecture Layers

```
┌─────────────────────────────────────────────────┐
│         Applications Layer                      │
│  (bootforge-cli, workshop-ui, forgeworks-core)  │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│         ForgeWorks Services Layer               │
│  (device-analysis, ownership-verification,      │
│   legal-classification, audit-logging, etc.)    │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│         libbootforge Core Library               │
│  - Scanner: Device enumeration                  │
│  - Classifier: Mode/platform/transport/vendor   │
│  - Fingerprint: Device family identification    │
│  - Profiles: Known device database              │
│  - Session: Logging and reporting               │
│  - Descriptors: USB metadata extraction         │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│         Platform USB Stack                      │
│  Linux: libusb-1.0                              │
│  macOS: IOKit (via rusb)                        │
│  Windows: WinUSB/libusb-win32 (via rusb)        │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│         USB Hardware                            │
│  (Physical USB devices connected to host)       │
└─────────────────────────────────────────────────┘
```

---

## Core Components

### 1. Scanner (`libbootforge::detect::scanner`)

**Purpose**: Enumerate all USB devices connected to the host system.

**Functionality**:
- Iterates through all USB buses and devices
- Reads device descriptors from USB hardware
- Filters removable/mass storage devices
- Handles device enumeration errors gracefully

**Key Types**:
```rust
pub struct DeviceScanner {
    context: rusb::Context,
}

pub struct ScannedDevice {
    pub descriptor: DeviceDescriptor,
    pub bus: u8,
    pub address: u8,
    pub port_path: Vec<u8>,
}
```

**Read-Only Guarantee**: Uses `rusb::Device::device_descriptor()` which performs USB control transfers for reading only (no writes).

---

### 2. Classifier (`libbootforge::detect::classifier`)

**Purpose**: Classify devices by mode, platform, transport, and vendor.

**Functionality**:
- **Mode Classification**: Normal, DFU, Recovery, Fastboot, Bootloader, Unknown
- **Platform Classification**: Apple, Google, Samsung, Generic, Unknown
- **Transport Classification**: USB, Thunderbolt, Unknown
- **Vendor Classification**: Based on VID and device class

**Classification Logic**:
```rust
pub enum DeviceMode {
    Normal,       // Standard operating mode
    Dfu,          // Device Firmware Update mode
    Recovery,     // Recovery/restore mode
    Fastboot,     // Android fastboot mode
    Bootloader,   // Bootloader/download mode
    Unknown,      // Cannot determine
}
```

**Decision Tree**:
1. Check VID/PID against known profiles
2. Inspect USB device class codes
3. Analyze interface descriptors
4. Check for vendor-specific protocols

---

### 3. Fingerprint (`libbootforge::detect::fingerprint`)

**Purpose**: Identify device families with confidence scores.

**Functionality**:
- Match devices against known profiles
- Calculate confidence score (0.0 - 1.0)
- Support for fuzzy matching (e.g., different firmware versions)
- Extensible profile database

**Confidence Scoring**:
```rust
pub struct FingerprintMatch {
    pub profile_id: String,
    pub confidence: f32,  // 0.0 = no match, 1.0 = perfect match
    pub reasons: Vec<MatchReason>,
}

pub enum MatchReason {
    VidPidExact,         // Confidence: +0.6
    SerialPattern,       // Confidence: +0.2
    DeviceClass,         // Confidence: +0.1
    ManufacturerString,  // Confidence: +0.1
}
```

**Example**:
- Apple iPhone in Recovery Mode: 0.95 confidence (VID/PID exact + known serial pattern)
- Generic USB Flash Drive: 0.4 confidence (device class only)

---

### 4. Profiles (`libbootforge::detect::profiles`)

**Purpose**: Database of known device profiles for accurate identification.

**Profile Structure**:
```rust
pub struct DeviceProfile {
    pub id: String,
    pub name: String,
    pub vendor: String,
    pub vid: u16,
    pub pid: u16,
    pub device_class: u8,
    pub mode: DeviceMode,
    pub description: String,
}
```

**Built-In Profiles**:
- Apple iPhone (DFU mode): VID 0x05ac, PID 0x1227
- Apple iPhone (Recovery): VID 0x05ac, PID 0x1281
- Google Pixel (Fastboot): VID 0x18d1, PID 0x4ee0
- SanDisk USB Flash: VID 0x0781, PID varies (mass storage class)

**Extensibility**: Profiles can be loaded from external JSON files for community contributions.

---

### 5. Session Management (`libbootforge::session`)

**Purpose**: Track device operations across time with audit trails.

**Functionality**:
- **History**: Record device connection/disconnection events
- **Logging**: Append-only session logs with timestamps
- **Reporting**: Generate session reports in JSON/text format

**Session Log Entry**:
```rust
pub struct SessionLogEntry {
    pub timestamp: DateTime<Utc>,
    pub event_type: EventType,
    pub device_id: Option<String>,
    pub details: String,
}

pub enum EventType {
    DeviceConnected,
    DeviceDisconnected,
    ScanCompleted,
    ChecksumVerified,
    DryRunGenerated,
}
```

---

### 6. Descriptors (`libbootforge::descriptors`)

**Purpose**: Extract and parse USB descriptors.

**USB Descriptor Types**:
- **Device Descriptor**: VID, PID, manufacturer, product, serial
- **Configuration Descriptor**: Power requirements, interfaces
- **Interface Descriptor**: Class, subclass, protocol
- **String Descriptors**: Human-readable names

**Extraction Process**:
1. Open USB device handle (read-only)
2. Read device descriptor (standard USB request)
3. Read string descriptors for manufacturer/product/serial
4. Read configuration descriptor for interfaces
5. Close device handle

**Error Handling**: Gracefully handles missing or incomplete descriptors (common on low-power devices).

---

## Discovery Workflow

### Step-by-Step Process

```
1. Initialize USB Context
   ↓
2. Enumerate USB Buses
   ↓
3. For Each Device:
   a. Read Device Descriptor
   b. Filter by Device Class (mass storage, etc.)
   c. Read String Descriptors
   d. Classify Mode/Platform/Transport/Vendor
   e. Fingerprint Against Known Profiles
   f. Log to Session History
   ↓
4. Return List of ScannedDevices
   ↓
5. Generate Session Report
```

### Filtering Logic

**Included Devices**:
- USB Mass Storage (class 0x08)
- Removable Media
- Devices in DFU/Recovery/Fastboot modes
- Vendor-specific protocols (Apple, Android)

**Excluded Devices**:
- USB Hubs (class 0x09)
- Human Interface Devices (class 0x03) - keyboards, mice
- Audio Devices (class 0x01)
- Internal System Controllers

**Rationale**: Focus on storage and device recovery scenarios.

---

## Read-Only Guarantees

### Technical Enforcement

1. **No Write APIs**: libbootforge does not expose any USB write functions
2. **rusb Read-Only Calls**: Only uses `device_descriptor()`, `read_*()` methods
3. **No Control Transfers (Write)**: No SET_* USB requests, only GET_*
4. **No Firmware Flashing**: No bulk/interrupt OUT endpoints used
5. **Compiler-Level Safety**: Rust type system prevents accidental writes

### Runtime Verification

- Audit logs record only read operations
- Health check script verifies no write capability
- Integration tests assert no device modification

---

## Platform-Specific Considerations

### Linux

- **USB Access**: Requires read permission on `/dev/bus/usb/*/*`
- **udev Rules**: Recommended for non-root access
- **libusb-1.0**: Required system library

**udev Rule Example**:
```
# /etc/udev/rules.d/99-bootforge.rules
SUBSYSTEM=="usb", MODE="0664", GROUP="plugdev"
```

### macOS

- **IOKit**: Native macOS USB framework (via rusb)
- **Permissions**: No special permissions for read-only access
- **Sandboxing**: Compatible with macOS App Sandbox (read-only USB)

### Windows

- **Driver**: WinUSB or libusb-win32
- **Driver Installation**: May require Zadig for generic USB devices
- **Permissions**: Standard user permissions sufficient for read

---

## Performance Characteristics

### Benchmarks (Typical System)

| Operation | Time | Notes |
|-----------|------|-------|
| USB Context Init | < 10ms | One-time per application |
| Device Scan (10 devices) | < 500ms | Includes descriptor reads |
| Fingerprint Matching | < 5ms per device | Profile database lookup |
| Session Log Write | < 1ms per entry | Append to SQLite |
| Audit Hash Chain Verify | < 10ms per 1000 entries | SHA256 computation |

### Optimization Strategies

- **Lazy Loading**: Only read detailed descriptors on demand
- **Caching**: Cache device list with 1-second TTL
- **Asynchronous I/O**: Use rusb async API for concurrent reads
- **Profile Indexing**: Hash map for O(1) profile lookups

---

## Error Handling

### Common Error Scenarios

| Error | Cause | Handling |
|-------|-------|----------|
| `NoDevice` | Device unplugged during scan | Skip, log event |
| `Access` | Insufficient permissions | Report to user, suggest udev rules |
| `NotFound` | USB descriptor missing | Use defaults, mark as incomplete |
| `Io` | USB I/O error | Retry 3 times, then skip |
| `Timeout` | USB request timeout | Skip device, log warning |

### Error Recovery

```rust
match scanner.scan() {
    Ok(devices) => {
        // Success: process devices
    }
    Err(ScanError::Access) => {
        eprintln!("Permission denied. Run with sudo or configure udev rules.");
    }
    Err(ScanError::NoUsbContext) => {
        eprintln!("Failed to initialize USB. Check libusb installation.");
    }
    Err(e) => {
        eprintln!("Unexpected error: {}", e);
    }
}
```

---

## Security Model

### Threat Model

**In Scope**:
- Malicious USB devices (BadUSB)
- Descriptor parsing vulnerabilities
- Information leakage via logs

**Out of Scope**:
- Physical attacks (user has physical access)
- Kernel-level USB stack exploits
- Supply chain attacks on libusb

### Mitigations

1. **Input Validation**: All descriptor fields validated and sanitized
2. **Bounds Checking**: Rust prevents buffer overflows
3. **No Arbitrary Code Execution**: No dynamic loading of code
4. **Audit Logging**: All operations logged for forensics
5. **Least Privilege**: Runs as standard user (no root)

---

## Future Enhancements

### Planned Improvements (Post-MVP)

1. **Hot-Plug Support**: Real-time device events without polling
2. **USB 3.0+ Features**: SuperSpeed device enumeration
3. **USB-C Power Delivery**: Read PD capabilities and negotiated voltage
4. **Composite Devices**: Better handling of multi-function devices
5. **Wireless USB**: Support for USB-over-network protocols

### Research Areas

- Machine learning for unknown device classification
- Behavioral analysis for device mode detection
- Community-driven profile database

---

## References

- [USB 2.0 Specification](https://www.usb.org/document-library/usb-20-specification)
- [rusb Documentation](https://docs.rs/rusb/)
- [libusb 1.0 API](https://libusb.info/documentation/)

---

## Document History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-05-23 | Initial USB discovery model documentation |
