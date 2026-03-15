//! Example: List all USB devices in JSON format

use libbootforge::scan_devices_json;

fn main() {
    match scan_devices_json() {
        Ok(json) => {
            println!("{}", json);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
