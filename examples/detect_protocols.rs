//! Example: Detect device protocols
//!
//! This example demonstrates how to detect which protocols
//! USB devices support (ADB, Fastboot, Apple, MTP, etc.)
//!
//! Run with: cargo run --example detect_protocols

use bootforge_usb::api::UsbEnumerator;
use bootforge_usb::classify_device_protocols;
use bootforge_usb::enumerate::FallbackEnumerator;
use bootforge_usb::DeviceProtocol;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    println!("Detecting USB device protocols...\n");

    // Create enumerator and get devices
    let enumerator = FallbackEnumerator::default();
    let devices = enumerator.enumerate()?;

    println!("Found {} USB device(s):\n", devices.len());

    // Check protocols for each device
    for (i, device) in devices.iter().enumerate() {
        println!("Device {}:", i + 1);
        println!("  ID: {}", device.id.as_hex_string());
        
        if let Some(ref manufacturer) = device.descriptor.manufacturer {
            println!("  Manufacturer: {}", manufacturer);
        }
        
        if let Some(ref product) = device.descriptor.product {
            println!("  Product: {}", product);
        }

        // Classify protocols
        let protocols = classify_device_protocols(device);
        
        print!("  Protocols: ");
        for (j, protocol) in protocols.iter().enumerate() {
            if j > 0 {
                print!(", ");
            }
            match protocol {
                DeviceProtocol::Adb => print!("🤖 ADB"),
                DeviceProtocol::Fastboot => print!("⚡ Fastboot"),
                DeviceProtocol::AppleDevice => print!("🍎 Apple Device"),
                DeviceProtocol::Mtp => print!("📁 MTP"),
                DeviceProtocol::Unknown => print!("❓ Unknown"),
            }
        }
        println!("\n");
    }

    // Summary
    let mut adb_count = 0;
    let mut fastboot_count = 0;
    let mut apple_count = 0;
    let mut mtp_count = 0;

    for device in &devices {
        let protocols = classify_device_protocols(device);
        for protocol in protocols {
            match protocol {
                DeviceProtocol::Adb => adb_count += 1,
                DeviceProtocol::Fastboot => fastboot_count += 1,
                DeviceProtocol::AppleDevice => apple_count += 1,
                DeviceProtocol::Mtp => mtp_count += 1,
                DeviceProtocol::Unknown => {}
            }
        }
    }

    println!("Summary:");
    println!("  ADB devices: {}", adb_count);
    println!("  Fastboot devices: {}", fastboot_count);
    println!("  Apple devices: {}", apple_count);
    println!("  MTP devices: {}", mtp_count);

    Ok(())
}
