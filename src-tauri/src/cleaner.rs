use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Runtime};

use crate::categories::Method;
use crate::safety;
use crate::scanner::global::ScannedItemInternal;
use crate::settings::Settings;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanFailedItem {
    pub id: String,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanResult {
    pub freed_bytes: u64,
    pub succeeded: Vec<String>,
    pub failed: Vec<CleanFailedItem>,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanProgress {
    pub done: usize,
    pub total: usize,
    pub current_path: String,
}

/// Core function to perform cleaning with strict safety validation.
/// Returns the CleanResult summary and a list of succeeded item IDs to remove from state.
pub fn clean_items_core<R: Runtime>(
    app_handle: Option<&AppHandle<R>>,
    item_ids: &[String],
    scanned_items: &HashMap<String, ScannedItemInternal>,
    settings: &Settings,
    dry_run: bool,
) -> (CleanResult, Vec<String>) {
    let mut freed_bytes: u64 = 0;
    let mut succeeded: Vec<String> = Vec::new();
    let mut failed: Vec<CleanFailedItem> = Vec::new();
    let total_items = item_ids.len();

    let mut last_progress_emit = Instant::now() - Duration::from_millis(500);

    for (index, id) in item_ids.iter().enumerate() {
        let internal = match scanned_items.get(id) {
            Some(item) => item,
            None => {
                failed.push(CleanFailedItem {
                    id: id.clone(),
                    error: format!("Item ID '{}' tidak ditemukan dalam hasil scan aktif.", id),
                });
                continue;
            }
        };

        // Emit progress periodically or for every item if small
        if let Some(handle) = app_handle {
            if total_items <= 50 || last_progress_emit.elapsed() >= Duration::from_millis(50) {
                let _ = handle.emit(
                    "clean://progress",
                    CleanProgress {
                        done: index,
                        total: total_items,
                        current_path: internal.item.path.clone(),
                    },
                );
                last_progress_emit = Instant::now();
            }
        }

        // 1. Handle Command Method
        if internal.method == Method::Command {
            if let Some(ref cmd_str) = internal.command {
                if dry_run {
                    succeeded.push(id.clone());
                    freed_bytes += internal.item.bytes;
                    continue;
                }

                match execute_clean_command(cmd_str) {
                    Ok(_) => {
                        succeeded.push(id.clone());
                        freed_bytes += internal.item.bytes;
                    }
                    Err(e) => {
                        failed.push(CleanFailedItem {
                            id: id.clone(),
                            error: format!("Gagal menjalankan perintah '{}': {}", cmd_str, e),
                        });
                    }
                }
                continue;
            } else {
                failed.push(CleanFailedItem {
                    id: id.clone(),
                    error: "Definisi perintah CLI kosong.".to_string(),
                });
                continue;
            }
        }

        // 2. Handle File / Directory Cleaning (Trash or Delete)
        let target_path = &internal.real_path;

        // Verify target path exists
        if !target_path.exists() {
            failed.push(CleanFailedItem {
                id: id.clone(),
                error: format!("Path '{}' sudah tidak ditemukan di disk.", target_path.display()),
            });
            continue;
        }

        // Check Hard-Deny list
        if safety::is_hard_denied(target_path) {
            failed.push(CleanFailedItem {
                id: id.clone(),
                error: format!(
                    "Path '{}' termasuk dalam hard-deny list keamanan sistem.",
                    target_path.display()
                ),
            });
            continue;
        }

        // Validate allowed roots and symlink rejection
        let allowed_roots = match &internal.allowed_root {
            Some(root) => vec![root.clone()],
            None => Vec::new(),
        };

        let validated_path = match safety::is_path_allowed(target_path, &allowed_roots) {
            Ok(p) => p,
            Err(e) => {
                failed.push(CleanFailedItem {
                    id: id.clone(),
                    error: format!(
                        "Validasi keamanan gagal untuk '{}': {}",
                        target_path.display(),
                        e
                    ),
                });
                continue;
            }
        };

        // Check user exclude list
        if is_path_in_exclude_list(&validated_path, &settings.exclude_paths) {
            failed.push(CleanFailedItem {
                id: id.clone(),
                error: format!(
                    "Path '{}' dikecualikan dalam pengaturan (exclude list).",
                    validated_path.display()
                ),
            });
            continue;
        }

        // If dry run, record success without touching disk
        if dry_run {
            succeeded.push(id.clone());
            freed_bytes += internal.item.bytes;
            continue;
        }

        // Perform actual deletion
        let clean_op = match internal.method {
            Method::Trash => trash::delete(&validated_path)
                .map_err(|e| format!("Gagal memindahkan ke Trash: {}", e)),
            Method::Delete => {
                if validated_path.is_dir() {
                    std::fs::remove_dir_all(&validated_path)
                        .map_err(|e| format!("Gagal menghapus direktori: {}", e))
                } else if validated_path.is_file() {
                    std::fs::remove_file(&validated_path)
                        .map_err(|e| format!("Gagal menghapus file: {}", e))
                } else {
                    Err("Target bukan merupakan file atau direktori".to_string())
                }
            }
            Method::Command => Ok(()),
        };

        match clean_op {
            Ok(_) => {
                succeeded.push(id.clone());
                freed_bytes += internal.item.bytes;
            }
            Err(e) => {
                failed.push(CleanFailedItem {
                    id: id.clone(),
                    error: e,
                });
            }
        }
    }

    // Final progress emit (100%)
    if let Some(handle) = app_handle {
        let _ = handle.emit(
            "clean://progress",
            CleanProgress {
                done: total_items,
                total: total_items,
                current_path: String::new(),
            },
        );
    }

    let result = CleanResult {
        freed_bytes,
        succeeded: succeeded.clone(),
        failed,
        dry_run,
    };

    (result, succeeded)
}

/// Helper to check if a path is matched by user exclude list
fn is_path_in_exclude_list(path: &Path, exclude_paths: &[String]) -> bool {
    for pattern in exclude_paths {
        let exc_path = PathBuf::from(pattern);
        if let Ok(can_exc) = exc_path.canonicalize() {
            if path.starts_with(&can_exc) {
                return true;
            }
        } else if path.starts_with(&exc_path) {
            return true;
        }
    }
    false
}

/// Helper to execute shell command safely
fn execute_clean_command(cmd_str: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let mut cmd = std::process::Command::new("cmd");
    #[cfg(target_os = "windows")]
    cmd.args(["/C", cmd_str]);

    #[cfg(not(target_os = "windows"))]
    let mut cmd = std::process::Command::new("sh");
    #[cfg(not(target_os = "windows"))]
    cmd.args(["-c", cmd_str]);

    let output = cmd.output().map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        Err(if err.trim().is_empty() {
            format!("Perintah gagal dengan kode status {:?}", output.status.code())
        } else {
            err.trim().to_string()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::categories::Method;
    use crate::scanner::ScanItem;
    use std::fs;
    use tempfile::TempDir;

    fn create_dummy_scanned_item(
        id: &str,
        path: PathBuf,
        root: Option<PathBuf>,
        method: Method,
        bytes: u64,
    ) -> ScannedItemInternal {
        ScannedItemInternal {
            item: ScanItem {
                id: id.to_string(),
                category_id: "test".to_string(),
                path: path.to_string_lossy().to_string(),
                bytes,
                project_name: None,
                last_activity: None,
                stale: None,
                note: None,
            },
            real_path: path,
            allowed_root: root,
            method,
            command: None,
        }
    }

    #[test]
    fn test_unknown_item_id_fails() {
        let items: HashMap<String, ScannedItemInternal> = HashMap::new();
        let settings = Settings::default();

        let (result, succeeded) = clean_items_core::<tauri::Wry>(
            None,
            &["unknown_123".to_string()],
            &items,
            &settings,
            true,
        );

        assert_eq!(result.succeeded.len(), 0);
        assert_eq!(succeeded.len(), 0);
        assert_eq!(result.failed.len(), 1);
        assert_eq!(result.failed[0].id, "unknown_123");
    }

    #[test]
    fn test_dry_run_does_not_delete_files() {
        let temp = TempDir::new().unwrap();
        let root = temp.path().to_path_buf();
        let target_dir = root.join("node_modules");
        fs::create_dir_all(&target_dir).unwrap();
        fs::write(target_dir.join("package.json"), "{}").unwrap();

        let mut items = HashMap::new();
        let item = create_dummy_scanned_item(
            "item_1",
            target_dir.clone(),
            Some(root),
            Method::Delete,
            1024,
        );
        items.insert("item_1".to_string(), item);

        let settings = Settings::default();

        // Run with dry_run = true
        let (result, succeeded) = clean_items_core::<tauri::Wry>(
            None,
            &["item_1".to_string()],
            &items,
            &settings,
            true,
        );

        assert_eq!(result.succeeded.len(), 1);
        assert_eq!(succeeded.len(), 1);
        assert_eq!(result.freed_bytes, 1024);
        assert!(result.dry_run);

        // Verify folder still exists on disk
        assert!(target_dir.exists(), "Folder must still exist during dry-run");
    }

    #[test]
    fn test_live_delete_removes_files() {
        let temp = TempDir::new().unwrap();
        let root = temp.path().to_path_buf();
        let target_dir = root.join("target");
        fs::create_dir_all(&target_dir).unwrap();
        fs::write(target_dir.join("build.rs"), "// test").unwrap();

        let mut items = HashMap::new();
        let item = create_dummy_scanned_item(
            "item_2",
            target_dir.clone(),
            Some(root),
            Method::Delete,
            2048,
        );
        items.insert("item_2".to_string(), item);

        let settings = Settings::default();

        // Run with dry_run = false
        let (result, succeeded) = clean_items_core::<tauri::Wry>(
            None,
            &["item_2".to_string()],
            &items,
            &settings,
            false,
        );

        assert_eq!(result.succeeded.len(), 1);
        assert_eq!(succeeded.len(), 1);
        assert_eq!(result.freed_bytes, 2048);
        assert!(!result.dry_run);

        // Verify folder is deleted from disk
        assert!(!target_dir.exists(), "Folder must be deleted on disk");
    }

    #[test]
    fn test_hard_deny_item_rejected() {
        let mut items = HashMap::new();
        let item = create_dummy_scanned_item(
            "item_root",
            PathBuf::from("/"),
            None,
            Method::Delete,
            999999,
        );
        items.insert("item_root".to_string(), item);

        let settings = Settings::default();

        let (result, succeeded) = clean_items_core::<tauri::Wry>(
            None,
            &["item_root".to_string()],
            &items,
            &settings,
            false,
        );

        assert_eq!(result.succeeded.len(), 0);
        assert_eq!(succeeded.len(), 0);
        assert_eq!(result.failed.len(), 1);
        assert!(result.failed[0].error.contains("hard-deny"));
    }

    #[test]
    fn test_symlink_item_rejected() {
        let temp = TempDir::new().unwrap();
        let root = temp.path().to_path_buf();
        let real_dir = root.join("real");
        fs::create_dir_all(&real_dir).unwrap();

        let symlink_path = root.join("symlink");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&real_dir, &symlink_path).unwrap();

        #[cfg(unix)]
        {
            let mut items = HashMap::new();
            let item = create_dummy_scanned_item(
                "item_sym",
                symlink_path.clone(),
                Some(root),
                Method::Delete,
                512,
            );
            items.insert("item_sym".to_string(), item);

            let settings = Settings::default();

            let (result, succeeded) = clean_items_core::<tauri::Wry>(
                None,
                &["item_sym".to_string()],
                &items,
                &settings,
                false,
            );

            assert_eq!(result.succeeded.len(), 0);
            assert_eq!(succeeded.len(), 0);
            assert_eq!(result.failed.len(), 1);
            assert!(result.failed[0].error.contains("symlink"));
            assert!(real_dir.exists(), "Target real directory must not be deleted");
        }
    }

    #[test]
    fn test_partial_failure_continues_processing() {
        let temp = TempDir::new().unwrap();
        let root = temp.path().to_path_buf();

        let valid_dir = root.join("valid_node_modules");
        fs::create_dir_all(&valid_dir).unwrap();

        let mut items = HashMap::new();
        // Item 1 is non-existent
        let item1 = create_dummy_scanned_item(
            "bad_item",
            root.join("non_existent"),
            Some(root.clone()),
            Method::Delete,
            100,
        );
        // Item 2 is valid
        let item2 = create_dummy_scanned_item(
            "good_item",
            valid_dir.clone(),
            Some(root),
            Method::Delete,
            500,
        );

        items.insert("bad_item".to_string(), item1);
        items.insert("good_item".to_string(), item2);

        let settings = Settings::default();

        let (result, succeeded) = clean_items_core::<tauri::Wry>(
            None,
            &["bad_item".to_string(), "good_item".to_string()],
            &items,
            &settings,
            false,
        );

        assert_eq!(result.failed.len(), 1);
        assert_eq!(result.failed[0].id, "bad_item");
        assert_eq!(result.succeeded.len(), 1);
        assert_eq!(result.succeeded[0], "good_item");
        assert_eq!(succeeded.len(), 1);
        assert_eq!(result.freed_bytes, 500);
        assert!(!valid_dir.exists());
    }
}
