pub mod cache;
pub mod catalog;
pub mod library;
pub mod library_service;
pub mod metadata;

#[cfg(not(test))]
#[tauri::command]
fn runtime_status() -> serde_json::Value {
    serde_json::json!({
        "audioEngine": "not-initialized",
        "inference": "cpu-fallback",
        "privacy": "local-only"
    })
}

#[cfg(not(test))]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![runtime_status])
        .run(tauri::generate_context!())
        .expect("failed to run Hibiki");
}

