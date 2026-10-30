pub mod analysis_job;
pub mod analysis_worker;
pub mod audio_devices;
pub mod audio_probe;
pub mod cache;
pub mod catalog;
pub mod library;
pub mod library_commands;
pub mod library_service;
pub mod metadata;
pub mod model_manifest;

#[cfg(not(test))]
use library_commands::LibraryState;
#[cfg(not(test))]
use tauri::Manager;

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
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let application_data = app.path().app_data_dir()?;
            std::fs::create_dir_all(&application_data)?;
            app.manage(LibraryState::new(application_data.join("catalog.sqlite3")));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            runtime_status,
            audio_devices::list_audio_devices,
            audio_probe::play_output_probe,
            audio_probe::probe_input_level,
            library_commands::select_library_folder,
            library_commands::index_selected_library,
            library_commands::load_library_catalog
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Hibiki");
}

