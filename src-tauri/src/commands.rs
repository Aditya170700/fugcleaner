use serde::{Deserialize, Serialize};
use sysinfo::Disks;
use crate::error::{AppError, AppResult};

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
        println!(
            "Disk info: mount={}, total={} bytes (~{:.2} GB), available={} bytes (~{:.2} GB)",
            info.mount_point,
            info.total,
            info.total as f64 / 1_000_000_000.0,
            info.available,
            info.available as f64 / 1_000_000_000.0,
        );
    }

    #[test]
    fn test_check_full_disk_access() {
        let _ = check_full_disk_access();
    }
}

