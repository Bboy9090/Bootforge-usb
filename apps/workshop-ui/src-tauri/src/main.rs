#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use libbootforge::{scan_devices, DeviceInfo};

#[tauri::command]
fn scan_connected_devices() -> Result<Vec<DeviceInfo>, String> {
    scan_devices().map_err(|error| error.to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![scan_connected_devices])
        .run(tauri::generate_context!())
        .expect("failed to run Phoenix Key desktop application");
}
