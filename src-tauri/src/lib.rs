pub mod categories;
pub mod cleaner;
pub mod commands;
pub mod error;
pub mod history;
pub mod safety;
pub mod scanner;
pub mod settings;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_disk_info,
            commands::check_full_disk_access,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
