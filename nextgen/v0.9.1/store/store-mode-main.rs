use bootforge_core::{chromebook::ChromebookProfile, HostPlatform, MediaTarget};
use bootforge_desktop_controller::{analyze_source as analyze, SourceAnalysis};

#[tauri::command]
fn host_platform() -> HostPlatform {
    HostPlatform::current()
}

#[tauri::command]
fn store_mode() -> bool {
    true
}

#[tauri::command]
async fn analyze_source(path: String) -> Result<SourceAnalysis, String> {
    tauri::async_runtime::spawn_blocking(move || analyze(path).map_err(|e| e.to_string()))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn scan_devices() -> Result<Vec<serde_json::Value>, String> {
    Ok(Vec::new())
}

#[tauri::command]
async fn plan_media(
    _source: SourceAnalysis,
    _target: MediaTarget,
    _selected_device_id: String,
    _chromebook: Option<ChromebookProfile>,
) -> Result<serde_json::Value, String> {
    Err("BootForge Store/TestFlight mode is read-only. USB writing is available only in the separately signed hardware-test build.".into())
}

#[tauri::command]
async fn create_media(
    _source: SourceAnalysis,
    _target: MediaTarget,
    _selected_device_id: String,
    _chromebook: Option<ChromebookProfile>,
    _confirmation: String,
    _volume_label: Option<String>,
) -> Result<serde_json::Value, String> {
    Err("BootForge Store/TestFlight mode does not perform destructive disk writes.".into())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            host_platform,
            store_mode,
            analyze_source,
            scan_devices,
            plan_media,
            create_media
        ])
        .run(tauri::generate_context!())
        .expect("failed to run BootForge Store/TestFlight application");
}
