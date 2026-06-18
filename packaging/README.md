# BootForge Packaging Guide

This directory contains platform-specific packaging configurations and instructions for distributing BootForge.

---

## Platform Support

BootForge supports packaging for:
- **Windows**: MSIX packages
- **Blue Phoenix OS**: Native integration (planned)
- **Linux**: Manual distribution (DEB/RPM planned for future)
- **macOS**: Manual distribution (DMG planned for future)

---

## Windows MSIX Packaging

### Overview

MSIX is the modern Windows application package format, replacing MSI and AppX. BootForge can be packaged as an MSIX for distribution via Microsoft Store or enterprise deployment.

### Prerequisites

**Development Machine**:
- Windows 10 SDK (19041 or later)
- Visual Studio 2019+ with "Desktop development with C++" workload
- MSIX Packaging Tool (optional, for GUI-based packaging)
- Rust toolchain for Windows (MSVC target)

**Build Requirements**:
```bash
# Install Rust with MSVC target
rustup target add x86_64-pc-windows-msvc

# Verify Windows SDK installed
reg query "HKLM\SOFTWARE\Microsoft\Windows Kits\Installed Roots" /v KitsRoot10
```

### Build Steps

#### 1. Build Release Binary

```bash
# Build for Windows x64
cargo build --release --target x86_64-pc-windows-msvc

# Binary location: target/x86_64-pc-windows-msvc/release/bootforge-cli.exe
```

#### 2. Create Package Manifest

Create `AppxManifest.xml` in `packaging/windows/`:

```xml
<?xml version="1.0" encoding="utf-8"?>
<Package xmlns="http://schemas.microsoft.com/appx/manifest/foundation/windows10"
         xmlns:uap="http://schemas.microsoft.com/appx/manifest/uap/windows10"
         xmlns:rescap="http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities">

  <Identity Name="com.bobbysworld.bootforge"
            Publisher="CN=Bobby's World"
            Version="3.0.0.0" />

  <Properties>
    <DisplayName>BootForge</DisplayName>
    <PublisherDisplayName>Bobby's World</PublisherDisplayName>
    <Logo>Assets\Logo.png</Logo>
    <Description>Cross-platform USB device detection and enumeration tool</Description>
  </Properties>

  <Dependencies>
    <TargetDeviceFamily Name="Windows.Desktop" MinVersion="10.0.17763.0" MaxVersionTested="10.0.22000.0" />
  </Dependencies>

  <Resources>
    <Resource Language="en-us" />
  </Resources>

  <Applications>
    <Application Id="BootForge" Executable="bootforge-cli.exe" EntryPoint="Windows.FullTrustApplication">
      <uap:VisualElements DisplayName="BootForge"
                          Description="USB device detection tool"
                          BackgroundColor="transparent"
                          Square150x150Logo="Assets\Square150x150Logo.png"
                          Square44x44Logo="Assets\Square44x44Logo.png">
      </uap:VisualElements>
    </Application>
  </Applications>

  <Capabilities>
    <rescap:Capability Name="runFullTrust" />
    <DeviceCapability Name="usb" />
  </Capabilities>

</Package>
```

#### 3. Create Asset Files

Required assets in `packaging/windows/Assets/`:
- `Logo.png` (256x256)
- `Square150x150Logo.png` (150x150)
- `Square44x44Logo.png` (44x44)

#### 4. Package MSIX

```powershell
# Using MakeAppx tool (from Windows SDK)
"C:\Program Files (x86)\Windows Kits\10\bin\10.0.22000.0\x64\MakeAppx.exe" pack `
  /d packaging\windows\package `
  /p bootforge-3.0.0-x64.msix

# Sign the package (required for installation)
"C:\Program Files (x86)\Windows Kits\10\bin\10.0.22000.0\x64\SignTool.exe" sign `
  /fd SHA256 `
  /a `
  /f my_certificate.pfx `
  /p certificate_password `
  bootforge-3.0.0-x64.msix
```

### Testing MSIX Package

```powershell
# Install locally for testing
Add-AppxPackage -Path bootforge-3.0.0-x64.msix

# Run the app
Start-Process "bootforge:"

# Uninstall after testing
Remove-AppxPackage -Package com.bobbysworld.bootforge_3.0.0.0_x64__*
```

### Distribution

**Microsoft Store**:
1. Create Partner Center account
2. Submit MSIX package for certification
3. Publish to Store

**Enterprise Deployment**:
- Use Intune or SCCM for enterprise distribution
- Side-loading enabled via Group Policy

**Direct Download**:
- Host MSIX on GitHub Releases
- Users must install signing certificate first (for non-Store installs)

---

## Blue Phoenix OS Integration

### Overview

Blue Phoenix OS is the target operating system for BootForge. Native integration includes:
- System-level USB enumeration service
- OS installer verification toolkit
- Boot media validation

### Package Format

Blue Phoenix OS uses a custom package format (`.bpos` files) based on:
- Compressed tarball (`.tar.zst`)
- Metadata in JSON format
- Cryptographic signature for verification

### Build for Blue Phoenix OS

```bash
# Cross-compile for Blue Phoenix OS (Linux-based)
cargo build --release --target x86_64-unknown-linux-gnu

