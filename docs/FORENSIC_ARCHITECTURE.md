# BootForge USB Forensic Architecture

## Mission

BootForge USB is a read-only-first, cross-platform USB intelligence library for Windows, Linux, macOS, and ARCWYRE. Its job is to observe, normalize, correlate, and report USB device state without modifying devices or user data.

## Design principles

1. Evidence before assumption.
2. Stable identity must expose confidence, never pretend certainty.
3. Platform-specific enrichment must preserve the raw source data.
4. Every event must be timestamped, ordered, serializable, and traceable.
5. Protocol detection must be passive unless a caller explicitly opts into a separately reviewed active probe.
6. The core library remains GUI-independent and non-destructive.
7. Unsupported data is reported as unavailable, not invented.

## Layer model

```text
Applications and SDK bindings
        |
Public libbootforge API
        |
Event correlation and forensic timeline
        |
Identity | Protocol classification | Health | Driver intelligence
        |
Normalized USB model
        |
Windows | Linux | macOS | ARCWYRE backends
        |
Operating-system USB facilities and libusb
```

## Core subsystems

### 1. Platform backends

Each backend gathers the strongest read-only evidence its platform exposes.

- Windows: SetupAPI, Configuration Manager, device interfaces, registry properties, WinUSB visibility, driver status, container IDs, location paths.
- Linux: sysfs, udev, usbfs/libusb, kernel driver binding, bus and port topology, mount and block correlation when applicable.
- macOS: IOKit USB registry, IORegistry location data, interface classes, BSD name correlation where applicable.
- ARCWYRE: native kernel/device-manager enumeration, controller and port topology, driver binding, TruthLog-compatible event source.

Backends return raw evidence plus a normalized record. Raw platform fields remain available for forensic review.

### 2. Normalized device model

The normalized model should include, where observable:

- vendor ID and product ID
- device, USB, and binary-coded firmware versions
- manufacturer, product, and serial strings
- bus, address, port chain, hub chain, and controller identity
- device, interface, subclass, and protocol codes
- configurations, interfaces, alternate settings, endpoints, and transfer types
- negotiated speed and advertised capability
- power characteristics
- platform path, location path, container identity, and driver binding
- mount, volume, filesystem, and block-device correlation when applicable
- protocol classifications and confidence
- health observations
- identity fingerprint and confidence

### 3. Stable identity engine

A device fingerprint is built from ranked evidence rather than one fragile field.

Strong evidence:

- cryptographically unique or manufacturer-issued serial number
- Windows container ID
- platform persistent registry identity
- ARCWYRE persistent device identity

Moderate evidence:

- VID/PID + serial + interface set
- VID/PID + stable physical topology + product strings
- storage volume UUID or filesystem UUID combined with USB identity

Weak evidence:

- VID/PID alone
- transient bus address
- mount point or COM number

The engine returns:

- stable ID
- confidence score from 0 to 100
- evidence list
- conflict list
- whether the correlation is exact, probable, weak, or unknown

No caller may receive a bare boolean claiming two devices are identical without correlation evidence.

### 4. Watcher and correlation engine

The watcher converts platform notifications and periodic reconciliation into ordered events.

Required event kinds:

- discovered
- attached
- descriptor_available
- interface_available
- driver_bound
- driver_changed
- mounted
- unmounted
- protocol_detected
- mode_changed
- health_changed
- disconnected
- reconnected
- identity_conflict
- enumeration_failed
- permission_limited

Reconnect events must include the prior stable ID, correlation score, elapsed time, old path, new path, and evidence used.

### 5. Protocol classification

Passive classification foundations:

- ADB
- Fastboot
- Apple normal, recovery, restore, and DFU families
- MTP
- PTP
- USB DFU class
- CDC ACM and related CDC profiles
- HID
- mass storage
- USB serial bridges

Classification sources include VID/PID databases, interface class tuples, descriptors, driver bindings, and known platform service visibility. Results contain confidence and supporting evidence.

Active protocol handshakes are outside the passive core by default and must be separately feature-gated.

### 6. Driver intelligence

Driver reporting should normalize:

- bound driver name
- provider
- version
- date
- signature status where exposed
- service and filter drivers
- problem or error code
- expected versus observed binding
- WinUSB/libusb compatibility visibility
- kernel module or IOKit driver identity

The library reports facts and remediation hints, but does not install or replace drivers.

### 7. Health and reliability observations

BootForge USB must distinguish observed facts from inferred health.

Observable signals may include:

- repeated disconnects and reconnects
- enumeration failures
- descriptor read failures
- resets and re-enumeration
- speed fallback or unstable negotiation
- power insufficiency warnings
- driver start failures
- endpoint or transfer errors exposed by the operating system
- storage SMART data only when safely exposed through a USB bridge

The health model returns:

- score
- grade
- observations
- unavailable metrics
- confidence
- time window

It must never claim flash wear, bad blocks, CRC failures, or remaining life unless a trustworthy source actually exposes them.

### 8. Forensic event record

Every record includes:

- schema version
- event ID
- monotonic sequence number
- wall-clock timestamp in UTC
- monotonic timestamp or elapsed time
- source platform and backend
- event kind
- stable device ID and identity confidence
- current normalized snapshot
- previous snapshot when relevant
- raw evidence references
- correlation explanation
- collection limitations

Exports:

- JSON Lines for streaming and evidence pipelines
- JSON for complete sessions
- CSV for basic analysis

Hash chaining may be provided as an optional integrity layer. Signing belongs to a higher-level evidence package, not hidden inside ordinary enumeration.

## Performance targets

- initial enumeration: under 250 ms on ordinary systems excluding slow descriptor timeouts
- event publication: under 100 ms after receipt of an OS notification
- idle watcher CPU: below 0.5 percent on representative hardware
- bounded memory over seven-day watcher runs
- no leaked handles after repeated attach and detach cycles
- deterministic serialization for the same normalized evidence

## Reliability gates

- unit tests for normalization, identity scoring, protocol classification, and health scoring
- recorded fixture tests for all four platforms
- hardware matrix tests with hubs, composite devices, phones, storage, serial adapters, cameras, and unstable devices
- 24-hour, 72-hour, and seven-day watcher soak tests
- reconnect tests across port changes, hub changes, reboot, driver changes, and mode transitions
- permission-denied and partial-evidence tests
- fuzz testing for descriptor and platform-property parsers

## Public API rule

The public API exposes normalized evidence and confidence. Platform internals remain available through explicit raw-evidence structures, but applications must not depend on undocumented backend quirks.

## Safety contract

The core performs no formatting, flashing, firmware modification, partition changes, unlocking, bypassing, driver installation, or user-data alteration. Read-only inspection is the default and defining boundary.
