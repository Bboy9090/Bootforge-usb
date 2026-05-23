# BootForge Roadmap

**Last Updated**: 2026-05-23
**Current Version**: 3.0.0 (MVP)

## Overview

This roadmap outlines the planned evolution of BootForge from the initial Reforged MVP to a comprehensive device analysis and management platform. All development maintains the core principle: **read-only, compliance-first, ownership-respecting**.

---

## Released

### v3.0.0 - Reforged MVP (Current)
**Target**: Q2 2026
**Status**: In Development

**Features**:
- ✅ USB device detection (read-only)
- ✅ Device descriptor display (VID/PID/serial/etc.)
- ✅ ISO/image selection (stub, no write)
- ✅ SHA256 checksum verification
- ✅ Dry-run write planning
- ✅ Basic audit logging
- ✅ Cross-platform support (Linux/macOS/Windows)
- ✅ CLI tool (bootforge-cli)
- ✅ Health check and smoke test scripts

---

## Planned Releases

### v3.1.0 - Enhanced Audit & Compliance
**Target**: Q3 2026
**Theme**: Production-grade audit trails and compliance reporting

**Features**:
- Advanced audit logging with query API
- Compliance report generation (PDF/HTML)
- Jurisdiction-aware device classification
- Ownership attestation framework
- Authority routing pathways (OEM/carrier/court)
- SAML/OIDC authentication integration
- Role-based access control (RBAC)

**Technical**:
- PostgreSQL support for enterprise deployments
- Audit log retention and archival policies
- Cryptographic log verification tools
- Multi-user session management

---

### v3.2.0 - Workshop UI (Desktop App)
**Target**: Q4 2026
**Theme**: User-friendly desktop interface

**Features**:
- Tauri-based desktop application
- Visual device browser with live updates
- Interactive descriptor viewer
- File selection with drag-and-drop
- Checksum verification progress UI
- Session history browser
- Dark/light theme support

**Technical**:
- React 18+ frontend
- TypeScript 5+ with strict mode
- Integration with libbootforge via Tauri IPC
- Platform-native installers (MSI, DMG, DEB)

---

### v3.3.0 - Device Fingerprinting & Classification
**Target**: Q1 2027
**Theme**: Advanced device intelligence

**Features**:
- Device family fingerprinting with confidence scores
- Known device profile database (expandable)
- Device mode detection (DFU, recovery, fastboot, etc.)
- Vendor-specific protocol classification
- Device capability analysis (without modification)
- Risk assessment framework
- Device history tracking across sessions

**Technical**:
- Machine learning models for device classification
- Extensible profile plugin system
- Performance optimizations for large device databases
- Privacy-preserving device telemetry

---

### v3.4.0 - Automation & Integration
**Target**: Q2 2027
**Theme**: Workflow automation and CI/CD integration

**Features**:
- RESTful API for device enumeration
- Webhook notifications for device events
- Bulk device scanning and reporting
- JSON/YAML configuration for batch operations
- Integration with lab management systems
- Automated test fixture support
- Command-line scripting improvements

**Technical**:
- OpenAPI 3.0 specification
- GraphQL query support (optional)
- Docker container images
- Kubernetes deployment manifests
- Prometheus metrics exporter

---

### v3.5.0 - ForgeCore Hardware Integration
**Target**: Q3 2027
**Theme**: Custom USB diagnostic bridge

**Features**:
- ForgeCore device support (USB diagnostic bridge)
- Secure element integration (ATECC608B)
- Smart thermal platform control
- Precision tool matrix automation
- Hardware-accelerated checksums
- Real-time device monitoring
- Lab equipment API

**Technical**:
- ForgeCore firmware SDK
- Hardware abstraction layer (HAL)
- Embedded Rust for microcontroller
- USB Type-C PD negotiation
- I2C/SPI peripheral support

---

### v4.0.0 - Blue Phoenix OS Integration
**Target**: Q4 2027
**Theme**: Native OS integration and ecosystem

**Features**:
- Native Blue Phoenix OS support
- System-level device management
- OS installer verification (read-only)
- Boot media validation toolkit
- Secure boot chain verification
- TPM/Secure Enclave integration
- Platform certification framework

**Technical**:
- OS-level USB stack integration
- Kernel module for privileged operations
- System service (daemon/systemd)
- Boot-time device enumeration
- UEFI/BIOS interaction (read-only)

---

## Future Exploration (No Timeline)

### Potential Features
- Network device discovery (IoT, USB-over-IP)
- Remote device management console
- Mobile app (iOS/Android) for USB-C devices
- Cloud-based device analytics (opt-in, privacy-first)
- Device repair workflow automation
- Integration with OEM repair portals
- Multi-language support (i18n)
- Accessibility improvements (screen readers, keyboard nav)

### Research Areas
- USB4/Thunderbolt 4 advanced enumeration
- eSIM/eUICC device detection
- Quantum-safe cryptography for audit logs
- Privacy-preserving device telemetry
- AI-powered device anomaly detection
- Blockchain-based device provenance (if viable)

---

## Non-Goals (Will Not Implement)

The following features are explicitly **out of scope** for BootForge:

- ❌ **Destructive operations**: Format, partition, write, flash
- ❌ **Security bypass**: Bootloader unlock, jailbreak, root
- ❌ **Firmware modification**: Custom ROMs, kernel patches
- ❌ **Data recovery**: File carving, partition reconstruction
- ❌ **Exploit development**: Security vulnerability research tools
- ❌ **DRM circumvention**: Copy protection bypass

BootForge remains a **read-only, diagnostic, compliance-first** platform.

---

## Contribution Opportunities

We welcome community contributions in the following areas:

1. **Device Profiles**: Add known device fingerprints to the database
2. **Platform Testing**: Verify compatibility on new OS versions
3. **Documentation**: Improve guides, tutorials, and translations
4. **Bug Reports**: File detailed bug reports with reproduction steps
5. **Feature Requests**: Suggest enhancements aligned with roadmap

See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

---

## Versioning Strategy

BootForge follows [Semantic Versioning](https://semver.org/) (SemVer):

- **Major (X.0.0)**: Breaking API changes, major architecture shifts
- **Minor (3.X.0)**: New features, backward-compatible
- **Patch (3.0.X)**: Bug fixes, security patches, documentation

Release cadence: Quarterly minor releases, monthly patches as needed.

---

## Feedback

Have ideas for the roadmap? Open an issue on GitHub:
https://github.com/Bboy9090/Bootforge-usb/issues

For security-sensitive feedback, see [SECURITY.md](../SECURITY.md).

---

## Document History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-05-23 | Initial roadmap through v4.0.0 |
