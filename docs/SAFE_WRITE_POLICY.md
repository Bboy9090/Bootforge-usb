# Safe Write Policy
## BootForge Read-Only Commitment

**Version**: 3.0.0
**Last Updated**: 2026-05-23
**Status**: Immutable Policy

---

## Policy Statement

**BootForge is a read-only USB device detection and analysis tool.**

BootForge, including all its components (libbootforge, CLI, GUI, services), **SHALL NOT**:
- Write data to USB devices
- Modify device firmware or bootloaders
- Erase or format storage media
- Modify partition tables or file systems
- Execute device-modifying commands
- Bypass device security mechanisms

This policy is permanent and non-negotiable for all BootForge versions.

---

## Scope

### What BootForge DOES (Allowed Operations)

✅ **Read-Only USB Operations**:
- Enumerate connected USB devices
- Read USB device descriptors (VID, PID, serial, manufacturer)
- Query device capabilities and configuration
- Monitor device connection/disconnection events
- Read device mode (DFU, recovery, fastboot, normal)

✅ **File System Operations** (Host OS Only):
- Read ISO/IMG files from host filesystem
- Calculate checksums (SHA256, MD5) of local files
- Verify checksum integrity against known-good values
- Write audit logs to host filesystem (append-only)

✅ **Simulation & Planning**:
- Generate dry-run write plans (without execution)
- Estimate write times and resource usage
- Validate file size vs. device capacity
- Simulate write operations (no actual device I/O)

✅ **Reporting & Logging**:
- Write session logs to local database
- Generate device analysis reports
- Export data in JSON/CSV/PDF formats
- Audit trail with hash-chained immutability

### What BootForge DOES NOT DO (Prohibited Operations)

❌ **Device Modification**:
- Writing data to USB storage
- Flashing firmware or bootloaders
- Modifying device configurations
- Sending vendor-specific write commands

❌ **Security Bypass**:
- Bootloader unlocking
- Device rooting or jailbreaking
- DRM circumvention
- Exploit execution

❌ **Destructive Operations**:
- Formatting or partitioning
- Data erasure or wiping
- Secure erase commands
- Low-level disk operations

❌ **Execution**:
- Running code on connected devices
- Installing applications or packages
- Modifying system partitions
- Bootloader/recovery mode activation (only detection)

---

## Technical Enforcement

### Code-Level Guarantees

1. **No Write APIs Exposed**:
   - `libbootforge` public API contains zero write functions
   - All USB operations use read-only rusb APIs
   - No bulk OUT, interrupt OUT, or control OUT endpoints used (except standard read requests)

2. **Rust Type System**:
   ```rust
   // Example: Scanner only returns immutable data
   pub struct DeviceScanner {
       context: rusb::Context,  // Read-only context
   }

   impl DeviceScanner {
       // Returns borrowed data, no mutation possible
       pub fn scan(&self) -> Result<Vec<ScannedDevice>> { ... }
   }
   ```

3. **rusb Read-Only Methods**:
   - `device_descriptor()` - reads device descriptor (USB GET_DESCRIPTOR)
   - `read_*()` - read string descriptors, configurations
   - `active_config_descriptor()` - read current configuration
   - **NEVER** uses: `write_*()`, `reset()`, `set_configuration()`, `claim_interface()`

4. **Compiler Warnings**:
   - Any attempt to use write APIs triggers compile-time errors
   - Static analysis (clippy) flags suspicious USB operations

### Runtime Verification

1. **Audit Logging**:
   - Every USB operation logged with operation type
   - Audit log parser can verify only read operations occurred
   - Log entries are immutable (hash-chained SHA256)

2. **Health Check Script**:
   ```bash
   # scripts/healthcheck.sh verifies:
   - USB devices detected (read success)
   - No write capability compiled into binary
   - Audit logs contain only read operations
   ```

3. **Integration Tests**:
   ```rust
   #[test]
   fn test_no_write_operations() {
       let scanner = DeviceScanner::new();
       let devices = scanner.scan().unwrap();

       // Assert: No device modification occurred
       // (Verified by external USB analyzer)
   }
   ```

---

## Use Case Scenarios

### Scenario 1: Device Diagnosis

**Goal**: Identify USB flash drive not recognized by OS.

**BootForge Actions**:
1. ✅ Scan USB bus, detect device at bus 1, address 5
2. ✅ Read descriptor: VID=0x0781 (SanDisk), PID=0x5581
3. ✅ Read serial number, manufacturer string
4. ✅ Report device in "mass storage" mode
5. ✅ Log to audit trail

**Result**: User knows device is detected, can troubleshoot driver/permission issues. **No data written.**

---

### Scenario 2: ISO Verification

**Goal**: Verify integrity of downloaded Ubuntu ISO before creating bootable USB.

**BootForge Actions**:
1. ✅ User selects ISO file from host filesystem
2. ✅ Calculate SHA256 checksum of ISO file
3. ✅ Compare against known-good Ubuntu checksum
4. ✅ Report: Match ✅ or Mismatch ❌
5. ✅ Log checksum operation to audit trail

**Result**: User knows ISO is valid. **No device touched.**

---

### Scenario 3: Write Planning (Dry-Run)

**Goal**: Prepare to write Ubuntu ISO to USB flash drive (using other tool).

**BootForge Actions**:
1. ✅ Scan and detect USB flash drive (16GB, SanDisk)
2. ✅ User selects Ubuntu ISO (4.2GB)
3. ✅ Validate: ISO size < device capacity ✅
4. ✅ Generate dry-run plan:
   - Source: /path/to/ubuntu.iso (4.2GB)
   - Target: Bus 1, Address 5, SanDisk 16GB
   - Estimated time: 8 minutes (USB 2.0 write speed)
   - Block count: 8,640 blocks (512KB each)
