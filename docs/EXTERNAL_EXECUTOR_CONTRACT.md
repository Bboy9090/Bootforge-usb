# External Media Executor Contract

BootForge itself remains read-only.

This document defines the boundary between the BootForge analyzer/planner and a separate media-writing application.

## BootForge responsibilities

BootForge may:

- identify the host operating system,
- inspect removable devices,
- validate source media,
- calculate hashes,
- determine which target workflow applies,
- generate a dry-run plan,
- require an explicit destination-specific confirmation phrase,
- export a handoff manifest.

BootForge must not:

- open raw block devices for writing,
- format disks,
- alter partition tables,
- invoke destructive shell commands,
- elevate for disk writes,
- perform firmware or bootloader modification.

## Executor responsibilities

A separate media writer application may consume the exported handoff manifest and implement host-native media creation.

That executor is a different product boundary and must independently verify:

1. the destination still exists,
2. the destination is removable,
3. the destination is not the system disk,
4. capacity still matches the handoff,
5. the source hash still matches,
6. the user reconfirms the destructive action,
7. the selected host/target route is supported.

The executor must never trust a raw device path supplied by UI text.

## Planned executor families

### Windows host

- Windows installer media
- Linux hybrid/raw images
- compatible user-supplied macOS raw images

### macOS host

- Windows installer media
- Linux hybrid/raw images
- Apple's supported macOS installer workflow
- compatible raw images

### Linux host

- Windows installer media
- Linux hybrid/raw images
- compatible user-supplied macOS raw images

## macOS official installer rule

Only the macOS-host path may advertise Apple's official installer workflow. Windows and Linux may validate or stage macOS source material, but must not claim equivalence to Apple's `createinstallmedia` workflow.

## Handoff integrity

The final executor contract should include:

- schema version,
- source path,
- source SHA-256,
- target media type,
- host platform,
- expected destination identity,
- expected capacity,
- removable/system-disk observations,
- planner timestamp,
- required confirmation phrase,
- planner version.

The separate executor must re-check all safety-critical fields at execution time rather than trusting stale planner observations.
