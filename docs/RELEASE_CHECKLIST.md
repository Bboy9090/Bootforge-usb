# Release Checklist
## BootForge v3.0.0 Reforged MVP

**Purpose**: Ensure all critical validations pass before releasing BootForge to production.

---

## Pre-Release Validation

### 1. Code Quality

- [ ] **Build**: `cargo build --release` completes without errors
- [ ] **Warnings**: Zero compiler warnings in release mode
- [ ] **Format**: `cargo fmt --check` passes (code formatted)
- [ ] **Lint**: `cargo clippy -- -D warnings` passes (no clippy warnings)
- [ ] **Dependencies**: All Cargo.lock versions are up-to-date
- [ ] **Security**: `cargo audit` shows no known vulnerabilities

**Commands**:
```bash
cargo build --release
cargo fmt --check
cargo clippy -- -D warnings
cargo audit
```

---

### 2. Testing

- [ ] **Unit Tests**: All unit tests pass (`cargo test --lib`)
- [ ] **Integration Tests**: Integration tests pass (`cargo test --test '*'`)
- [ ] **libbootforge**: 100% of libbootforge tests pass
- [ ] **Services**: All service tests pass
- [ ] **Examples**: All examples compile and run
- [ ] **Hardware Tests**: Manual verification with real USB devices

**Commands**:
```bash
cargo test --workspace
cargo test -p libbootforge
cargo test -p audit-logging
cargo test -p device-analysis
cargo run --example list_devices
```

**Known Issues**:
- `audit-logging::test_verify_integrity_fails_on_tamper` may fail (tracked)
- Event monitoring tests are ignored (require USB hardware)

---

### 3. Health Checks

- [ ] **Healthcheck**: `./scripts/healthcheck.sh` passes all checks
- [ ] **Smoke Test**: `./scripts/smoke-test.sh` passes all builds
- [ ] **USB Detection**: Detects at least one USB device on test machine
- [ ] **Safe Mode**: Confirms no write operations are possible

**Commands**:
```bash
./scripts/healthcheck.sh
./scripts/smoke-test.sh
```

---

### 4. Documentation

- [ ] **README.md**: Updated with accurate build/test/usage instructions
- [ ] **PRD.md**: Product requirements documented
- [ ] **ROADMAP.md**: Future roadmap published
- [ ] **USB_DISCOVERY_MODEL.md**: Technical architecture documented
- [ ] **SAFE_WRITE_POLICY.md**: Read-only policy documented
- [ ] **RELEASE_CHECKLIST.md**: This file is complete
- [ ] **Packaging README**: Platform packaging instructions complete
- [ ] **Changelog**: Version 3.0.0 changes documented

**Files**:
- `/home/runner/work/Bootforge-usb/Bootforge-usb/README.md`
- `/home/runner/work/Bootforge-usb/Bootforge-usb/docs/PRD.md`
- `/home/runner/work/Bootforge-usb/Bootforge-usb/docs/ROADMAP.md`
- `/home/runner/work/Bootforge-usb/Bootforge-usb/docs/USB_DISCOVERY_MODEL.md`
- `/home/runner/work/Bootforge-usb/Bootforge-usb/docs/SAFE_WRITE_POLICY.md`
- `/home/runner/work/Bootforge-usb/Bootforge-usb/packaging/README.md`

---

### 5. Metadata & Configuration

- [ ] **app.metadata.json**: Created with package ID `com.bobbysworld.bootforge`
- [ ] **Version Numbers**: All Cargo.toml files show version 3.0.0
- [ ] **License**: MIT/Apache-2.0 dual license confirmed
- [ ] **Authors**: Contributor list is accurate
- [ ] **Repository**: GitHub URL is correct

**Files**:
- `/home/runner/work/Bootforge-usb/Bootforge-usb/app.metadata.json`
- `/home/runner/work/Bootforge-usb/Bootforge-usb/Cargo.toml`

---

### 6. Platform Verification

Test on each supported platform:

#### Linux
- [ ] Ubuntu 22.04 LTS: Build, test, run
- [ ] Fedora 38: Build, test, run
- [ ] Arch Linux: Build, test, run
- [ ] libusb-1.0 installed and detected

#### macOS
- [ ] macOS 13 Ventura: Build, test, run
- [ ] macOS 14 Sonoma: Build, test, run
- [ ] Native IOKit support verified

#### Windows
- [ ] Windows 10: Build, test, run
- [ ] Windows 11: Build, test, run
- [ ] WinUSB/libusb-win32 driver verified

