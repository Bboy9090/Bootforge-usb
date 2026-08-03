# BootForge USB R4 Enterprise SDK Plan

R4 begins only after R2 and R3 are green, merged, and software-validated.

## Scope

### Stable API and compatibility
- lock the Rust public API v1 surface
- semantic-versioning and deprecation policy
- API compatibility snapshots and compile tests
- feature-flag policy and minimal dependency profile
- sync, async, callback, and streaming contracts

### Language bindings
- C ABI and generated headers
- Python bindings
- C# bindings
- Swift bindings
- Kotlin bindings
- Node.js bindings
- language-specific examples and package publishing gates

### Security and supply chain
- unsafe Rust audit
- dependency and license audits
- SBOM generation
- reproducible builds
- signed artifacts and checksums
- provenance attestations
- release signing and verification guidance

### Performance certification
- scan/event latency benchmarks
- bounded memory targets
- dropped-event accounting
- 1/10/50/100/250-device synthetic matrices
- hub cascade and composite-device benchmarks
- seven-day soak testing

### Hardware validation
- Windows, Linux, macOS, and ARCWYRE receipts
- direct port, powered hub, bus-powered hub, dock, USB-C, and Thunderbolt matrices
- Android mode transitions
- Apple recovery and DFU
- MTP, PTP, CDC, storage, HID, and composite devices

## Release gates

R4 may be called a release candidate only when all required software gates pass and named physical hardware receipts exist. No inferred or marketing-grade validation is permitted.