5. ✅ Export plan as JSON
6. ✅ **NO WRITE EXECUTED**

**Result**: User has detailed plan, can execute write with another tool (e.g., `dd`, Etcher). **BootForge did not write.**

---

## Edge Cases & Clarifications

### Q: Can BootForge write audit logs?

**A**: Yes, to the **host filesystem only**, not to USB devices.
- Audit logs are written to local SQLite database (e.g., `/var/lib/bootforge/audit.db`)
- Never writes logs to connected USB devices
- Logs are append-only and cryptographically tamper-evident

---

### Q: Can BootForge modify device firmware in "DFU mode"?

**A**: **No.** BootForge can *detect* DFU mode, but cannot and will not:
- Send DFU download commands
- Upload firmware images to device
- Trigger DFU state transitions
- Modify device flash memory

DFU mode is detected via VID/PID and device class, but no DFU protocol commands are sent.

---

### Q: What if a user requests BootForge to "unlock" a device?

**A**: **Refused.** BootForge cannot and will not:
- Unlock bootloaders (e.g., `fastboot oem unlock`)
- Bypass FRP (Factory Reset Protection)
- Root or jailbreak devices
- Execute exploits or security bypasses

Such requests are **out of scope** and **violate the Safe Write Policy**.

---

### Q: Can BootForge "prepare" a device for writing?

**A**: Only via read-only analysis and reporting:
- ✅ Detect device mode (normal, DFU, recovery)
- ✅ Report device capacity and file system type
- ✅ Validate source file integrity (checksum)
- ✅ Generate write plan (dry-run, no execution)
- ❌ **Does NOT** format, partition, or write to device

User must use separate tools (e.g., `dd`, Etcher, Rufus) to execute writes.

---

## Compliance & Auditability

### Regulatory Compliance

BootForge's read-only policy ensures compliance with:
- **GDPR**: No data modification, only observation (lawful processing)
- **CFAA (US)**: No unauthorized access or modification of devices
- **DMCA**: No circumvention of technological protection measures
- **Industry Standards**: ISO 27001 (data integrity), NIST (least privilege)

### Audit Trail Verification

Audit logs can prove BootForge did not modify devices:

```bash
# Extract all operations from audit log
cargo run --bin audit-verify -- --log /var/lib/bootforge/audit.db

# Output:
# [2026-05-23 15:30:00] DeviceConnected: Bus 1, Addr 5, VID:PID 0x0781:0x5581
# [2026-05-23 15:30:01] DescriptorRead: Serial=AA12345678
# [2026-05-23 15:30:15] ChecksumCalculated: ubuntu.iso, SHA256=abc123...
# [2026-05-23 15:31:00] DryRunGenerated: Plan ID 1234
#
# Verification: ✅ All operations are read-only
# Hash Chain: ✅ Intact (no tampering detected)
```

---

## Exceptions & Waivers

**There are NO exceptions to the Safe Write Policy.**

- No "advanced mode" that enables writes
- No hidden developer options
- No privileged access that bypasses read-only
- No future versions will add write capability

If write capability is needed, users must:
1. Use BootForge for read-only analysis
2. Export dry-run plan
3. Use dedicated write tools (dd, Etcher, Rufus, etc.)
4. Log write operation with external audit trail

---

## Incident Response

### If Write Operation Detected

If any write operation is detected in BootForge:

1. **Immediate Actions**:
   - Halt all operations
   - Isolate affected devices
   - Preserve audit logs and device state

2. **Investigation**:
   - Review audit logs for evidence
   - Analyze code for unexpected behavior
   - Check for malicious code injection or tampering

3. **Remediation**:
   - Patch vulnerability immediately
   - Release emergency update
   - Notify users via security advisory
   - Conduct post-mortem analysis

4. **Reporting**:
   - Disclose vulnerability per SECURITY.md
   - Publish CVE (if applicable)
   - Update Safe Write Policy with lessons learned

### Reporting Violations

If you discover BootForge performing write operations:

- **Email**: security@bootforge.io (PGP key available)
- **GitHub**: File issue with [SECURITY] tag
- **Severity**: Treat as critical security vulnerability

See [SECURITY.md](../SECURITY.md) for responsible disclosure policy.

---

## Design Philosophy

### Why Read-Only?

1. **Safety First**: Prevents accidental data loss
2. **Trust**: Users trust BootForge won't modify their devices
3. **Compliance**: Legal and regulatory requirements
4. **Separation of Concerns**: Analysis separate from action
5. **Auditability**: Read-only operations are easier to audit

### Principle of Least Privilege

BootForge requests **only** the permissions necessary:
- Read USB device descriptors
- Read local files
- Write local audit logs

BootForge **does not** request:
- Root/admin privileges (unless required by OS for USB read)
- Raw disk access
- Network access (for MVP)

---

## Future Considerations

### Planned Features (Still Read-Only)

- ✅ Advanced device fingerprinting (read descriptors + analyze)
- ✅ Real-time device monitoring (read connection events)
- ✅ Device health assessment (read SMART data, if available via USB)
- ✅ Compliance reporting (read operations, generate reports)

### Will NEVER Add

- ❌ Write operations of any kind
- ❌ Firmware flashing
- ❌ Bootloader modification
- ❌ Security bypass tools

---

## Acknowledgments

This policy is inspired by:
- **GNU ddrescue**: Read-only recovery tool philosophy
- **lsusb**: USB inspection without modification
- **hdparm**: Disk parameter reading (read-only mode)

---

## Document History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-05-23 | Initial Safe Write Policy for MVP v3.0.0 |

---

**This policy is immutable. Any violation is a critical security incident.**

For questions, contact: security@bootforge.io
