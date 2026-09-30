#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use libbootforge::{
    host_matrix, scan_devices, DeviceInfo, HostPlatform, MediaCapability,
};

#[tauri::command]
fn scan_connected_devices() -> Result<Vec<DeviceInfo>, String> {
    scan_devices().map_err(|error| error.to_string())
}

#[tauri::command]
fn media_capability_matrix() -> Vec<MediaCapability> {
    host_matrix(HostPlatform::current())
}

#[tauri::command]
fn current_host_platform() -> HostPlatform {
    HostPlatform::current()
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            scan_connected_devices,
            media_capability_matrix,
            current_host_platform
        ])
        .run(tauri::generate_context!())
        .expect("failed to run BootForge desktop application");
}
