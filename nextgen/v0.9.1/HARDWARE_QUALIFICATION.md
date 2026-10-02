# BootForge v0.9.1 Sacrificial-Device Qualification Gate

This gate is for controlled removable-media testing only. It does not change the existing read-only BootForge governance boundary.

## Lab rules

Use only expendable removable USB media with no needed data.

Before every destructive write:

1. Disconnect unrelated removable drives.
2. Record the intended device vendor, model, serial/fingerprint, capacity, and host-visible device identity.
3. Confirm the operating-system/system disk is not selected.
4. Confirm the source hash matches the planned source.
5. Re-scan the destination immediately before execution.
6. Require the destination-specific confirmation phrase.
7. Abort if device identity, capacity, removability, or system-disk status changes.

A raw device path typed by the user is never sufficient authority.

## Required host matrix

| Host | Windows installer | Linux hybrid/raw | Compatible macOS raw image | Official Apple installer |
| --- | --- | --- | --- | --- |
| Windows | Test | Test | Test | Not advertised |
| macOS | Test | Test | Test | Test with Apple's supported workflow |
| Linux | Test | Test | Test | Not advertised |

## Test sequence for each supported host/target route

### 1. Preflight detection

Pass only if:

- the sacrificial device is detected as removable,
- system disk detection is false,
- reported capacity matches the physical device within expected platform reporting differences,
- device identity remains stable over two consecutive scans,
- source exists and source SHA-256 is verified,
- source size fits the destination.

### 2. Negative safety tests

The executor must refuse:

- a non-removable destination,
- a system disk,
- a stale or changed device identity,
- a destination smaller than the source,
- a modified source whose hash no longer matches,
- an incorrect confirmation phrase,
- a partition/slice path where a whole physical disk is required,
- an unsupported host/target route.

Any false acceptance is an immediate release blocker.

### 3. Write execution

For routes that are actually enabled for hardware testing:

- capture start time,
- capture destination identity again immediately before opening the device,
- stream progress monotonically from 0 to completion,
- sync/flush before verification,
- report cancellation distinctly from success,
- never report success after a partial write.

### 4. Post-write verification

Pass only if:

- the device is re-detected after write,
- expected partition/filesystem structure exists,
- byte or file verification required by the route succeeds,
- no unrelated disk was modified,
- application reports the same destination identity that was confirmed before writing.

### 5. Boot/media validation

Windows installer:
- target boots on a supported UEFI test machine/VM path,
- Windows Setup reaches the initial installer screen,
- split-WIM media, when required by FAT32 constraints, is accepted by Setup.

Linux hybrid/raw:
- target boots,
- expected live/install environment reaches its initial screen,
- checksum verification succeeds where the distribution supports it.

Compatible macOS raw image:
- image layout is reproduced as expected,
- target is recognized by a compatible Mac/test environment,
- no claim is made that this equals Apple's official installer workflow.

Official Apple installer on macOS:
- use Apple's supported `createinstallmedia` path,
- installer volume is created successfully,
- compatible Mac recognizes it as bootable installer media.

## Result classes

- GREEN: all applicable tests pass with no safety deviation.
- RED: any system-disk, identity, write, verification, or boot failure.
- BLOCKED: route cannot be tested because required hardware/source/OS privilege is unavailable.

Do not convert BLOCKED to GREEN.

## Evidence to retain per test

- host OS/version,
- BootForge/MediaForge candidate version,
- source file name and SHA-256,
- device vendor/model/serial or stable fingerprint,
- capacity,
- route selected,
- confirmation phrase used,
- start/end result,
- verification result,
- boot validation result,
- relevant logs.

## Release condition

Physical media writing is not release-ready until every advertised host/target route is either GREEN or explicitly removed from the advertised capability matrix.
