//! Example: Watch for USB device hotplug events
//!
//! This example demonstrates how to monitor for USB device connection
//! and disconnection events in real-time.
//!
//! Run with: cargo run --example watch_devices

use bootforge_usb::{DeviceEvent, DeviceWatcher, PlatformWatcher};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    println!("Starting USB device watcher...");
    println!("Connect or disconnect USB devices to see events.");
    println!("Press Ctrl+C to exit.\n");

    // Create and start the watcher
    let mut watcher = PlatformWatcher::default();
    let receiver = watcher.start()?;

    // Monitor for events
    loop {
        match receiver.recv_timeout(Duration::from_secs(1)) {
            Ok(event) => {
                match event {
                    DeviceEvent::Added(device) => {
                        println!("📱 Device ADDED:");
                        println!("   ID: {}", device.id.as_hex_string());
                        if let Some(ref product) = device.descriptor.product {
                            println!("   Product: {}", product);
                        }
                        if let Some(ref manufacturer) = device.descriptor.manufacturer {
                            println!("   Manufacturer: {}", manufacturer);
                        }
                        println!();
                    }
                    DeviceEvent::Removed(device) => {
                        println!("🔌 Device REMOVED:");
                        println!("   ID: {}", device.id.as_hex_string());
                        if let Some(ref product) = device.descriptor.product {
                            println!("   Product: {}", product);
                        }
                        println!();
                    }
                    DeviceEvent::Changed(device) => {
                        println!("🔄 Device CHANGED:");
                        println!("   ID: {}", device.id.as_hex_string());
                        if let Some(ref product) = device.descriptor.product {
                            println!("   Product: {}", product);
                        }
                        println!();
                    }
                    DeviceEvent::Reconnected { device, previous_location } => {
                        println!("🔁 Device RECONNECTED:");
                        println!("   ID: {}", device.id.as_hex_string());
                        if let Some(ref product) = device.descriptor.product {
                            println!("   Product: {}", product);
                        }
                        if let Some(prev) = previous_location {
                            if let Some(ref path) = prev.port_path {
                                println!("   Previous port: {}", path);
                            }
                        }
                        println!();
                    }
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // No event received, continue waiting
                continue;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                println!("Watcher stopped");
                break;
            }
        }
    }

    watcher.stop()?;
    Ok(())
}
