use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

pub mod categories;
pub mod cleaner;
pub mod commands;
pub mod error;
pub mod history;
pub mod safety;
pub mod scanner;
pub mod settings;

use scanner::global::ScannedItemInternal;

/// Shared Application State managed by Tauri
pub struct AppState {
    pub active_scan_cancel: Mutex<HashMap<String, Arc<AtomicBool>>>,
    pub scanned_items: Mutex<HashMap<String, ScannedItemInternal>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active_scan_cancel: Mutex::new(HashMap::new()),
            scanned_items: Mutex::new(HashMap::new()),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_disk_info,
            commands::check_full_disk_access,
            commands::list_categories,
            commands::get_settings,
            commands::save_settings,
            commands::scan_global,
            commands::scan_projects,
            commands::cancel_scan,
            commands::clean_items,
            commands::open_in_file_manager,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
