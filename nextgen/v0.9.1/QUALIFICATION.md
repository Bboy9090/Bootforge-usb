# BootForge v0.9.1 Qualification Candidate

## Candidate artifact

- File: `BootForge_v0.9.1_unified_desktop_rc.zip`
- SHA-256: `2eced1f762fe2bacd6cf8350e27f2005979a7fbe710788437832c80a1d958be4`
- Size: 58,259 bytes

## Corrections folded into v0.9.1

1. Correct the file-writer test expectation from 21 bytes to 20 bytes for `bootforge-test-image`.
2. Give Chromebook manifest tests unique temporary fixture paths so parallel tests cannot delete another test's source file.
3. Isolate the Tauri desktop package with its own `[workspace]` boundary when staged under the existing repository.
4. Add valid RGBA PNG and ICO assets required by Tauri compile-time context generation.
5. Bump workspace, Tauri package, and Tauri application versions from 0.9.0 to 0.9.1.

## Native authority evidence

The equivalent correction set was exercised by GitHub Actions run `37046315745`.

All three jobs completed successfully:

- Windows: core build, core tests, core clippy, Tauri build, Tauri clippy.
- macOS: core build, core tests, core clippy, Tauri build, Tauri clippy.
- Linux: core build, core tests, core clippy, Tauri build, Tauri clippy.

The existing repository Compliance Guard also remained green during the correction wave.

## Remaining gates

This qualification does **not** claim physical-device release readiness.

Remaining release gates include:

- sacrificial removable-device tests on each supported host,
- post-write boot/media validation for each supported media family,
- production branding/icon replacement,
- platform signing/notarization/installer packaging,
- release-channel smoke testing.

The existing BootForge read-only governance boundary remains unchanged. Any destructive media-writing product remains isolated from the read-only analyzer/planner.
