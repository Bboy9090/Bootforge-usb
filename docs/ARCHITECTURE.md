# BootForge USB Architecture

This document describes the architecture of BootForge USB, including the detection pipeline, identity resolution, and operation safety model.

## Overview

BootForge USB is a cross-platform USB device detection and monitoring library that provides:
- Unified device enumeration across Windows, macOS, and Linux
- Real-time hotplug monitoring
- Stable device identification across reconnections
- Protocol classification (ADB, Fastboot, Apple, MTP)
- USB topology mapping

## Detection Pipeline

The detection pipeline transforms raw USB bus data into fully-identified, classified devices ready for application use.

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Detection Pipeline                            │
└─────────────────────────────────────────────────────────────────────┘

Stage 1: Transport Scanning
┌──────────────────────────────────────────────────────────────────┐
│  scanUsbTransports()                                              │
│  ├─ Initialize libusb context                                     │
│  ├─ Query USB device list                                         │
│  └─ For each device: probeUsbCandidate()                          │
│     ├─ Read device descriptor (VID/PID/class)                     │
│     ├─ Get bus number and device address                          │
│     └─ Create candidate device record                             │
│                                                                    │
│  Output: Vec<CandidateDevice>                                     │
│    - VID/PID known                                                │
│    - Bus/address known                                            │
│    - String descriptors NOT yet read                              │
└──────────────────────────────────────────────────────────────────┘
                              ↓
Stage 2: Descriptor Reading
┌──────────────────────────────────────────────────────────────────┐
│  extractStringDescriptors()                                       │
│  ├─ Open device handle (may fail due to permissions)             │
│  ├─ Read supported languages                                      │
│  ├─ Read manufacturer string (if index present)                   │
│  ├─ Read product string (if index present)                        │
│  └─ Read serial number (if index present)                         │
│                                                                    │
│  Output: Enhanced candidates                                      │
│    - Strings populated when available                             │
│    - Graceful degradation on permission denied                    │
└──────────────────────────────────────────────────────────────────┘
                              ↓
Stage 3: Platform Enrichment
┌──────────────────────────────────────────────────────────────────┐
│  enrichDeviceWithPlatformData()                                   │
│                                                                    │
│  ┌─ Windows: enrich_windows()                                     │
│  │  ├─ Query SetupAPI for device paths                           │
│  │  ├─ Read instance paths and hardware IDs                       │
│  │  └─ Determine driver status                                    │
│  │                                                                 │
│  ┌─ Linux: enrich_linux()                                         │
│  │  ├─ Map to sysfs paths                                         │
│  │  ├─ Read driver binding from sysfs                             │
│  │  ├─ Check authorization status                                 │
│  │  └─ Get device node (/dev/bus/usb/...)                         │
│  │                                                                 │
│  └─ macOS: enrich_macos()                                         │
│     ├─ Query IOKit registry                                       │
│     ├─ Get IORegistry path                                        │
│     ├─ Read location ID                                           │
│     └─ Determine driver binding                                   │
│                                                                    │
│  Output: Platform-enriched devices                                │
│    - Platform-specific paths added                                │
│    - Driver status determined                                     │
│    - System metadata populated                                    │
└──────────────────────────────────────────────────────────────────┘
                              ↓
Stage 4: Protocol Classification
┌──────────────────────────────────────────────────────────────────┐
│  classify_device_protocols()                                      │
│  ├─ Check ADB (is_adb_device)                                     │
│  │  └─ VID/PID in known list OR class 0xFF/0x42/0x01             │
│  ├─ Check Fastboot (is_fastboot_device)                           │
│  │  └─ VID/PID in known list OR class 0xFF/0x42/0x03             │
│  ├─ Check Apple (is_apple_device)                                 │
│  │  └─ VID 0x05AC (Apple Inc.)                                    │
│  └─ Check MTP (is_mtp_device)                                     │
│     └─ Class 0x06 (Still Image) or known MTP VID/PID             │
│                                                                    │
│  Output: Confirmed devices with protocol tags                     │
│    - All information complete                                     │
│    - Ready for application use                                    │
└──────────────────────────────────────────────────────────────────┘
                              ↓
                    ┌─────────────────┐
                    │ Confirmed Device │
                    │ (UsbDeviceRecord)│
                    └─────────────────┘
