use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use sysinfo::Disks;
use tauri::{AppHandle, State};

use crate::categories::{list_available_categories, Category};
use crate::error::{AppError, AppResult};
use crate::scanner::global::scan_global_caches;
use crate::scanner::ScanItem;
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
pub async fn scan_global(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    scan_id: String,
) -> AppResult<Vec<ScanItem>> {
    let cancel_flag = Arc::new(AtomicBool::new(false));

    // Register cancel flag in AppState
    {
        let mut active_scans = state.active_scan_cancel.lock().unwrap();
        active_scans.insert(scan_id.clone(), cancel_flag.clone());
    }

    let app_handle_clone = app_handle.clone();
    let scan_id_clone = scan_id.clone();
    let cancel_flag_clone = cancel_flag.clone();

    // Run heavy scanning on a blocking thread
    let result = tokio::task::spawn_blocking(move || {
        scan_global_caches(&app_handle_clone, &scan_id_clone, cancel_flag_clone)
    })
    .await
    .map_err(|e| AppError::Scan(format!("Task spawn blocking error: {}", e)))?;

    // Cleanup cancel flag
    {
        let mut active_scans = state.active_scan_cancel.lock().unwrap();
        active_scans.remove(&scan_id);
    }

    let scanned_items = result?;

    // Store items in AppState managed state
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
    fn test_check_full_disk_access() {
        let _ = check_full_disk_access();
    }
}
