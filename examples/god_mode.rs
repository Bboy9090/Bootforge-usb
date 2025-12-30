//! Example: God Mode - Full USB Descriptor Enumeration
//!
//! This example demonstrates the full power of BootForge USB's God Mode
//! features including:
//! - Complete descriptor enumeration (interfaces, endpoints)
//! - USB 3.0+ SuperSpeed detection
//! - BOS capability parsing
//! - Power delivery status
//! - Alternate mode detection
//!
//! Run with: cargo run --example god_mode

use bootforge_usb::{
    enumerate_all,
    descriptors::{
        parse_device_descriptors,
        TransferType, DeviceCapability,
    },
};
use rusb::UsbContext;

fn main() -> anyhow::Result<()> {
    // Initialize logger
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║          BootForge USB - GOD MODE Enumeration                ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // First, do basic enumeration
    let devices = enumerate_all()?;
    println!("Found {} USB device(s)\n", devices.len());

    // Now use God Mode to get full descriptor information
    let context = rusb::Context::new()?;
    let device_list = context.devices()?;

    for (idx, device) in device_list.iter().enumerate() {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Device {}: Bus {:03} Address {:03}", idx + 1, device.bus_number(), device.address());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        match parse_device_descriptors(&device) {
            Ok(desc) => {
                // Basic info
                println!("\n📋 DEVICE INFORMATION:");
                println!("   VID:PID:        {:04X}:{:04X}", desc.vendor_id, desc.product_id);
                if let Some(ref mfg) = desc.manufacturer {
                    println!("   Manufacturer:   {}", mfg);
                }
                if let Some(ref prod) = desc.product {
                    println!("   Product:        {}", prod);
                }
                if let Some(ref serial) = desc.serial_number {
                    println!("   Serial:         {}", serial);
                }
                println!("   USB Version:    {:X}.{:X}", (desc.usb_version >> 8) & 0xF, (desc.usb_version >> 4) & 0xF);
                println!("   Device Class:   {}", desc.device_class.name());
                println!("   Speed:          {}", desc.speed.name());
                println!("   Bandwidth:      {} Mbps", desc.speed.bandwidth_mbps());

                // Configurations
                println!("\n📦 CONFIGURATIONS ({}):", desc.configurations.len());
                for config in &desc.configurations {
                    println!("\n   Configuration {}:", config.number);
                    if let Some(ref desc_str) = config.description {
                        println!("      Description: {}", desc_str);
                    }
                    println!("      Max Power:   {} mA", config.max_power_ma);
                    println!("      Self-powered: {}", config.attributes.self_powered);
                    println!("      Remote Wake:  {}", config.attributes.remote_wakeup);
                    println!("      Interfaces:   {}", config.interfaces.len());

                    // Interfaces
                    for iface in &config.interfaces {
                        println!("\n      🔌 Interface {}.{}:", iface.number, iface.alternate_setting);
                        println!("         Class:     {}", iface.class.name());
                        println!("         Subclass:  {:02X}", iface.subclass);
                        println!("         Protocol:  {:02X}", iface.protocol);
                        
                        if let Some(ref desc_str) = iface.description {
                            println!("         Desc:      {}", desc_str);
                        }

                        // Class-specific info
                        if let Some(ref class_info) = iface.class_specific {
                            match class_info {
                                bootforge_usb::descriptors::ClassSpecificInfo::Hid(hid) => {
                                    println!("         HID Info:");
                                    println!("            Subclass: {:?}", hid.subclass);
                                    println!("            Protocol: {:?}", hid.protocol);
                                    println!("            Report Len: {} bytes", hid.report_descriptor_length);
                                }
                                bootforge_usb::descriptors::ClassSpecificInfo::MassStorage(ms) => {
                                    println!("         Mass Storage:");
                                    println!("            Subclass: {:?}", ms.subclass);
                                    println!("            Protocol: {:?}", ms.protocol);
                                }
                                bootforge_usb::descriptors::ClassSpecificInfo::Audio(audio) => {
                                    println!("         Audio Info:");
                                    println!("            Subclass: {:?}", audio.subclass);
                                }
                                bootforge_usb::descriptors::ClassSpecificInfo::Video(video) => {
                                    println!("         Video Info:");
                                    println!("            Subclass: {:?}", video.subclass);
                                }
                                bootforge_usb::descriptors::ClassSpecificInfo::Cdc(cdc) => {
                                    println!("         CDC Info:");
                                    println!("            Subclass: {:?}", cdc.subclass);
                                    println!("            Protocol: {:?}", cdc.protocol);
                                }
                                _ => {}
                            }
                        }

                        // Endpoints
                        if !iface.endpoints.is_empty() {
                            println!("         Endpoints:");
                            for ep in &iface.endpoints {
                                let dir = match ep.direction {
                                    bootforge_usb::descriptors::EndpointDirection::In => "IN",
                                    bootforge_usb::descriptors::EndpointDirection::Out => "OUT",
                                };
                                let transfer = match ep.transfer_type {
                                    TransferType::Control => "Control",
                                    TransferType::Isochronous => "Isochronous",
                                    TransferType::Bulk => "Bulk",
                                    TransferType::Interrupt => "Interrupt",
                                };
                                println!("            EP{} {}: {} (max {} bytes)",
                                    ep.number, dir, transfer, ep.max_packet_size);
                                
                                if ep.interval_ms > 0.0 {
                                    println!("               Interval: {:.2} ms", ep.interval_ms);
                                }
                                
                                // SuperSpeed companion info
                                if let Some(ref ss) = ep.ss_companion {
                                    println!("               SS Companion: MaxBurst={}, BytesPerInterval={}",
                                        ss.max_burst, ss.bytes_per_interval);
                                }
                            }
                        }
                    }
                }

                // BOS Descriptor
                if let Some(ref bos) = desc.bos {
                    println!("\n🔋 BOS CAPABILITIES ({}):", bos.num_capabilities);
                    for cap in &bos.capabilities {
                        match cap {
                            DeviceCapability::Usb20Extension(ext) => {
                                println!("   USB 2.0 Extension:");
                                println!("      LPM Supported:  {}", ext.lpm_supported);
                                println!("      BESL Supported: {}", ext.besl_supported);
                            }
                            DeviceCapability::SuperSpeed(ss) => {
                                println!("   SuperSpeed Capability:");
                                println!("      LPM Supported: {}", ss.lpm_supported);
                                println!("      U1 Exit Lat:   {} µs", ss.u1_dev_exit_lat);
                                println!("      U2 Exit Lat:   {} µs", ss.u2_dev_exit_lat);
                            }
                            DeviceCapability::SuperSpeedPlus(ssp) => {
                                println!("   SuperSpeedPlus Capability:");
                                for attr in ssp.parse_sublink_speeds() {
                                    println!("      Sublink: {}", attr.speed_string());
                                }
                            }
                            DeviceCapability::ContainerId(cid) => {
                                println!("   Container ID: {}", cid.container_id_string);
                            }
                            DeviceCapability::Platform(plat) => {
                                println!("   Platform Capability: {}", plat.uuid_string);
                                if plat.webusb.is_some() {
                                    println!("      WebUSB: Supported");
                                }
                                if plat.microsoft_os_20.is_some() {
                                    println!("      MS OS 2.0: Supported");
                                }
                            }
                            DeviceCapability::Billboard(bb) => {
                                println!("   Billboard Capability:");
                                println!("      Alternate Modes: {}", bb.num_alternate_modes);
                                println!("      VCONN Power: {:?}", bb.vconn_power);
                            }
                            _ => {
                                println!("   Other Capability");
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("   ⚠️  Could not read full descriptors: {}", e);
                println!("      (May require elevated permissions)");
            }
        }
        println!();
    }

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                    GOD MODE COMPLETE                         ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    Ok(())
}
