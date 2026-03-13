# BootForge USB — REFORGE OS Platform

## Overview

**BootForge USB** is the cross-platform USB enumeration and device detection layer that powers the REFORGE OS platform (formerly Bobby's Workshop 3.0). Written in Rust, it provides diagnostic and read-only device analysis capabilities across Windows, macOS, and Linux.

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

## Platform Structure

### 1. Layers
- **Workshop (Public)**: Brand trust, education, and customer transparency. (`apps/workshop-ui`)
- **ForgeWorks (Core)**: Decision engine, audit logging, and authority routing. (`services/*`, `apps/forgeworks-core`)
- **Pandora Codex (Internal)**: Historical research and risk models. (`internal/pandora-codex`)

### 2. Services
- `device-analysis`: Capability ceiling and modification classification.
- `ownership-verification`: Confidence-based attestation engine.
- `legal-classification`: Jurisdiction-aware status labeling.
- `audit-logging`: Immutable, hash-chained activity trail.
- `authority-routing`: OEM, carrier, and court-system pathways.

### 3. Manufacturing
- `ForgeCore`: USB diagnostic bridge (EVT/DVT/PVT).
- `Smart Thermal Platform`: Digitally controlled repair surfaces.
- `Precision Tool Matrix`: Pro-grade calibrated toolsets.

## Getting Started

### Prerequisites
- Rust (Edition 2021)
- Node.js (for Tauri apps)
- Postgres (Production DB)

### Installation
```bash
cargo build --workspace
```

## Governance & Compliance
- **Language Guardrails**: Enforced via CI to ensure regulator-neutral terminology.
- **Auditability**: All actions are logged to an append-only hash chain.
- **No Execution**: The platform analyzes and routes; it does not perform bypasses or modifications.

## Documentation
- [Platform Overview](docs/public/platform-overview.md)
- [Legal Taxonomy](docs/public/legal-taxonomy.md)
- [Handoff Checklist](HANDOFF_CHECKLIST.md)

---
*Platform, Not Product.*
