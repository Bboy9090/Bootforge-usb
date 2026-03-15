//! bootforge-cli: Command-line interface for USB device detection

use libbootforge::{scan_devices, DeviceInfo, DeviceMode, DevicePlatform};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::thread;
use std::time::Duration;

struct CliOptions {
    json_output: bool,
    json_file: Option<String>,
    report_file: Option<String>,
    session_log: Option<String>,
    watch: bool,
    apple_only: bool,
    android_only: bool,
    vendor_filter: Option<u16>,
    mode_filter: Option<DeviceMode>,
}

impl CliOptions {
    fn parse() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut options = CliOptions {
            json_output: false,
            json_file: None,
            report_file: None,
            session_log: None,
            watch: false,
            apple_only: false,
            android_only: false,
            vendor_filter: None,
            mode_filter: None,
        };

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--json" => options.json_output = true,
                "--json-file" => {
                    i += 1;
                    if i < args.len() {
                        options.json_file = Some(args[i].clone());
                    } else {
                        eprintln!("Error: --json-file requires a path argument");
                        std::process::exit(1);
                    }
                }
                "--report-file" => {
                    i += 1;
                    if i < args.len() {
                        options.report_file = Some(args[i].clone());
                    } else {
                        eprintln!("Error: --report-file requires a path argument");
                        std::process::exit(1);
                    }
                }
                "--session-log" => {
                    i += 1;
                    if i < args.len() {
                        options.session_log = Some(args[i].clone());
                    } else {
                        eprintln!("Error: --session-log requires a path argument");
                        std::process::exit(1);
                    }
                }
                "--watch" => options.watch = true,
                "--apple" => options.apple_only = true,
                "--android" => options.android_only = true,
                "--mode" => {
                    i += 1;
                    if i < args.len() {
                        if let Some(mode) = DeviceMode::from_str(&args[i]) {
                            options.mode_filter = Some(mode);
                        } else {
                            eprintln!("Error: Invalid mode '{}'. Valid modes: normal, recovery, dfu, bootloader, fastboot, adb, massstorage", args[i]);
                            std::process::exit(1);
                        }
                    } else {
                        eprintln!("Error: --mode requires a mode argument");
                        std::process::exit(1);
                    }
                }
                "--vendor" => {
                    i += 1;
                    if i < args.len() {
                        if let Ok(vendor_id) = u16::from_str_radix(&args[i], 16) {
                            options.vendor_filter = Some(vendor_id);
                        } else {
                            eprintln!("Error: Invalid vendor ID '{}'", args[i]);
                            std::process::exit(1);
                        }
                    } else {
                        eprintln!("Error: --vendor requires a vendor ID argument");
                        std::process::exit(1);
                    }
                }
                "--help" => {
                    print_help();
                    std::process::exit(0);
                }
                _ => {
                    eprintln!("Error: Unknown option '{}'", args[i]);
                    print_help();
                    std::process::exit(1);
                }
            }
            i += 1;
        }

        options
    }
}

