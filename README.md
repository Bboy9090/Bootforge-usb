# libbootforge

libbootforge is a low-level USB device detection library designed for hardware discovery, repair workflows, and device preparation.

The library provides structured access to USB device information including:

- vendor and product identifiers
- device descriptors
- device mode detection
- device connection events

libbootforge serves as the USB hardware discovery layer for the Bobby's Workshop device ecosystem.

## Platform Structure

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