---

### 7. Functionality Validation

Manual testing of MVP features:

- [ ] **USB Detection**: Enumerate all connected devices
- [ ] **Descriptor Display**: Show VID/PID/serial/manufacturer
- [ ] **ISO Selection**: Select ISO file (stub, no write)
- [ ] **Checksum Verification**: Calculate and verify SHA256
- [ ] **Dry-Run Generation**: Generate write plan without modification
- [ ] **Audit Logging**: All operations logged to audit trail

**Test Cases**:
1. Connect USB flash drive → detected with correct VID/PID
2. Select ISO file → file metadata displayed correctly
3. Verify checksum → SHA256 calculated and verified
4. Generate dry-run → write plan generated, no device modified
5. Check audit log → all operations logged with timestamps

---

### 8. Security & Compliance

- [ ] **No Root Required**: Read operations work without sudo/elevation
- [ ] **No Write Operations**: Verified no device modification possible
- [ ] **Audit Immutability**: Audit logs are hash-chained and tamper-evident
- [ ] **Data Privacy**: No sensitive data logged or transmitted
- [ ] **Dependency Audit**: No known security vulnerabilities

**Verification**:
```bash
# Run without sudo - should succeed for read operations
cargo run --bin bootforge-cli

# Verify audit log hash chain
cargo test -p audit-logging
```

---

### 9. Performance Benchmarks

- [ ] **Scan Time**: Device scan completes in < 2 seconds
- [ ] **Checksum Speed**: > 100 MB/s on SSD
- [ ] **Memory Usage**: < 100 MB for typical workload
- [ ] **Startup Time**: CLI startup in < 500ms

**Measurement**:
```bash
# Time device scan
time cargo run --bin bootforge-cli

# Memory profiling (if available)
cargo instruments -t alloc --bin bootforge-cli
```

---

### 10. Packaging & Distribution

- [ ] **Binary Size**: Release binary < 10 MB
- [ ] **Cross-Compilation**: Can cross-compile for all platforms
- [ ] **Installers**: Platform-specific installers created (if applicable)
- [ ] **MSIX Package**: Windows MSIX package validated (see packaging/README.md)
- [ ] **Blue Phoenix OS**: Integration notes documented

**Build Artifacts**:
```bash
# Release builds
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-apple-darwin
cargo build --release --target x86_64-pc-windows-msvc

# Check binary size
ls -lh target/release/bootforge-cli
```

---

### 11. Naming Consistency

- [ ] **Public Name**: All user-facing text says "BootForge" (not libbootforge)
- [ ] **Internal Name**: All code uses "libbootforge" for library
- [ ] **Consistency**: README, docs, CLI output use correct names
- [ ] **Package IDs**: com.bobbysworld.bootforge in metadata

**Files to Check**:
- CLI help text (`--help`)
- README.md
- Documentation files
- app.metadata.json

---

### 12. Final Sign-Off

- [ ] **Team Review**: Code reviewed by at least one other developer
- [ ] **Manual Testing**: Full manual test pass on 2+ platforms
- [ ] **Documentation Review**: All docs reviewed for accuracy
- [ ] **Release Notes**: Version 3.0.0 release notes drafted
- [ ] **Git Tag**: Tagged as `v3.0.0` in repository
- [ ] **Changelog**: CHANGELOG.md updated with release date

**Release Process**:
```bash
# Tag release
git tag -a v3.0.0 -m "BootForge Reforged MVP v3.0.0"
git push origin v3.0.0

# Create GitHub release
gh release create v3.0.0 --title "BootForge v3.0.0 - Reforged MVP" --notes-file RELEASE_NOTES.md
```

---

## Post-Release Monitoring

After release, monitor for:

- [ ] **Issue Reports**: GitHub issues for bugs or problems
- [ ] **Crash Reports**: User-reported crashes or panics
- [ ] **Performance**: Slow operations or high memory usage
- [ ] **Compatibility**: Platform-specific issues
- [ ] **Security**: Vulnerability reports via SECURITY.md

**Hotfix Criteria**:
- Critical security vulnerability
- Data corruption or loss
- Complete failure on supported platform
- Regression from previous version

---

## Checklist Completion

**Date**: _______________
**Released By**: _______________
**Version**: v3.0.0

All items above must be checked before release. Document any exceptions or waivers with justification.

---

## Document History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-05-23 | Initial release checklist for MVP v3.0.0 |
