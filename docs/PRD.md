# Product Requirements Document (PRD)
## BootForge Reforged MVP

**Version**: 3.0.0
**Status**: In Development
**Last Updated**: 2026-05-23

## Executive Summary

BootForge is a cross-platform USB device detection and enumeration tool designed for diagnostic and read-only device analysis. The Reforged MVP establishes the foundation for Blue Phoenix OS / Bobby's World device ecosystem with standardized production features.

## Product Vision

Provide a compliance-first, ownership-respecting platform for USB device detection, classification, and analysis without any destructive or modifying operations.

## Target Users

- Device repair technicians
- Hardware diagnostics engineers
- Device recovery specialists
- Lab and QA engineers
- System administrators managing device fleets

## MVP Scope (v3.0.0)

### Core Features

#### 1. USB Device Detection (Read-Only)
- Enumerate all connected USB devices
- Read device descriptors without modification
- Support for Linux, macOS, and Windows platforms
- Real-time device connection/disconnection monitoring

**Acceptance Criteria**:
- Detects all removable USB devices on supported platforms
- No write operations to devices
- Returns complete device descriptor information
- Handles missing or unplugged devices gracefully

#### 2. Device Information Display
- Vendor ID (VID) and Product ID (PID)
- Manufacturer and product strings
- Serial numbers
- Device class and protocol information
- Bus/port location details
- Connection speed and configuration

**Acceptance Criteria**:
- All descriptor fields accurately extracted
- Human-readable display format
- JSON export capability for automation
- Handles devices with incomplete descriptors

#### 3. ISO/Image Selection (Stub)
- File browser for selecting ISO/IMG files
- File path validation
- Supported format detection (ISO, IMG, DMG)
- File size and metadata display

**Acceptance Criteria**:
- File selection dialog works on all platforms
- Validates file exists and is readable
- Displays file metadata (size, type, checksum)
- **Does NOT write to device** (stub for future implementation)

#### 4. Checksum Verification
- Calculate SHA256 checksums for selected files
- Verify checksums against known-good values
- Display verification status
- Support for checksum files (.sha256, .md5)

**Acceptance Criteria**:
- Accurately calculates SHA256 for files up to 16GB
- Supports common checksum file formats
- Clear pass/fail indication
- Progress reporting for large files

#### 5. Dry-Run Write Generation
- Simulate write operations without touching hardware
- Generate write plan with block-level details
- Estimate write time and resource usage
- Validate source/target compatibility

**Acceptance Criteria**:
- Produces detailed write plan without device modification
- Validates file size vs. device capacity
- Detects incompatible device configurations
- **Never executes actual write** (read-only guarantee)

#### 6. Audit Logging
- Immutable audit trail for all operations
- SHA256 hash-chained log entries
- Timestamps with ISO 8601 format
- Session-based log organization

**Acceptance Criteria**:
- Every operation logged with timestamp and details
- Logs are tamper-evident via hash chaining
- Logs exportable in JSON format
- Logs stored in read-only append mode

### Non-Functional Requirements

#### Performance
- Device scan completes in < 2 seconds
- Checksum calculation: > 100 MB/s on SSD
- UI remains responsive during long operations
- Memory usage < 100 MB for typical workload

#### Reliability
- Zero crashes during 8-hour operation
- Graceful handling of device disconnection
- Automatic error recovery for transient USB errors
- Comprehensive error messages for user action

#### Security
- No elevation/root required for read operations
- No modification of device firmware or data
- Audit logs are cryptographically verifiable
- Secure handling of sensitive device information

#### Usability
- CLI tool usable by advanced users
- Clear, actionable error messages
- Progressive disclosure of advanced features
- Consistent terminology (BootForge public, libbootforge internal)

## Out of Scope (MVP)

The following features are explicitly **NOT included** in MVP:

- ❌ Destructive disk operations (format, partition, write)
- ❌ Device firmware modification or flashing
- ❌ Bootloader unlocking or security bypass
- ❌ Advanced GUI (Tauri app planned for future)
- ❌ Network-based device discovery
- ❌ Cloud backup/restore features
- ❌ Multi-device batch operations
- ❌ Custom partition table editing

## Technical Architecture

### Components

1. **libbootforge**: Core USB detection library (Rust)
2. **bootforge-cli**: Command-line interface
3. **ForgeWorks Services**: Compliance and audit microservices
4. **Database Layer**: SQLite for session/audit storage

### Technology Stack

- **Language**: Rust 2021 edition
- **USB Library**: rusb 0.9 (cross-platform libusb wrapper)
- **Serialization**: serde + serde_json
- **Cryptography**: sha2 for checksums and audit chains
- **Database**: SQLite (embedded)

### Platform Support Matrix

| Platform | Read USB | Enumerate | Events | Notes |
|----------|----------|-----------|--------|-------|
| Linux    | ✅       | ✅        | ✅     | Requires libusb-1.0 |
| macOS    | ✅       | ✅        | ✅     | IOKit native support |
| Windows  | ✅       | ✅        | ✅     | WinUSB/libusb-win32 |

## Success Metrics

### Launch Criteria (MVP v3.0.0)

- ✅ All healthcheck.sh tests pass
- ✅ All smoke-test.sh tests pass
- ✅ Zero compiler warnings in release build
- ✅ 100% of libbootforge unit tests pass
- ✅ Documentation complete (README, docs/*)
- ✅ Packaging instructions verified on 2+ platforms

### Key Performance Indicators (KPIs)

- **Reliability**: 99.9% successful device detection rate
- **Performance**: < 2s device scan time
- **Quality**: Zero critical bugs in first 30 days
- **Adoption**: 10+ successful device enumerations by beta users

## Dependencies

### External Dependencies
- libusb 1.0+ (system library)
- Rust toolchain 1.70+
- Platform-specific USB drivers

### Internal Dependencies
- audit-logging service for tamper-evident logs
- device-analysis service for capability classification
- legal-classification service for jurisdiction awareness

## Risks and Mitigations

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| USB driver incompatibility | High | Medium | Extensive platform testing, fallback strategies |
| Performance on slow USB 2.0 | Medium | High | Asynchronous operations, progress indicators |
| Audit log storage growth | Low | High | Log rotation, compression, retention policies |
| User expects write capability | Medium | Medium | Clear UI/docs stating read-only limitation |

## Release Checklist

See [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md) for detailed pre-release validation steps.

## Future Roadmap

See [ROADMAP.md](ROADMAP.md) for planned features beyond MVP.

## Glossary

- **BootForge**: Public-facing application name
- **libbootforge**: Internal Rust library name
- **Descriptor**: USB device metadata (VID/PID/serial/etc.)
- **DFU**: Device Firmware Update mode
- **Dry-Run**: Simulated operation without actual execution
- **ForgeWorks**: Compliance and analysis engine layer
- **MVP**: Minimum Viable Product (v3.0.0)

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2026-05-23 | BootForge Team | Initial MVP PRD |