```

## Identity Resolution Strategy

Tracking devices across reconnections requires stable identification:

```
┌─────────────────────────────────────────────────────────────────┐
│              Device Identity Resolution                          │
└─────────────────────────────────────────────────────────────────┘

Priority 1: Serial Number
┌──────────────────────────────────────────────────────────────┐
│ IF device.descriptor.serial_number.is_some():                 │
│   identity = (VID, PID, serial_number)                        │
│   ✓ Stable across any port                                    │
│   ✓ Unique per device                                         │
│   ✗ Some devices lack serials                                 │
│   ✗ Some devices have duplicate serials                       │
└──────────────────────────────────────────────────────────────┘

Priority 2: Port Path
┌──────────────────────────────────────────────────────────────┐
│ ELSE IF device.location.port_path.is_some():                  │
│   identity = (VID, PID, port_path)                            │
│   ✓ Stable if device stays in same port                       │
│   ✓ Works for devices without serials                         │
│   ✗ Changes if device moved to different port                 │
└──────────────────────────────────────────────────────────────┘

Priority 3: Location Fingerprint
┌──────────────────────────────────────────────────────────────┐
│ ELSE:                                                          │
│   identity = (VID, PID, bus, address, descriptor_hash)        │
│   ✓ Works as fallback                                         │
│   ✗ May change on reconnect                                   │
│   ✗ Least stable option                                       │
└──────────────────────────────────────────────────────────────┘

Usage in Application:
  - Cache confirmed devices by resolved identity
  - On hotplug event, resolve identity and match to cache
  - Update cached device or create new entry
  - Handle identity conflicts (duplicate serials) gracefully
```

## Hotplug Monitoring Architecture

Real-time device event detection using platform-specific notification systems:

```
┌─────────────────────────────────────────────────────────────────┐
│                   Hotplug Monitoring                             │
└─────────────────────────────────────────────────────────────────┘

Application Thread                    Watcher Thread
      │                                      │
      │ watcher.start()                      │
      ├──────────────────────────────────────>│
      │                                      │ Initialize platform watcher
      │                                      │ ┌─ Linux: udev monitor
      │                                      │ ├─ Windows: RegisterDeviceNotification
      │                                      │ └─ macOS: IOKit notifications
      │                                      │
      │<──────── Receiver<DeviceEvent> ─────┤
      │                                      │
      │                                      │ [Monitoring loop]
      │                                      │ Wait for USB events...
      │                                      │
  [Event loop]                               │ Device plugged in!
  receiver.recv()                            │ ├─ Detect bus change
      │                                      │ ├─ Enumerate new device
      │                                      │ ├─ Run detection pipeline
      │                                      │ └─ Send DeviceEvent::Added
      │                                      │
      │<───── DeviceEvent::Added(device) ───┤
      │                                      │
      │ Process device...                    │
      │                                      │
      │                                      │ Device unplugged!
      │                                      │ ├─ Detect removal
      │                                      │ └─ Send DeviceEvent::Removed
      │                                      │
      │<──── DeviceEvent::Removed(device) ──┤
      │                                      │
      │ Handle disconnection...              │
      │                                      │

Event Types:
  - DeviceEvent::Added    → New device confirmed and ready
  - DeviceEvent::Removed  → Device disconnected
  - DeviceEvent::Changed  → Driver or state changed
```

## Operation Safety Model

While BootForge USB focuses on detection and monitoring, safe device operations require understanding of concurrent access patterns.

### Conceptual Lock Semantics

```
┌─────────────────────────────────────────────────────────────────┐
│                   Lock Scope Hierarchy                           │
└─────────────────────────────────────────────────────────────────┘

