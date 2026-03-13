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