fn print_help() {
    println!("bootforge-cli - USB device detection tool");
    println!();
    println!("USAGE:");
    println!("    bootforge-cli [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    --json                  Output in JSON format");
    println!("    --json-file <path>      Write JSON output to file");
    println!("    --report-file <path>    Write scan report to file");
    println!("    --session-log <path>    Write session log to file");
    println!("    --watch                 Watch for device changes in real-time");
    println!("    --apple                 Show only Apple devices");
    println!("    --android               Show only Android devices");
    println!("    --mode <mode>           Filter by device mode");
    println!("    --vendor <id>           Filter by vendor ID (hex)");
    println!("    --help                  Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("    bootforge-cli");
    println!("    bootforge-cli --json");
    println!("    bootforge-cli --apple");
    println!("    bootforge-cli --mode recovery");
    println!("    bootforge-cli --vendor 05ac");
    println!("    bootforge-cli --watch");
    println!("    bootforge-cli --json-file devices.json");
    println!("    bootforge-cli --report-file report.json");
    println!("    bootforge-cli --session-log session.json");
    println!("    bootforge-cli --watch --session-log session.json");
}

fn filter_devices(devices: Vec<DeviceInfo>, options: &CliOptions) -> Vec<DeviceInfo> {
    devices
        .into_iter()
        .filter(|device| {
            if options.apple_only && device.platform != DevicePlatform::Apple {
                return false;
            }
            if options.android_only && device.platform != DevicePlatform::Android {
                return false;
            }
            if let Some(vendor) = options.vendor_filter {
                if device.vendor_id != vendor {
                    return false;
                }
            }
            if let Some(mode) = options.mode_filter {
                if device.mode != mode {
                    return false;
                }
            }
            true
        })
        .collect()
}

fn print_devices_human(devices: &[DeviceInfo]) {
    if devices.is_empty() {
        println!("No USB devices found.");
        return;
    }

    println!("USB Devices Found:\n");
    for device in devices {
        println!(
            "Bus {:03} Device {:03} ID {:04x}:{:04x}",
            device.bus_number, device.address, device.vendor_id, device.product_id
        );
        println!("  Vendor ID              : {:04x}", device.vendor_id);
        println!("  Product ID             : {:04x}", device.product_id);
        println!(
            "  Vendor Name            : {}",
            device.vendor_name.as_deref().unwrap_or("Unknown")
        );
        println!(
            "  Manufacturer           : {}",
            device.manufacturer.as_deref().unwrap_or("Unknown")
        );
        println!(
            "  Product                : {}",
            device.product_name.as_deref().unwrap_or("Unknown")
        );
        println!(
            "  Serial                 : {}",
            device.serial_number.as_deref().unwrap_or("Unknown")
        );
        println!("  Platform               : {:?}", device.platform);
        println!("  Transport              : {:?}", device.transport);
        println!("  Mode                   : {:?}", device.mode);
        println!("  Fingerprint Family     : {:?}", device.fingerprint.family);
        println!(
            "  Model Hint             : {}",
            device
                .fingerprint
                .model_hint
                .as_deref()
                .unwrap_or("Unknown")
        );
        println!(
            "  Fingerprint Confidence : {:?}",
            device.fingerprint.confidence
        );
        println!(
            "  Recommended Workflow   : {:?}",
            device.recommended_workflow
        );
        println!();
    }
}

fn print_devices_json(devices: &[DeviceInfo]) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(devices)?;
    println!("{}", json);
    Ok(())
}

fn write_devices_json(
    devices: &[DeviceInfo],
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(devices)?;
    fs::write(path, json)?;
    println!("Wrote {} devices to {}", devices.len(), path);
    Ok(())
}

fn watch_devices(options: &CliOptions) -> Result<(), Box<dyn std::error::Error>> {
    let mut known_devices: HashSet<(u8, u8, u16, u16)> = HashSet::new();

    println!("Watching for USB device changes (press Ctrl+C to stop)...\n");

    loop {
        let devices = scan_devices()?;
        let filtered_devices = filter_devices(devices, options);

        let mut current_devices: HashSet<(u8, u8, u16, u16)> = HashSet::new();

        // Check for new devices
        for device in &filtered_devices {
            let key = (
                device.bus_number,
                device.address,
                device.vendor_id,
                device.product_id,
            );
            current_devices.insert(key);

            if !known_devices.contains(&key) {
                println!("[+] Device Connected");
                println!(
                    "    Vendor            : {}",
                    device.vendor_name.as_deref().unwrap_or("Unknown")
                );
                println!(
                    "    Product           : {}",
                    device.product_name.as_deref().unwrap_or("Unknown")
                );
                println!("    Mode              : {:?}", device.mode);
                println!("    Family            : {:?}", device.fingerprint.family);
                println!("    Recommended Action: {:?}", device.recommended_workflow);
                println!();
            }
        }

        // Check for removed devices
        for key in &known_devices {
            if !current_devices.contains(key) {
                println!("[-] Device Disconnected");
                println!("    Vendor ID  : {:04x}", key.2);
                println!("    Product ID : {:04x}", key.3);
                println!();
            }
        }

        known_devices = current_devices;
        thread::sleep(Duration::from_secs(2));
    }
}

fn main() {
    let options = CliOptions::parse();

    if options.watch {
        if let Err(e) = watch_devices(&options) {
            eprintln!("Error in watch mode: {}", e);
            std::process::exit(1);
        }
        return;
    }

    // Normal scan mode
    let devices = match scan_devices() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error scanning devices: {}", e);
            std::process::exit(1);
        }
    };

    let filtered_devices = filter_devices(devices, &options);

    // Handle JSON file export
    if let Some(path) = &options.json_file {
        if let Err(e) = write_devices_json(&filtered_devices, path) {
            eprintln!("Error writing JSON file: {}", e);
            std::process::exit(1);
        }
        return;
    }

    // Handle output
    if options.json_output {
        if let Err(e) = print_devices_json(&filtered_devices) {
            eprintln!("Error generating JSON: {}", e);
            std::process::exit(1);
        }
    } else {
        print_devices_human(&filtered_devices);
    }
}