Global Enumeration Lock (Conceptual)
┌──────────────────────────────────────────────────────────────┐
│  Scope: All USB enumeration operations                        │
│  Type: Exclusive (single enumerator at a time)                │
│  Rationale: Prevent concurrent bus scanning                   │
│                                                                │
│  Operations protected:                                         │
│    - enumerate_all()                                           │
│    - Full device list queries                                  │
│                                                                │
│  Implementation: Application responsibility                    │
└──────────────────────────────────────────────────────────────┘

Transport Lock (Per-bus)
┌──────────────────────────────────────────────────────────────┐
│  Scope: Access to specific USB bus/controller                 │
│  Type: Shared-read / Exclusive-write                          │
│  Rationale: Allow concurrent reads, serialize writes          │
│                                                                │
│  Read operations (shared):                                     │
│    - Query device descriptors                                  │
│    - Read device information                                   │
│    - Check connection status                                   │
│                                                                │
│  Write operations (exclusive):                                 │
│    - Device reset                                              │
│    - Configuration changes                                     │
│    - Control transfers                                         │
│                                                                │
│  Implementation: rusb::DeviceHandle provides thread safety     │
└──────────────────────────────────────────────────────────────┘

Device Session Lock (Per-device)
┌──────────────────────────────────────────────────────────────┐
│  Scope: Operations on a specific device                       │
│  Type: Exclusive (single active session per device)           │
│  Rationale: Prevent conflicting operations on same device     │
│                                                                │
│  Protects:                                                     │
│    - Interface claiming                                        │
│    - Bulk/interrupt transfers                                  │
│    - Protocol-specific communication (ADB, Fastboot, etc.)    │
│                                                                │
│  Pattern:                                                      │
│    acquireDeviceSession(device_id)                             │
│    try { /* perform operations */ }                            │
│    finally { releaseDeviceSession(device_id) }                 │
│                                                                │
│  Implementation: Application responsibility                    │
└──────────────────────────────────────────────────────────────┘
```

### Safe Operation Patterns

#### Pattern 1: Enumeration with Monitoring
```rust
// Safe: Single thread for enumeration
let devices = enumerate_all()?;

// Safe: Watcher in separate thread
let mut watcher = PlatformWatcher::default();
let receiver = watcher.start()?;

// Safe: Process events in application thread
for event in receiver {
    match event {
        DeviceEvent::Added(device) => handle_new_device(device),
        DeviceEvent::Removed(device) => handle_disconnect(device),
        DeviceEvent::Changed(device) => handle_change(device),
    }
}
```

#### Pattern 2: Concurrent Device Access
```rust
// Application must implement per-device locks
let device_locks: HashMap<UsbId, Mutex<()>> = HashMap::new();

// Thread 1: Reading device A
{
    let _guard = device_locks[&device_a_id].lock();
    let info = get_device_info(device_a); // Safe
}

// Thread 2: Can read device B concurrently
{
    let _guard = device_locks[&device_b_id].lock();
    let info = get_device_info(device_b); // Safe
}

