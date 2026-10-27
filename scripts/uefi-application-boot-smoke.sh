#!/usr/bin/env bash
# Runs a virtual-only x86-64 UEFI application boot. Never attaches a host disk.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FIRMWARE="${1:-}"
[[ -f "$FIRMWARE" ]] || { echo 'Usage: scripts/uefi-application-boot-smoke.sh /path/to/edk2-x86_64-code.fd' >&2; exit 2; }
for tool in qemu-system-x86_64 mkfs.fat mmd mcopy timeout; do command -v "$tool" >/dev/null || { echo "Missing $tool" >&2; exit 2; }; done
"$ROOT/firmware/uefi/build.sh"
TEMP="$(mktemp -d)"
trap 'rm -rf "$TEMP"' EXIT
truncate -s 64M "$TEMP/uefi.img"
mkfs.fat -F 32 "$TEMP/uefi.img" >/dev/null
mmd -i "$TEMP/uefi.img" ::/EFI ::/EFI/BOOT
mcopy -i "$TEMP/uefi.img" "$ROOT/firmware/uefi/out/BOOTX64.EFI" ::/EFI/BOOT/BOOTX64.EFI
set +e
timeout 15 qemu-system-x86_64 -machine q35 -m 256 -display none -monitor none -serial none -debugcon "file:$TEMP/uefi.log" -global isa-debugcon.iobase=0xe9 -net none -drive "if=pflash,format=raw,readonly=on,file=$FIRMWARE" -drive "if=ide,format=raw,file=$TEMP/uefi.img,snapshot=on" -boot order=c >/dev/null 2>"$TEMP/qemu.err"
status=$?
set -e
if grep -Fq 'INFINITEBOOT_UEFI_CERT_OK' "$TEMP/uefi.log"; then echo 'PASS: EFI application executed in QEMU and emitted certification marker'; exit 0; fi
cat "$TEMP/qemu.err" >&2
printf 'FAIL: no EFI application marker (QEMU exit %s)\n' "$status" >&2
exit 1
