# Boot It Back Capability Matrix

Status: authoritative product-expansion target
Public product name: **Boot It Back**
Parent brand: **Bobby's Workshop**
Existing detection core: `libbootforge` (read-only and unchanged)

## Architectural rule

Boot It Back is split into two trust zones.

1. **Detection Core** — existing `libbootforge`, permanently read-only. It discovers devices, fingerprints hardware, validates media, plans recovery, and emits audit evidence.
2. **Authorized Execution Engine** — separate privileged component for operations that intentionally modify user-owned media or devices. It must never be linked into the read-only library and must require explicit target confirmation, operation preview, ownership/authorization acknowledgement, backup warning, and post-operation verification.

The existing `docs/SAFE_WRITE_POLICY.md` remains binding on `libbootforge`.

## Capability levels

- **L0 Detect** — read-only identification and health checks.
- **L1 Verify** — checksum/signature/source validation and compatibility planning.
- **L2 Prepare** — create host-side plans, downloads, staging areas, and recovery packages.
- **L3 Execute** — authorized writes to removable media or user-owned devices using documented/vendor-supported mechanisms.
- **L4 Recover** — boot/partition/OS repair and restoration using documented recovery mechanisms.
- **L5 Advanced Authorized** — vendor-supported bootloader unlock/root workflows after explicit ownership and destructive-data warnings.
- **Blocked** — credential/security circumvention, activation/FRP bypass, exploit-based unlocking, DRM bypass, unauthorized persistence, or stealth access.

## PC boot and installer matrix

| Family | Detect/Verify | Create media | Install/Recover target |
|---|---|---|---|
| Windows 10/11 | L0-L1 | L3: ISO/WIM/ESD, UEFI/GPT, legacy BIOS where supported | L4: Startup Repair, BCD/EFI reconstruction, WinRE, offline SFC/DISM, restore/reset |
| Windows Server | L0-L1 | L3 | L4 using Microsoft-supported recovery |
| Linux distributions | L0-L1 | L3: hybrid ISO/IMG, persistence where distro supports it | L4: GRUB/systemd-boot repair, chroot recovery, filesystem checks |
| macOS Intel | L0-L1 | L3: Apple-supported installer/recovery media | L4: Recovery, reinstall, Disk Utility, Startup Security-compliant paths |
| macOS Apple Silicon | L0-L1 | L2-L3 where Apple supports restore/install tooling | L4: Recovery/DFU restore using Apple-supported tooling; no Activation Lock bypass |
| ChromeOS / ChromeOS Flex | L0-L1 | L3 recovery media | L4 official recovery/reinstall |
| BSD family | L0-L1 | L3 installer media | L4 documented bootloader/filesystem recovery |
| Proxmox/VMware/Hyper-V install media | L0-L1 | L3 | L4 installer/rescue workflows |
| Rescue/live environments | L0-L1 | L3 | L4 vendor/project-supported rescue operations |

## Boot media formats and firmware

Target support:
- ISO, hybrid ISO, IMG, raw disk image, WIM/ESD/SWM, VHD/VHDX where applicable, DMG only through supported conversion/Apple workflows.
- GPT/UEFI, removable-media fallback path, MBR/legacy BIOS where hardware supports it.
- FAT32, exFAT, NTFS, ext family and other filesystems only where the selected workflow legitimately requires them.
- Secure Boot-aware media validation and signed boot-chain checks.
- Multi-boot through supported technologies such as Ventoy/GRUB/systemd-boot without bypassing Secure Boot policy.
- Architecture validation for x86, x86_64, ARM64 and other supported targets.
- USB 2/3/4 and USB-C removable storage handling.
- Write verification by block/hash sampling or full image hash when technically meaningful.

## Recovery matrix

Target workflows:
- wrong/missing EFI boot entry
- damaged EFI System Partition
- missing/corrupt BCD
- GRUB/systemd-boot failures
- initramfs failures
- filesystem repair via supported tools
- partition-table backup/restore where a verified copy exists
- lost boot flag / wrong firmware boot target
- failed OS update recovery
- rollback/restore from known-good backup
- reinstall while preserving data where the platform supports it
- clean reinstall with explicit destructive confirmation
- disk imaging and image restore
- SMART/NVMe health preflight
- memory/storage preflight before blaming the OS
- boot-loop classification
- no-boot / no-POST handoff to hardware diagnostics
- encrypted-volume workflows only after the user supplies authorized recovery credentials

## Android device matrix

Allowed target workflows:
- ADB detection and authorized diagnostics.
- Fastboot/fastbootd detection.
- OEM/vendor documented bootloader unlock flows where the device owner explicitly opts in and accepts data-loss warnings.
- Stock firmware flashing using official/vendor-documented tooling and exact model/revision compatibility checks.
- Recovery/OTA sideload when supported by the OEM.
- Backup/restore where the platform permits it.
- Root workflows only when they rely on a vendor-supported/unlocked bootloader and the user explicitly chooses them.
- Device-specific compatibility database covering model, region, bootloader revision, anti-rollback status, partition layout and image provenance.

Never include:
- FRP bypass.
- OEM account/Activation Lock bypass.
- stolen-device workflows.
- exploit-based bootloader unlocking.
- EDL/authentication circumvention or leaked service credentials.
- carrier/network lock circumvention.

## Apple mobile-device matrix

Allowed target workflows:
- normal/recovery/DFU detection.
- Apple-supported update/restore.
- IPSW provenance and compatibility verification.
- backup/restore and recovery guidance.
- signed/authorized developer-device operations.

Never include:
- Activation Lock bypass.
- passcode bypass.
- SEP/security circumvention.
- exploit-based jailbreak delivery as a product capability.

## USB and removable-media creation

Execution engine target:
- deterministic target-disk identity lock before write.
- source checksum/signature verification.
- capacity and sector-size validation.
- unmount/eject coordination.
- streaming image write with progress, cancellation boundaries, and power-loss-safe reporting.
- post-write verification.
- partition-table/media layout inspection.
- optional persistence partition where supported.
- multi-ISO catalog management.
- recovery documentation copied onto media only when the user opts in.
- immutable audit receipt containing source hash, target fingerprint, operation version and verification result.

## Firmware and provenance

Every downloadable image/firmware workflow should record:
- original vendor/project URL.
- release/build/version.
- model/region/revision compatibility.
- cryptographic checksum/signature when published.
- download timestamp.
- local checksum.
- whether the source is official, community, or user-supplied.
- anti-rollback/security constraints.

Unknown or unverifiable images fail closed by default.

## "Unknown problem" handling

Boot It Back must not claim impossible omniscience. Instead it should use a layered diagnosis engine:

1. identify host + target hardware.
2. detect current mode/state.
3. classify boot stage reached.
4. inventory firmware/partition/filesystem/OS evidence available read-only.
5. run hardware-health checks.
6. compare against known failure signatures.
7. generate ranked recovery plans with confidence and risk.
8. require explicit user choice before any write.
9. verify outcome and learn only from non-sensitive diagnostic telemetry when the user opts in.

This makes novel or previously unseen failures diagnosable without inventing unsupported operations.

## Release gate

A capability may be marketed as supported only after all applicable gates pass:
- fixture/unit coverage.
- exact-model compatibility tests.
- destructive-operation confirmation tests.
- wrong-target protection tests.
- interrupted-write/power-loss tests.
- rollback/recovery-path tests.
- real hardware certification.
- provenance/signature verification.
- audit receipt verification.
- platform-specific signing/notarization/store requirements.

Until then it must be labeled Planned, Experimental, or Diagnostic-only.
