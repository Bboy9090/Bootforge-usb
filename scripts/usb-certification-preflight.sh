#!/usr/bin/env bash
# Non-destructive evidence collection. Does not mount, unmount, format, or write to devices.
set -euo pipefail
umask 077
OUT="${1:-./usb-certification-evidence}"
mkdir -p "$OUT"
case "$(uname -s)" in
  Darwin)
    uname -a > "$OUT/host.txt"
    diskutil list external physical > "$OUT/external-disks.txt"
    system_profiler SPUSBDataType -detailLevel mini > "$OUT/usb-inventory.txt"
    system_profiler SPHardwareDataType -detailLevel mini > "$OUT/host-hardware.txt"
    ;;
  Linux)
    uname -a > "$OUT/host.txt"
    lsblk -o NAME,PATH,TRAN,RM,RO,SIZE,MODEL,SERIAL,TYPE,FSTYPE,MOUNTPOINTS > "$OUT/block-inventory.txt"
    if command -v lsusb >/dev/null 2>&1; then lsusb > "$OUT/usb-inventory.txt"; else printf 'lsusb unavailable\n' > "$OUT/usb-inventory.txt"; fi
    ;;
  *) printf 'Unsupported host: read-only preflight only supports macOS/Linux\n' >&2; exit 2 ;;
esac
printf 'READ_ONLY_PREFLIGHT_COMPLETE\nNo device was modified.\n' > "$OUT/status.txt"
printf 'Evidence directory: %s\n' "$OUT"
printf 'Review device identity and removable-media status before any physical test.\n'
