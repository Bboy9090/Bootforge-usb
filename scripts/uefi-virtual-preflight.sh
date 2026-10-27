#!/usr/bin/env bash
# Virtual-only UEFI firmware smoke test. Never passes through physical disks.
set -euo pipefail
command -v qemu-system-x86_64 >/dev/null || { echo 'QEMU unavailable' >&2; exit 2; }
FIRMWARE="${1:-}"
if [[ -z "$FIRMWARE" || ! -f "$FIRMWARE" ]]; then
  echo 'Usage: uefi-virtual-preflight.sh /path/to/edk2-x86_64-code.fd' >&2
  exit 2
fi
printf 'Firmware SHA-256: '
shasum -a 256 "$FIRMWARE"
printf 'QEMU: '
qemu-system-x86_64 --version | head -1
printf 'Starting virtual-only UEFI boot, no attached disks. Timeout is expected.\n'
if command -v timeout >/dev/null 2>&1; then
  timeout 12 qemu-system-x86_64 -machine q35 -m 256 -nographic -monitor none -serial stdio -net none -drive "if=pflash,format=raw,readonly=on,file=$FIRMWARE" || status=$?
  if [[ "${status:-0}" -eq 124 ]]; then echo 'UEFI_TIMEOUT_EXPECTED (not boot certification)'; exit 0; fi
  exit "${status:-0}"
else
  echo 'No timeout utility: refusing unbounded emulator launch' >&2
  exit 2
fi
