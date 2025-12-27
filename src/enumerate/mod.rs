use crate::types::UsbDeviceInfo;
use anyhow::Result;
use log::info;

pub mod common;
mod libusb;
mod linux;
mod macos;
mod windows;

pub use common::FallbackEnumerator;
pub use libusb::enumerate_libusb;
pub use linux::enrich_linux;
pub use macos::enrich_macos;
pub use windows::enrich_windows;

/// Enumerate all USB devices on the system
///
/// This is the main entry point for USB device enumeration. It executes
/// the complete detection pipeline with four stages:
///
/// **Stage 1: Transport Scanning**
/// - Uses libusb (rusb) for cross-platform base enumeration
/// - Probes candidate devices with VID/PID/bus/address
/// - Attempts to read string descriptors (gracefully handles permission errors)
///
/// **Stage 2: Descriptor Reading** (integrated into Stage 1)
/// - Reads manufacturer, product, and serial strings when accessible
/// - Creates candidate device records
///
/// **Stage 3: Platform Enrichment**
/// - Windows: Queries SetupAPI for device paths and driver status
/// - macOS: Reads IOKit registry for paths and location IDs
/// - Linux: Maps sysfs paths, driver bindings, and authorization status
///
/// **Stage 4: Protocol Classification** (performed by caller if needed)
/// - Use `classify_device_protocols()` to detect ADB, Fastboot, Apple, MTP
///
/// The result is a list of confirmed devices ready for application use.
///
/// # Example
///
/// ```no_run
/// use bootforge_usb::enumerate_all;
///
/// match enumerate_all() {
///     Ok(devices) => {
///         for device in devices {
///             println!("Found: {}", device);
///         }
///     }
///     Err(e) => eprintln!("Error enumerating devices: {}", e),
/// }
/// ```
pub fn enumerate_all() -> Result<Vec<UsbDeviceInfo>> {
    info!("Starting USB device detection pipeline");

    // Stage 1: Transport Scanning - probe candidate devices
    let mut devices = enumerate_libusb()?;

    info!("Stage 1 complete: {} candidate devices discovered", devices.len());

    // Stage 3: Platform Enrichment - augment with OS-specific data
    #[cfg(target_os = "windows")]
    {
        info!("Stage 3: Applying Windows platform enrichment");
        enrich_windows(&mut devices)?;
    }

    #[cfg(target_os = "macos")]
    {
        info!("Stage 3: Applying macOS platform enrichment");
        enrich_macos(&mut devices)?;
    }

    #[cfg(target_os = "linux")]
    {
        info!("Stage 3: Applying Linux platform enrichment");
        enrich_linux(&mut devices)?;
    }

    info!("Detection pipeline complete: {} confirmed devices", devices.len());
    Ok(devices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enumerate_all() {
        // Test that enumerate_all doesn't panic
        // Actual device enumeration may fail due to permissions or lack of devices
        let result = enumerate_all();
        match result {
            Ok(devices) => {
                println!("Successfully enumerated {} devices", devices.len());
            }
            Err(e) => {
                println!(
                    "Enumeration failed (may be expected in test environment): {}",
                    e
                );
            }
        }
    }
}
