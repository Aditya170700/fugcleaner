use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use sysinfo::Disks;
use tauri::{AppHandle, State};

use crate::categories::{list_available_categories, Category};
use crate::error::{AppError, AppResult};
use crate::scanner::global::scan_global_caches;
use crate::scanner::projects::{scan_projects_in_roots, ProjectScannerOptions, ScannedProject};
use crate::scanner::ScanItem;
use crate::settings::{load_settings, save_settings_to_disk, Settings};
use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskInfo {
    pub total: u64,
    pub available: u64,
    pub mount_point: String,
}

#[tauri::command]
pub fn get_disk_info() -> AppResult<DiskInfo> {
    let disks = Disks::new_with_refreshed_list();
    let primary = disks
        .iter()
        .find(|d| d.mount_point() == std::path::Path::new("/"))
        .or_else(|| disks.first());

    if let Some(disk) = primary {
        Ok(DiskInfo {
            total: disk.total_space(),
            available: disk.available_space(),
            mount_point: disk.mount_point().to_string_lossy().to_string(),
        })
    } else {
        Err(AppError::DiskInfo("Tidak ada disk yang ditemukan".to_string()))
    }
}

#[tauri::command]
pub fn check_full_disk_access() -> bool {
    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs::home_dir() {
            let safari_dir = home.join("Library/Safari");
            if safari_dir.exists() {
                return std::fs::read_dir(safari_dir).is_ok();
            }
        }
        true
    }
    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

#[tauri::command]
pub fn list_categories() -> Vec<Category> {
    list_available_categories()
}

#[tauri::command]
pub fn get_settings() -> Settings {
    load_settings()
}

#[tauri::command]
pub fn save_settings(settings: Settings) -> AppResult<Settings> {
    save_settings_to_disk(&settings)?;
    Ok(settings)
}

#[tauri::command]
pub async fn scan_global(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    scan_id: String,
) -> AppResult<Vec<ScanItem>> {
    let cancel_flag = Arc::new(AtomicBool::new(false));

    {
        let mut active_scans = state.active_scan_cancel.lock().unwrap();
        active_scans.insert(scan_id.clone(), cancel_flag.clone());
    }

    let app_handle_clone = app_handle.clone();
    let scan_id_clone = scan_id.clone();
    let cancel_flag_clone = cancel_flag.clone();

    let result = tokio::task::spawn_blocking(move || {
        scan_global_caches(&app_handle_clone, &scan_id_clone, cancel_flag_clone)
    })
    .await
    .map_err(|e| AppError::Scan(format!("Task spawn blocking error: {}", e)))?;

    {
        let mut active_scans = state.active_scan_cancel.lock().unwrap();
        active_scans.remove(&scan_id);
    }

    let scanned_items = result?;

    let mut frontend_items = Vec::new();
    {
        let mut state_items = state.scanned_items.lock().unwrap();
        for internal in scanned_items {
            frontend_items.push(internal.item.clone());
            state_items.insert(internal.item.id.clone(), internal);
        }
    }

    Ok(frontend_items)
}

#[tauri::command]
pub async fn scan_projects(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    scan_id: String,
    roots: Vec<String>,
) -> AppResult<Vec<ScannedProject>> {
    let settings = load_settings();
    let cancel_flag = Arc::new(AtomicBool::new(false));

    {
        let mut active_scans = state.active_scan_cancel.lock().unwrap();
        active_scans.insert(scan_id.clone(), cancel_flag.clone());
    }

    let roots_paths: Vec<PathBuf> = if roots.is_empty() {
        settings.project_roots.iter().map(PathBuf::from).collect()
    } else {
        roots.iter().map(PathBuf::from).collect()
    };

    let exclude_paths: Vec<PathBuf> = settings.exclude_paths.iter().map(PathBuf::from).collect();

    let options = ProjectScannerOptions {
        roots: roots_paths,
        stale_threshold_days: settings.stale_threshold_days,
        max_depth: settings.max_scan_depth,
        exclude_paths,
    };

    let app_handle_clone = app_handle.clone();
    let scan_id_clone = scan_id.clone();
    let cancel_flag_clone = cancel_flag.clone();

    let result = tokio::task::spawn_blocking(move || {
        scan_projects_in_roots(&app_handle_clone, &scan_id_clone, options, cancel_flag_clone)
    })
    .await
    .map_err(|e| AppError::Scan(format!("Task spawn blocking error: {}", e)))?;

    {
        let mut active_scans = state.active_scan_cancel.lock().unwrap();
        active_scans.remove(&scan_id);
    }

    let (projects, scanned_items) = result?;

    {
        let mut state_items = state.scanned_items.lock().unwrap();
        for internal in scanned_items {
            state_items.insert(internal.item.id.clone(), internal);
        }
    }

    Ok(projects)
}

#[tauri::command]
pub fn cancel_scan(state: State<'_, AppState>, scan_id: String) {
    let active_scans = state.active_scan_cancel.lock().unwrap();
    if let Some(cancel_flag) = active_scans.get(&scan_id) {
        cancel_flag.store(true, Ordering::Relaxed);
    }
}

#[tauri::command]
pub fn open_in_file_manager(
    state: State<'_, AppState>,
    item_id: String,
) -> AppResult<()> {
    let state_items = state.scanned_items.lock().unwrap();
    if let Some(internal) = state_items.get(&item_id) {
        let path = &internal.real_path;
        if path.exists() {
            let _ = tauri_plugin_opener::reveal_item_in_dir(path);
            return Ok(());
        }
    }
    Err(AppError::Custom(format!("Item ID '{}' tidak ditemukan atau path tidak ada.", item_id)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_disk_info() {
        let result = get_disk_info();
        assert!(result.is_ok());
        let info = result.unwrap();
        assert!(info.total > 0, "Total disk space must be greater than 0");
        assert!(info.available > 0, "Available disk space must be greater than 0");
        assert!(info.total >= info.available, "Total must be >= available");
    }

    #[test]
    fn test_list_categories() {
        let cats = list_categories();
        assert!(!cats.is_empty(), "Categories must not be empty on macOS");
        let npm = cats.iter().find(|c| c.id == "npm_cache");
        assert!(npm.is_some(), "npm_cache must exist in categories");
    }

    #[test]
    fn test_settings_load_save() {
        let mut settings = load_settings();
        settings.stale_threshold_days = 45;
        let saved = save_settings(settings.clone());
        assert!(saved.is_ok());
        assert_eq!(load_settings().stale_threshold_days, 45);
    }
}
