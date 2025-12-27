# BootForge USB Glossary

This document defines key terms and concepts used throughout the BootForge USB library to ensure consistent understanding of the detection pipeline, device identification, and operation safety.

## Device Discovery and Identification

### Candidate Device
A USB device that has been detected during initial bus scanning but has not yet been fully identified. Candidates have minimal information (vendor ID, product ID, bus/address) but may lack string descriptors, platform-specific paths, or protocol classification.

**Stage**: Initial detection  
**Properties**: VID/PID, bus number, device address  
**Next**: Enrichment → Confirmed Device

### Confirmed Device
A USB device that has been fully enumerated with all available information including string descriptors (manufacturer, product, serial), platform-specific paths, driver status, and protocol classification. A confirmed device represents a complete and stable identity.

**Stage**: Post-enrichment  
**Properties**: All candidate properties + descriptors + platform paths + protocols  
**Representation**: `UsbDeviceRecord` or `UsbDeviceInfo` structs

### Device Identity Resolution
The process of establishing a stable, unique identifier for a USB device across reconnections and system reboots. Identity resolution uses multiple strategies:

1. **Serial Number**: Preferred method when available
2. **Port Path**: Physical USB port location (stable if device stays in same port)
3. **VID/PID + Location**: Combination of vendor/product ID and bus topology
4. **Descriptor Fingerprint**: Unique combination of device characteristics

### Session
A period during which a device is connected and available for operations. A session begins when a device is detected and confirmed, and ends when the device is disconnected or becomes unavailable. Sessions track:
- Device arrival time
- Current state (available, in-use, disconnected)
- Active operations
- Health metrics

## Transport and Communication

### USB Transport
The underlying communication channel to a USB device, representing the physical or logical connection method. Transports can be:
- **Direct USB**: Standard USB connection via libusb/rusb
- **Platform-Specific**: Windows (SetupAPI), Linux (sysfs/udev), macOS (IOKit)
- **Virtual USB**: USB/IP or other virtualization layers

### Bus Topology
The hierarchical structure of USB connections, including:
- **Bus**: Root USB controller
- **Hub**: USB hub device with multiple ports
- **Port**: Physical connection point on a hub or controller
- **Port Path**: Hierarchical address (e.g., "1-2.3.4" = bus 1, port 2, hub port 3, device port 4)

### Protocol
A high-level communication protocol that a device supports:
- **ADB**: Android Debug Bridge
- **Fastboot**: Android bootloader protocol
- **MTP**: Media Transfer Protocol
- **Apple Device**: iOS/iPadOS device protocols
- **Unknown**: Generic USB device

## Detection Pipeline Stages

### Stage 1: Transport Scanning
Initial discovery of USB devices by querying all available transports. This stage produces a list of candidate devices with minimal information.

**Functions**: `scanUsbTransports`, `probeUsbCandidate`  
**Output**: List of candidate devices with VID/PID/address

### Stage 2: Descriptor Reading
Attempting to read USB string descriptors (manufacturer, product, serial) from each candidate. This may fail due to permissions or device state.

**Functions**: Internal descriptor reading in enumeration  
**Output**: Enhanced candidates with string data (when available)

### Stage 3: Platform Enrichment
Augmenting device information with platform-specific data using OS APIs. This adds paths, driver status, and system-specific metadata.

**Functions**: `enrich_windows`, `enrich_macos`, `enrich_linux`  
**Output**: Confirmed devices with complete information

### Stage 4: Protocol Classification
Detecting which high-level protocols a device supports based on USB descriptors, vendor/product IDs, and interface classes.

**Functions**: `classify_device_protocols`, protocol probe functions  
**Output**: Device with tagged protocols

## Operation Safety

### Lock Scope
The granularity at which operations are synchronized:
- **Per-Device Lock**: Prevents concurrent operations on the same device
- **Global Lock**: Prevents concurrent enumeration across all devices
- **Transport Lock**: Prevents concurrent access to the same USB transport

### Read Operation
An operation that queries device state without modifying it. Examples:
- Reading device descriptors
- Querying driver status
- Checking connection state
- Reading protocol information

**Safety**: Multiple concurrent reads allowed (read-shared)

### Write Operation
An operation that may modify device state or configuration. Examples:
- Claiming interfaces
- Sending control transfers
- Changing device configuration
- Driver binding/unbinding

**Safety**: Exclusive access required (write-exclusive)

### Operation Lock Semantics

#### Device Read Lock (Conceptual)
Allows multiple concurrent readers to access device information safely.
- **Acquire**: Before reading device state
- **Release**: After read completes
- **Conflicts**: None (multiple readers allowed)
- **Pattern**: Shared/concurrent access

#### Device Write Lock (Conceptual)
Ensures exclusive access when performing operations that modify device state.
- **Acquire**: Before any write/control operation
- **Release**: After operation completes or on error
- **Conflicts**: Other writes, and all reads
- **Pattern**: Exclusive access (mutex-like)

**Note**: Current implementation does not enforce locks; applications using libusb/rusb must implement their own synchronization for device handles.

## Event System

### Device Event
A notification that a device's state has changed. Events drive reactive monitoring and reconnection handling.

**Types**:
- **Added**: New device detected and confirmed
- **Removed**: Device disconnected or became unavailable
- **Changed**: Device state modified (driver change, protocol detection)

### Hotplug Monitoring
Real-time detection of device connection and disconnection events using platform-specific notification mechanisms:
- **Linux**: udev monitoring
- **Windows**: Device notification API
- **macOS**: IOKit notifications

## State Transitions

### Device Lifecycle States

```
[Disconnected] 
    ↓ (plug in)
[Candidate] ← Initial detection, minimal info
    ↓ (descriptor read)
[Enriching] ← Reading strings, platform data
    ↓ (classification)
[Confirmed] ← Fully identified, ready for use
    ↓ (in use)
[Active Session] ← Device being used by application
    ↓ (done/error)
[Confirmed] ← Back to idle
    ↓ (unplug)
[Disconnected]
```

## Driver and Health

### Driver Status
The state of the operating system driver binding to a device:
- **Unknown**: Cannot determine driver state
- **Bound**: Driver loaded and attached (includes driver name)
- **Missing**: No driver loaded
- **Blocked**: Driver binding prevented (includes reason)
- **Multiple**: Multiple drivers bound (includes driver list)

### Link Health
The physical connection quality and stability:
- **Good**: Normal operation
- **Unstable**: Intermittent connectivity (includes reason)
- **PowerIssueHint**: Possible power delivery problem (includes reason)
- **ResetLoop**: Device repeatedly resetting
- **Disconnected**: Device no longer accessible

## Best Practices

### Stable Device Identification
To reliably track devices across reconnects:
1. Prefer serial numbers for unique identification
2. Use port paths for position-dependent tracking
3. Combine VID/PID with location for fallback
4. Handle missing/duplicate serials gracefully

### Concurrent Operation Safety
When multiple threads access USB devices:
1. Enumerate devices on a single thread when possible
2. Use device handles from rusb with proper synchronization
3. Implement per-device locks at the application level
4. Avoid holding locks during blocking I/O

### Fast Refresh and Reconnection
For responsive device monitoring:
1. Use hotplug watchers instead of polling
2. Cache confirmed device information
3. Implement quick reconnection detection by port path
4. Rate-limit enumeration to avoid USB bus saturation

### Error Handling
USB operations can fail in many ways:
1. Assume descriptors may be unreadable (permissions)
2. Gracefully handle device disconnection during enumeration
3. Retry transient failures with exponential backoff
4. Log failures but continue with other devices
