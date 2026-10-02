# BootForge Store Testing Metadata Draft

## macOS TestFlight

**App name:** BootForge

**Beta description**

BootForge is a cross-platform media-planning and removable-device analysis utility. This TestFlight build is a sandboxed Store Mode preview for validating the desktop interface, source-image analysis, host/target compatibility logic, and safety messaging.

**What to test**

- Launch and window behavior.
- Source-image selection and analysis.
- SHA-256 calculation and source metadata.
- Windows/Linux/macOS/Chromebook target presentation.
- Chromebook board/architecture compatibility messaging.
- Clear indication that Store Mode is read-only.
- No destructive USB-writing control should successfully execute in this build.

**Important TestFlight note**

The TestFlight/macOS App Store build intentionally does not perform raw removable-disk writes because it runs under the macOS App Sandbox. Hardware writer testing is performed with the separately signed/notarized BootForge hardware-test build.

## Microsoft Store Private Audience

**App name:** BootForge

**Short description**

Create and validate operating-system and recovery media with device revalidation, source hashing, destination safeguards, and post-write verification.

**Private beta focus**

- Removable-device discovery.
- Source ISO/IMG analysis.
- Windows installation-media preparation.
- Linux live/raw image writing.
- Chromebook recovery and compatible Linux media workflows.
- Blue Phoenix prepared-image workflows.
- System-disk and non-removable-device refusal.
- Destination-specific erase confirmation.
- Post-write verification.

**Certification/testing notes**

BootForge performs destructive media operations only after fresh removable-device detection, source hashing, capacity validation, system-disk exclusion, and an exact device-bound confirmation phrase. Testers should use expendable removable media only.

## Listing assets still required

- Production BootForge icon.
- At least one store screenshot.
- Privacy policy URL if required by the selected Store declarations.
- Support/contact URL or email.
- Final category and age-rating answers.
- Final Apple Bundle ID / Microsoft Partner Center product identity.
