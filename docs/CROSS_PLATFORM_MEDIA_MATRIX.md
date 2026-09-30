# Cross-Platform Boot Media Matrix

BootForge is moving to one shared engine with three host applications:

- BootForge for macOS
- BootForge for Windows
- BootForge for Linux

Each host consumes the same capability contract from `libbootforge::media`.

## Host / Target Matrix

| Host app | Windows installer USB | Linux live USB | Official macOS installer | Compatible raw macOS image |
|---|---|---|---|---|
| macOS | supported via native backend | supported via raw-image backend | supported via Apple's native installer workflow | supported via raw-image backend |
| Windows | supported via native backend | supported via raw-image backend | preparation/validation only | supported for compatible user-supplied raw images |
| Linux | supported via native backend | supported via raw-image backend | preparation/validation only | supported for compatible user-supplied raw images |

## Important macOS distinction

An official Apple installer USB is not the same thing as restoring an arbitrary raw image.

The official installer path depends on macOS and Apple's installer application / `createinstallmedia` workflow. BootForge must not claim that a Windows or Linux host can reproduce that exact workflow.

Windows and Linux variants can still:

- validate macOS installer/image inputs,
- inspect hashes and format metadata,
- prepare staging material,
- restore a compatible user-supplied raw disk image once the destructive-write backend is implemented.

## Architecture

```text
                 libbootforge
           shared target/media planner
                       |
        +--------------+--------------+
        |              |              |
     macOS host     Windows host    Linux host
        |              |              |
  macOS adapters  Windows adapters Linux adapters
        |              |              |
   diskutil/asr    Win32 storage   udisks/blkid
   createinstall  VDS/Storage API  mount/fs tools
```

## Backend boundaries

The shared core owns:

- media target classification,
- source validation,
- capability reporting,
- image hashing,
- write-plan generation,
- common progress/event schemas,
- common safety policy.

Host adapters own:

- removable-drive discovery,
- unmount/eject,
- partition-table creation,
- filesystem formatting,
- elevated authorization,
- platform-native raw writes,
- platform-native installer tooling,
- installer/package distribution.

## Delivery order

1. Keep `libbootforge` read-only and add the non-destructive planner first.
2. Gate that planner in CI on Windows, macOS, and Linux.
3. Add write backends behind a separate crate/interface so read-only detection cannot accidentally become destructive.
4. Wire all three desktop variants to the same planner.
5. Add integration tests with loopback/virtual disks before physical-device release gates.
6. Add packaging:
   - Windows: MSI/MSIX/direct installer
   - macOS: direct-download privileged edition + sandbox-safe App Store edition
   - Linux: AppImage + DEB/RPM

## Safety rule

No backend should accept a raw device path from UI text alone. A write target must originate from verified removable-device discovery and pass an explicit destructive-action confirmation contract.
