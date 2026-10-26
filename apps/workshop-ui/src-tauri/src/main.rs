#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use libbootforge::{efi_planner::{plan_efi_recovery, EfiObservation, EfiPlan}, scan_devices, DeviceInfo};

#[tauri::command]
fn scan_connected_devices() -> Result<Vec<DeviceInfo>, String> {
    scan_devices().map_err(|error| error.to_string())
}

/// Plans a recovery from caller-provided observations; does not inspect or modify disks.
#[tauri::command]
fn plan_efi_recovery_readonly(observation: EfiObservation) -> EfiPlan {
    plan_efi_recovery(&observation)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![scan_connected_devices, plan_efi_recovery_readonly])
        .run(tauri::generate_context!())
        .expect("failed to run Phoenix Key desktop application");
}
