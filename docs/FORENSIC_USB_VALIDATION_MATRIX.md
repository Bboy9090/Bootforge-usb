# BootForge USB Forensic Validation Matrix

This document separates code presence from proof. A feature is not called hardware-validated merely because it compiled on a hosted runner, a surprisingly durable industry superstition.

## Classification

| Level | Meaning |
|---|---|
| Implemented | A code path exists with focused tests. |
| Integrated | The path is connected to its caller and dependency chain. |
| Emulator-validated | Reproduced under a named virtual or simulated configuration. |
| Hardware-validated | Reproduced on identified physical hardware with retained evidence. |
| Release candidate | All declared release gates pass; package is not yet published. |

## Current matrix

| Capability | Implemented | Integrated | CI target | Hardware evidence required |
|---|---:|---:|---|---|
| Cross-platform enumeration | Yes | Yes | Windows, Linux, macOS | At least two controllers per OS |
| Stable identity | Yes | Yes | Deterministic unit tests | Same device across ports and reconnects |
| Reconnect correlation | Yes | Yes | Synthetic snapshot tests | Serial and no-serial devices |
| Protocol classification | Yes | Yes | Fixture tests | ADB, Fastboot, Apple Recovery, DFU, MTP/PTP, CDC |
| Stateful health | Yes | Yes | Flap and transition tests | Controlled disconnect/reconnect campaign |
| Linux driver reporting | Yes | Yes | Ubuntu | Bound and unbound devices |
| Windows driver reporting | Yes | Yes | Windows | WinUSB, composite, missing/problem device |
| macOS driver reporting | Contract only | Yes | macOS compile | IOKit implementation required |
| ARCWYRE driver reporting | Contract only | Yes | Feature compile | Native backend implementation required |
| Driver change events | Yes | Yes | Tracker tests | Driver state change campaign |
| Notification wake contract | Yes | Yes | Concurrency tests | Windows callback implementation required |
| Tamper-evident session recording | Yes | Yes | Cross-platform tests | Altered/reordered/truncated evidence files |

## Pull request gates

1. `cargo fmt --all --check`
2. `cargo check -p libbootforge --all-targets`
3. `cargo clippy -p libbootforge --all-targets -- -D warnings`
4. `cargo test -p libbootforge --all-targets`
5. `cargo check -p libbootforge --features arcwyre`
6. `RUSTDOCFLAGS='-D warnings' cargo doc -p libbootforge --no-deps`
7. Windows, Linux, and macOS matrix completion
8. No destructive USB, driver, filesystem, or firmware operations introduced

## Hardware receipt format

Every hardware validation receipt must include:

- date and operator
- host OS and exact build
- USB host controller and driver
- device VID/PID, serial handling, and mode
- hub/cable topology
- command or test binary version
- raw JSONL evidence file
- final hash-chain verification result
- pass/fail statement and observed limitations