# Create package structure
mkdir -p bpos-package/usr/bin
mkdir -p bpos-package/usr/lib
mkdir -p bpos-package/etc/bootforge

# Copy binaries
cp target/x86_64-unknown-linux-gnu/release/bootforge-cli bpos-package/usr/bin/
cp target/x86_64-unknown-linux-gnu/release/libbootforge.so bpos-package/usr/lib/

# Create metadata
cat > bpos-package/package.json <<EOF
{
  "name": "bootforge",
  "version": "3.0.0",
  "architecture": "x86_64",
  "dependencies": ["libusb-1.0"],
  "provides": ["usb-detection"],
  "conflicts": [],
  "description": "USB device detection and enumeration tool"
}
EOF

# Package
tar --zstd -cvf bootforge-3.0.0-x86_64.bpos.tar.zst -C bpos-package .

# Sign (using Blue Phoenix OS signing key)
bpos-sign --key /path/to/signing-key.pem bootforge-3.0.0-x86_64.bpos.tar.zst
```

### Installation on Blue Phoenix OS

```bash
# Install via package manager
bpos-pkg install bootforge-3.0.0-x86_64.bpos.tar.zst

# Verify installation
bootforge-cli --version

# Enable system service (if applicable)
systemctl enable bootforge-usb-monitor.service
systemctl start bootforge-usb-monitor.service
```

---

## Linux Distribution Packaging (Future)

### DEB Package (Debian/Ubuntu)

**Future Roadmap**: v3.2.0

Requirements:
- `debhelper` tools
- `cargo-deb` for automated DEB generation

### RPM Package (Fedora/RHEL)

**Future Roadmap**: v3.2.0

Requirements:
- `rpmbuild` tools
- `cargo-rpm` for automated RPM generation

### AppImage (Universal Linux)

**Future Roadmap**: v3.3.0

Portable, single-file application bundle for all Linux distributions.

---

## macOS Distribution (Future)

### DMG Package

**Future Roadmap**: v3.2.0

Requirements:
- macOS 12+ for building
- Code signing certificate (Apple Developer)
- `create-dmg` tool

### Mac App Store

**Future Roadmap**: v4.0.0

Requirements:
- Apple Developer account ($99/year)
- App sandboxing (compatible with read-only USB access)
- Notarization for Gatekeeper

---

## Cross-Platform Build Automation

### GitHub Actions Workflow

BootForge uses GitHub Actions for automated builds across platforms. See `.github/workflows/release.yml`.

**Workflow**:
1. Checkout code
2. Install Rust toolchain
3. Build for all targets (Linux, macOS, Windows)
4. Run tests
5. Create release artifacts
6. Upload to GitHub Releases

### Local Multi-Platform Build

```bash
# Install cross-compilation tool
cargo install cross

# Build for all targets
cross build --release --target x86_64-unknown-linux-gnu
cross build --release --target x86_64-apple-darwin
cross build --release --target x86_64-pc-windows-msvc

# Artifacts in target/<triple>/release/
```

---

## Troubleshooting

### Windows: USB Access Permission Issues

**Problem**: "Access Denied" when enumerating USB devices on Windows.

**Solution**:
1. Install WinUSB driver via Zadig: https://zadig.akeo.ie/
2. Select USB device and choose "WinUSB" driver
3. Reinstall BootForge

### Linux: libusb Not Found

**Problem**: `cargo build` fails with "libusb-1.0 not found".

**Solution**:
```bash
# Debian/Ubuntu
sudo apt install libusb-1.0-0-dev pkg-config

# Fedora/RHEL
sudo yum install libusb1-devel

# Arch Linux
sudo pacman -S libusb
```

### macOS: Code Signing Required

**Problem**: macOS Gatekeeper blocks unsigned app.

**Workaround** (development only):
```bash
# Remove quarantine attribute
xattr -d com.apple.quarantine /path/to/bootforge-cli

# Allow execution
chmod +x /path/to/bootforge-cli
```

**Long-term**: Obtain Apple Developer certificate and sign binary.

---

## Versioning

Package versions follow SemVer (Semantic Versioning):
- **Major.Minor.Patch** (e.g., 3.0.0)
- MSIX requires 4-part version: **Major.Minor.Patch.Build** (e.g., 3.0.0.0)

Update version in:
- `Cargo.toml` (workspace.package.version)
- `app.metadata.json` (version field)
- `packaging/windows/AppxManifest.xml` (Identity.Version)

---

## Release Checklist

Before creating a release package:

1. ✅ Update version in all `Cargo.toml` files
2. ✅ Update `app.metadata.json` version
3. ✅ Run full test suite: `cargo test`
4. ✅ Run health check: `./scripts/healthcheck.sh`
5. ✅ Run smoke tests: `./scripts/smoke-test.sh`
6. ✅ Build release binaries for all platforms
7. ✅ Test binaries on target platforms
8. ✅ Create git tag: `git tag v3.0.0`
9. ✅ Push tag: `git push origin v3.0.0`
10. ✅ Create GitHub Release with artifacts

See `docs/RELEASE_CHECKLIST.md` for complete list.

---

## Support

For packaging issues:
- GitHub Issues: https://github.com/Bboy9090/Bootforge-usb/issues
- Documentation: `docs/`

---

**Last Updated**: 2026-05-23
**BootForge Version**: 3.0.0