// Both threads accessing device A would serialize
```

## Module Organization

```
bootforge-usb/
├── src/
│   ├── lib.rs                    # Public API surface
│   ├── api.rs                    # UsbEnumerator trait
│   ├── model.rs                  # Core data structures
│   ├── errors.rs                 # Error types
│   ├── types.rs                  # Legacy types (backward compat)
│   │
│   ├── enumerate/                # Detection pipeline
│   │   ├── mod.rs                # Main enumerate_all() entry point
│   │   ├── common.rs             # FallbackEnumerator (uses libusb)
│   │   ├── libusb.rs             # Stage 1: Transport scanning
│   │   ├── windows.rs            # Stage 3: Windows enrichment
│   │   ├── macos.rs              # Stage 3: macOS enrichment
│   │   └── linux.rs              # Stage 3: Linux enrichment
│   │
│   ├── watcher/                  # Hotplug monitoring
│   │   ├── mod.rs                # DeviceWatcher trait, events
│   │   ├── linux.rs              # udev-based monitoring
│   │   ├── windows.rs            # Windows device notifications
│   │   └── macos.rs              # IOKit notifications
│   │
│   ├── handshake/                # Protocol detection
│   │   ├── mod.rs                # classify_device_protocols()
│   │   ├── adb_probe.rs          # ADB detection
│   │   ├── fastboot_probe.rs     # Fastboot detection
│   │   ├── apple_probe.rs        # Apple device detection
│   │   └── mtp_probe.rs          # MTP detection
│   │
│   └── ports/                    # Topology mapping
│       └── mod.rs                # Port path parsing, hub enumeration
│
├── docs/
│   ├── GLOSSARY.md               # Term definitions
│   └── ARCHITECTURE.md           # This document
│
└── examples/
    ├── list_devices.rs           # Basic enumeration
    ├── watch_devices.rs          # Hotplug monitoring
    └── detect_protocols.rs       # Protocol classification
```

## Data Flow Summary

```
USB Bus
   ↓
[libusb/rusb] ← Cross-platform USB access
   ↓
Stage 1: Transport Scanning
   ├─ probeUsbCandidate() for each device
   └─ Output: Candidate devices (VID/PID/bus/address)
   ↓
Stage 2: Descriptor Reading
   ├─ Open device handle
   ├─ Read string descriptors
   └─ Output: Enhanced candidates (with strings)
   ↓
Stage 3: Platform Enrichment
   ├─ Windows: SetupAPI → device paths, drivers
   ├─ macOS: IOKit → registry paths, location IDs
   └─ Linux: sysfs → paths, drivers, authorization
   ↓
Stage 4: Protocol Classification
   ├─ Check VID/PID against known lists
   ├─ Check USB class/subclass/protocol codes
   └─ Output: Confirmed devices with protocol tags
   ↓
Application
   ├─ Use devices from enumerate_all()
   ├─ Monitor events from DeviceWatcher
   └─ Implement operation locks as needed
```

## Error Handling Philosophy

1. **Fail gracefully**: If one device fails to enumerate, continue with others
2. **Assume permissions may be denied**: String descriptors may be unreadable
3. **Handle hot-unplug**: Device may disconnect during enumeration
4. **Log but don't panic**: Warn about failures but return partial results
5. **Provide error context**: Use detailed error messages with device info

## Thread Safety Guarantees

- **Enumeration functions** (`enumerate_all`, etc.): Not thread-safe; call from single thread
- **UsbDeviceRecord/UsbDeviceInfo**: Safe to read from multiple threads (immutable)
- **DeviceWatcher**: Safe to run in separate thread; sends events via channels
- **rusb::DeviceHandle**: Thread-safe for read operations; applications must synchronize writes

## Performance Characteristics

- **Initial enumeration**: O(n) where n = number of USB devices (typically 10-50ms)
- **String descriptor reading**: May timeout (1 second per device if permission denied)
- **Platform enrichment**: Fast on Linux/macOS (sysfs/IOKit), slower on Windows (SetupAPI)
- **Hotplug monitoring**: Event-driven, minimal CPU usage when idle
- **Protocol classification**: O(1) per device (simple VID/PID lookups)

## Future Considerations

While the current implementation provides detection and monitoring, future enhancements could include:

1. **Explicit locking API**: Library-provided per-device locks for applications
2. **Session management**: Track device sessions with automatic cleanup
3. **Connection pooling**: Reuse device handles across operations
4. **Retry mechanisms**: Automatic retry with backoff for transient failures
5. **Caching layer**: Cache confirmed devices to speed up reconnection detection

These would be additive features that don't change the core detection pipeline.
