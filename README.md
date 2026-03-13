# libbootforge

libbootforge is a low-level USB device detection library designed for hardware discovery, repair workflows, and device preparation.

The library provides structured access to USB device information including:

- vendor and product identifiers
- device descriptors
- device mode detection
- device connection events

libbootforge serves as the USB hardware discovery layer for the Bobby's Workshop device ecosystem.
# BootForge USB — REFORGE OS Platform

## Overview

**BootForge USB** is the cross-platform USB enumeration and device detection layer that powers the REFORGE OS platform (formerly Bobby's Workshop 3.0). Written in Rust, it provides diagnostic and read-only device analysis capabilities across Windows, macOS, and Linux.

### Core Libraries
- **libbootforge**: Low-level USB device detection library
- **PhoenixCore**: Device engine (planned)
- **BootForge CLI**: Command line tool (planned)
- **Bobby's Workshop**: Platform UI

### Architecture

```
Bobby's Workshop Platform
        │
        ▼
PhoenixCore
        │
        ▼
libbootforge
        │
        ├── USB detection
        ├── descriptor reading
        ├── device mode detection
        └── device event monitoring
```

## ForgeWorks Platform

### Layers
- **Workshop (Public)**: Brand trust, education, and customer transparency. (`apps/workshop-ui`)
- **ForgeWorks (Core)**: Decision engine, audit logging, and authority routing. (`services/*`, `apps/forgeworks-core`)
- **Pandora Codex (Internal)**: Historical research and risk models. (`internal/pandora-codex`)

### Services
- `device-analysis`: Capability ceiling and modification classification.
- `ownership-verification`: Confidence-based attestation engine.
- `legal-classification`: Jurisdiction-aware status labeling.
- `audit-logging`: Immutable, hash-chained activity trail.
- `authority-routing`: OEM, carrier, and court-system pathways.

### Manufacturing
- `ForgeCore`: USB diagnostic bridge (EVT/DVT/PVT).
- `Smart Thermal Platform`: Digitally controlled repair surfaces.
- `Precision Tool Matrix`: Pro-grade calibrated toolsets.
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

BootForge USB serves as the foundational layer for device enumeration, providing:
- USB device detection and descriptor reading
- Vendor ID/Product ID identification
- Protocol classification (ADB, Fastboot, MTP, Apple)
- Platform-specific device path resolution
- **Diagnostic and read-only operations only** — no device modification

REFORGE OS is a compliance-first, ownership-respecting platform for device analysis, classification, and routing. Built on a modular Rust architecture, it separates public certification and education from core diagnostic logic and internal research models.
# libbootforge

libbootforge is a low-level USB device detection library designed for hardware discovery, repair workflows, and device preparation.

The library provides structured access to USB device information including:

- vendor and product identifiers
- device descriptors
- device mode detection
- device connection events

libbootforge serves as the USB hardware discovery layer for the Bobby's Workshop device ecosystem.

---

# Purpose

Modern device repair and system recovery workflows require reliable identification of connected hardware.

BootForge focuses on:

* detecting connected USB devices
* reading hardware descriptors
* identifying device modes
* exposing device information to higher-level tools

This allows repair systems to correctly identify devices before performing operations such as flashing, diagnostics, or recovery.

---

# Ecosystem Role

BootForge acts as the **USB discovery layer** within the Bobby's Workshop device platform.

```text
Bobby's Workshop Platform
        │
        ▼
PhoenixCore (device engine)
        │
        ▼
libbootforge (USB detection layer)
        │
        ├── USB detection
        ├── descriptor reading
        ├── device mode detection
        └── device event monitoring
```

libbootforge ensures that higher-level systems can safely and accurately identify connected hardware.

---

# Core Capabilities

### USB Device Detection

Scan connected USB buses and identify active devices.

---

### Hardware Descriptor Reading

Extract detailed device information such as:

* vendor ID
* product ID
* manufacturer
* serial number
* device class

---

### Device Mode Identification

Identify special device states such as:

* recovery mode
* DFU mode
* bootloader mode
* standard device mode

---

### Platform Integration

libbootforge can provide device information to:

* repair platforms
* flashing utilities
* diagnostic environments
* automation tools

---

# Design Principles

libbootforge follows three guiding principles.

**Low-Level First**

The system focuses on accurate hardware detection before higher-level workflows.

**Platform Neutral**

The tool is designed to support multiple operating systems and hardware environments.

**Reliable Discovery**

Device information must be consistent and trustworthy before repair actions are performed.

---

# Potential Technology Stack

libbootforge may incorporate technologies such as:

* Rust
* Python
* libusb
* system-level device APIs

These technologies allow direct interaction with USB hardware layers.

---

# Use Cases

libbootforge can support workflows including:

* device repair platforms
* flashing preparation
* hardware diagnostics
* recovery mode detection
* lab device management

---

# Stack

```text
PhoenixCore      → device engine
libbootforge     → USB detection library
BootForge CLI    → user tool
Bobby's Workshop  → platform interface
```

---

# Project Status

Prototype / early development.

Future improvements may include:

* device monitoring
* event-based device detection
* automatic device classification
* integration with repair platforms

---

# License

MIT License
