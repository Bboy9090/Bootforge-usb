# BootForge v0.9 native-CI staging

This directory stages the exact BootForge v0.9 Unified Desktop RC for native authority testing.

- RC SHA-256: `d457ba1d23fcb97a3bd993b446e198c565224714171f51d82031b5d65510820f`
- Source archive: `BootForge_v0.9_unified_desktop_rc.zip`
- Archive transport: eight numerically ordered base64 chunks in this directory.
- Staging branch: `feature/unified-desktop-v0.9`

## Boundary

This staging area does **not** replace the repository's existing read-only BootForge workspace or its immutable read-only policy. The v0.9 writer implementation is an isolated release-candidate payload used to prove native compilation and tests before any governance or merge decision.

## Authority gates

The native CI workflow must:

1. reconstruct the exact archive,
2. verify the SHA-256 above before extraction,
3. run rustfmt checks,
4. build and test the Rust workspace,
5. run clippy with warnings denied,
6. compile and clippy the Tauri desktop shell on Windows, macOS, and Linux.

Physical-device writes are not exercised on GitHub-hosted runners. Sacrificial-device validation remains a separate hardware gate.
