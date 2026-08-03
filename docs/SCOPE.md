# BootForge USB Scope

BootForge USB is a low-level, read-only-first forensic USB detection and device-enumeration library.

## Supported platforms

- Windows
- Linux
- macOS
- ARCWYRE

## In-scope capabilities

- USB device enumeration
- Device watchers
- Disconnect and reconnect tracking
- Stable device identity correlation where the operating system exposes enough information
- ADB foundation
- Fastboot foundation
- Apple mobile-device foundation
- MTP foundation
- DFU foundation
- CDC foundation
- PTP foundation
- Driver visibility and driver-state reporting
- USB topology, descriptor, interface, endpoint, and class reporting where supported
- USB health and reliability reporting based on observable operating-system and transport signals
- Structured forensic event records suitable for diagnostics, recovery tooling, and higher-level authorized applications

## Safety boundary

The library defaults to non-destructive, read-only inspection. It must not silently format, flash, erase, repartition, modify firmware, bypass device security, unlock devices, or alter user data.

Any future write-capable operation must live outside the core detection library, require explicit authorization, and use a separately reviewed safety contract.

## Explicitly out of scope

- Bootable USB creation
- ISO writing or modification
- Operating-system installation
- Disk formatting or partitioning
- EFI or bootloader installation
- Driver injection
- OpenCore patching
- Device unlocking or security bypass
- Cloud imaging
- Plugin marketplace
- General desktop creator-suite features

## Architectural rule

The core library must remain embeddable and independent of any desktop GUI. User interfaces, installers, and higher-level recovery products may consume the library, but they must not be implemented inside the core crate or package.
