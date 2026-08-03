# libbootforge Public API v1 Contract

## Status

This document defines the target compatibility contract for the first stable forensic USB API. Existing code may be adapted toward this contract before version 1.0 is declared stable.

## Stability rules

- Public structures are versioned and serializable.
- New optional fields may be added in compatible releases.
- Existing fields, enum meanings, and event semantics do not change within v1.
- Unknown enum values must remain representable during deserialization.
- Platform-specific evidence is namespaced and never silently folded into an unrelated generic field.
- All inferred results include confidence and evidence.

## Primary types

```rust
pub struct UsbManager;
pub struct UsbWatcher;
pub struct DeviceSnapshot;
pub struct DeviceIdentity;
pub struct IdentityEvidence;
pub struct DeviceTopology;
pub struct DriverReport;
pub struct ProtocolReport;
pub struct HealthReport;
pub struct ForensicEvent;
pub struct SessionRecorder;
```

## Manager

```rust
impl UsbManager {
    pub fn new() -> Result<Self>;
    pub fn platform() -> Platform;
    pub fn enumerate(&self) -> Result<Vec<DeviceSnapshot>>;
    pub fn enumerate_with(&self, options: EnumerationOptions)
        -> Result<Vec<DeviceSnapshot>>;
    pub fn watcher(&self, options: WatchOptions) -> Result<UsbWatcher>;
}
```

Enumeration is read-only. A failed optional enrichment must not discard an otherwise valid device snapshot. Limitations and field-level errors are attached to the snapshot.

## Device snapshot

```rust
pub struct DeviceSnapshot {
    pub schema_version: String,
    pub observed_at_utc: DateTime<Utc>,
    pub platform: Platform,
    pub identity: DeviceIdentity,
    pub descriptors: DescriptorSet,
    pub topology: DeviceTopology,
    pub interfaces: Vec<UsbInterface>,
    pub driver: Option<DriverReport>,
    pub protocols: Vec<ProtocolReport>,
    pub health: HealthReport,
    pub storage: Option<StorageCorrelation>,
    pub raw_evidence: Vec<RawEvidence>,
    pub limitations: Vec<CollectionLimitation>,
}
```

## Identity

```rust
pub struct DeviceIdentity {
    pub stable_id: String,
    pub confidence: u8,
    pub grade: IdentityGrade,
    pub evidence: Vec<IdentityEvidence>,
    pub conflicts: Vec<IdentityConflict>,
}

pub enum IdentityGrade {
    Exact,
    Probable,
    Weak,
    Unknown,
    Other(String),
}
```

`stable_id` is opaque. Applications may store and compare it but must not parse internal components.

## Events

```rust
pub struct ForensicEvent {
    pub schema_version: String,
    pub event_id: String,
    pub sequence: u64,
    pub timestamp_utc: DateTime<Utc>,
    pub elapsed_ns: u128,
    pub source: EventSource,
    pub kind: EventKind,
    pub stable_id: Option<String>,
    pub identity_confidence: Option<u8>,
    pub snapshot: Option<DeviceSnapshot>,
    pub previous_snapshot: Option<DeviceSnapshot>,
    pub correlation: Option<CorrelationReport>,
    pub limitations: Vec<CollectionLimitation>,
}

pub enum EventKind {
    Discovered,
    Attached,
    DescriptorAvailable,
    InterfaceAvailable,
    DriverBound,
    DriverChanged,
    Mounted,
    Unmounted,
    ProtocolDetected,
    ModeChanged,
    HealthChanged,
    Disconnected,
    Reconnected,
    IdentityConflict,
    EnumerationFailed,
    PermissionLimited,
    Other(String),
}
```

## Watcher

```rust
impl UsbWatcher {
    pub fn next_event(&mut self) -> Result<ForensicEvent>;
    pub fn try_next_event(&mut self) -> Result<Option<ForensicEvent>>;
    pub fn subscribe(&self) -> EventReceiver;
    pub fn snapshot(&self) -> Vec<DeviceSnapshot>;
    pub fn stop(self) -> Result<()>;
}
```

Event order is guaranteed per watcher instance. A reconnect is emitted only when the correlation engine can explain the match; otherwise a disconnect followed by a new attach is emitted.

## Protocol report

```rust
pub struct ProtocolReport {
    pub protocol: ProtocolKind,
    pub confidence: u8,
    pub passive: bool,
    pub mode: Option<String>,
    pub evidence: Vec<ProtocolEvidence>,
}

pub enum ProtocolKind {
    Adb,
    Fastboot,
    AppleMobile,
    AppleRecovery,
    AppleRestore,
    AppleDfu,
    Mtp,
    Ptp,
    UsbDfu,
    CdcAcm,
    Hid,
    MassStorage,
    SerialBridge,
    Other(String),
}
```

Passive protocol reports are allowed in the core. Any active handshake must require a separate explicit feature and API surface.

## Driver report

```rust
pub struct DriverReport {
    pub status: DriverStatus,
    pub name: Option<String>,
    pub provider: Option<String>,
    pub version: Option<String>,
    pub date: Option<String>,
    pub signature: SignatureStatus,
    pub service: Option<String>,
    pub filters: Vec<String>,
    pub problem_code: Option<String>,
    pub expected_binding: Option<String>,
    pub evidence: Vec<DriverEvidence>,
}
```

The core reports driver state. It does not install, update, replace, or remove drivers.

## Health report

```rust
pub struct HealthReport {
    pub score: Option<u8>,
    pub grade: HealthGrade,
    pub confidence: u8,
    pub window: ObservationWindow,
    pub observations: Vec<HealthObservation>,
    pub unavailable_metrics: Vec<String>,
}
```

A missing score is valid when evidence is insufficient. The library must prefer `None` over fabricated precision.

## Recording and export

```rust
impl SessionRecorder {
    pub fn new(metadata: SessionMetadata) -> Self;
    pub fn record(&mut self, event: &ForensicEvent) -> Result<()>;
    pub fn export_json(&self, path: impl AsRef<Path>) -> Result<()>;
    pub fn export_jsonl(&self, path: impl AsRef<Path>) -> Result<()>;
    pub fn export_csv(&self, path: impl AsRef<Path>) -> Result<()>;
}
```

JSON and JSON Lines are canonical. CSV is a reduced representation and may omit nested evidence.

## Error model

```rust
pub enum BootforgeError {
    BackendUnavailable,
    PermissionDenied,
    DeviceVanished,
    DescriptorUnavailable,
    EnumerationFailed,
    UnsupportedPlatform,
    InvalidEvidence,
    Serialization,
    Timeout,
    Other(String),
}
```

Errors must preserve source context without exposing secrets or unstable platform pointers.

## Platform support

```rust
pub enum Platform {
    Windows,
    Linux,
    MacOs,
    Arcwyre,
    Other(String),
}
```

ARCWYRE is a first-class backend, not treated as Linux by assumption.

## Feature flags

Suggested Cargo features:

```text
default = ["passive-core"]
passive-core
windows-setupapi
linux-udev
macos-iokit
arcwyre-native
recording
hash-chain
active-probes
ffi
python
```

`active-probes` is disabled by default.

## Compatibility promise

Version 1.0 will be declared only after:

- all public types have serialization round-trip tests
- event ordering and reconnect semantics have fixture tests
- Windows, Linux, macOS, and ARCWYRE backend contracts pass
- seven-day watcher soak tests complete without leaks or unbounded growth
- the safety contract is enforced by tests and documentation
