#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p out
clang -target x86_64-windows-msvc -ffreestanding -fno-stack-protector -fno-builtin -fshort-wchar -c boot.c -o out/boot.obj
lld-link /subsystem:efi_application /entry:efi_main /nodefaultlib /machine:x64 /out:out/BOOTX64.EFI out/boot.obj
file out/BOOTX64.EFI
shasum -a 256 out/BOOTX64.EFI
