use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryCleanedItem {
    pub category_id: String,
    pub path: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: String,
    pub timestamp: u64,
    pub freed_bytes: u64,
    pub item_count: usize,
    pub dry_run: bool,
    pub items: Vec<HistoryCleanedItem>,
}

pub fn get_history_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("fugcleaner/history.json"))
}

pub fn load_history() -> Vec<HistoryEntry> {
    if let Some(path) = get_history_path() {
        load_history_from_path(&path)
    } else {
        Vec::new()
    }
}

pub fn load_history_from_path(path: &Path) -> Vec<HistoryEntry> {
    if path.exists() {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(mut entries) = serde_json::from_str::<Vec<HistoryEntry>>(&content) {
                // Sort newest first
                entries.sort_by_key(|a| std::cmp::Reverse(a.timestamp));
                return entries;
            }
        }
    }
    Vec::new()
}

pub fn append_history_entry(entry: HistoryEntry) -> AppResult<()> {
    if let Some(path) = get_history_path() {
        append_history_entry_to_path(&path, entry)
    } else {
        Err(AppError::Custom("Direktori config tidak ditemukan.".to_string()))
    }
}

pub fn append_history_entry_to_path(path: &Path, entry: HistoryEntry) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            AppError::Custom(format!("Gagal membuat direktori history: {}", e))
        })?;
    }

    let mut entries = load_history_from_path(path);
    entries.insert(0, entry);

    // Limit history entries to 1000 items to avoid unbounded growth
    if entries.len() > 1000 {
        entries.truncate(1000);
    }

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| AppError::Custom(format!("Gagal serialisasi history: {}", e)))?;

    fs::write(path, json)
        .map_err(|e| AppError::Custom(format!("Gagal menyimpan history.json: {}", e)))?;

    Ok(())
}

pub fn clear_history() -> AppResult<()> {
    if let Some(path) = get_history_path() {
        clear_history_from_path(&path)
    } else {
        Ok(())
    }
}

pub fn clear_history_from_path(path: &Path) -> AppResult<()> {
    if path.exists() {
        fs::remove_file(path).map_err(|e| {
            AppError::Custom(format!("Gagal menghapus riwayat: {}", e))
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_history_append_and_load() {
        let temp = TempDir::new().unwrap();
        let hist_file = temp.path().join("history.json");

        // Initially empty
        let initial = load_history_from_path(&hist_file);
        assert_eq!(initial.len(), 0);

        let entry1 = HistoryEntry {
            id: "hist-1".to_string(),
            timestamp: 1000,
            freed_bytes: 1024,
            item_count: 2,
            dry_run: true,
            items: vec![
                HistoryCleanedItem {
                    category_id: "npm_cache".to_string(),
                    path: "/tmp/npm".to_string(),
                    bytes: 512,
                },
                HistoryCleanedItem {
                    category_id: "yarn_cache".to_string(),
                    path: "/tmp/yarn".to_string(),
                    bytes: 512,
                },
            ],
        };

        let entry2 = HistoryEntry {
            id: "hist-2".to_string(),
            timestamp: 2000,
            freed_bytes: 2048,
            item_count: 1,
            dry_run: false,
            items: vec![
                HistoryCleanedItem {
                    category_id: "rust_target".to_string(),
                    path: "/tmp/target".to_string(),
                    bytes: 2048,
                },
            ],
        };

        append_history_entry_to_path(&hist_file, entry1).unwrap();
        append_history_entry_to_path(&hist_file, entry2).unwrap();

        let entries = load_history_from_path(&hist_file);
        assert_eq!(entries.len(), 2);
        // Newest timestamp first
        assert_eq!(entries[0].id, "hist-2");
        assert_eq!(entries[0].freed_bytes, 2048);
        assert!(!entries[0].dry_run);

        assert_eq!(entries[1].id, "hist-1");
        assert_eq!(entries[1].freed_bytes, 1024);
        assert!(entries[1].dry_run);

        // Test clear
        clear_history_from_path(&hist_file).unwrap();
        let after_clear = load_history_from_path(&hist_file);
        assert_eq!(after_clear.len(), 0);
    }
}
