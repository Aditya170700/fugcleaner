use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Runtime};

use crate::categories::{expand_tilde, get_os_categories, CategoryGroup};
use crate::error::AppResult;
use crate::scanner::size::DiskSizeCalculator;
use crate::scanner::{ScanItem, ScanProgress};

#[derive(Debug, Clone)]
pub struct ScannedItemInternal {
    pub item: ScanItem,
    pub real_path: PathBuf,
    pub allowed_root: Option<PathBuf>,
}

/// Scan all global caches according to OS category definitions
pub fn scan_global_caches<R: Runtime>(
    app_handle: &AppHandle<R>,
    scan_id: &str,
    cancel_flag: Arc<AtomicBool>,
) -> AppResult<Vec<ScannedItemInternal>> {
    let categories = get_os_categories();
    let mut calc = DiskSizeCalculator::new();
    let mut results = Vec::new();
    let mut total_bytes: u64 = 0;

    let mut last_progress_emit = Instant::now() - Duration::from_millis(500);

    // Dedicated categories that might be located inside ~/Library/Caches
    let dedicated_subfolders: HashSet<&str> = [
        "ms-playwright",
        "Yarn",
        "Homebrew",
        "puppeteer",
        "CocoaPods",
    ]
    .into_iter()
    .collect();

    for cat in categories {
        if cancel_flag.load(Ordering::Relaxed) {
            break;
        }

        if cat.group != CategoryGroup::Global {
            continue;
        }

        if cat.split_subfolders {
            // E.g. ~/Library/Caches: scan each application subfolder as a separate item
            for &pattern in cat.paths_patterns {
                if let Some(base_dir) = expand_tilde(pattern) {
                    if !base_dir.exists() || !base_dir.is_dir() {
                        continue;
                    }

                    let entries = match std::fs::read_dir(&base_dir) {
                        Ok(e) => e,
                        Err(_) => continue,
                    };

                    for entry in entries.flatten() {
                        if cancel_flag.load(Ordering::Relaxed) {
                            break;
                        }

                        let path = entry.path();
                        let file_name = entry.file_name().to_string_lossy().to_string();

                        // Skip dedicated subfolders that are separate categories
                        if dedicated_subfolders.contains(file_name.as_str()) {
                            continue;
                        }

                        // Skip if symlink
                        if let Ok(meta) = std::fs::symlink_metadata(&path) {
                            if meta.file_type().is_symlink() {
                                continue;
                            }
                        }

                        let item_size = calc.calculate_path_size(&path, Some(&cancel_flag));
                        if item_size == 0 {
                            continue;
                        }

                        total_bytes += item_size;
                        let item_id = format!("{}:{}", cat.id, file_name);

                        let scan_item = ScanItem {
                            id: item_id,
                            category_id: cat.id.to_string(),
                            path: path.to_string_lossy().to_string(),
                            bytes: item_size,
                            project_name: None,
                            last_activity: None,
                            stale: None,
                            note: Some(format!("Cache {}", file_name)),
                        };

                        results.push(ScannedItemInternal {
                            item: scan_item,
                            real_path: path.clone(),
                            allowed_root: Some(base_dir.clone()),
                        });

                        // Throttled progress emit
                        if last_progress_emit.elapsed() >= Duration::from_millis(100) {
                            let progress = ScanProgress {
                                scan_id: scan_id.to_string(),
                                phase: format!("Memindai {}", cat.name),
                                current_path: path.to_string_lossy().to_string(),
                                items_found: results.len(),
                                bytes_found: total_bytes,
                            };
                            let _ = app_handle.emit("scan://progress", progress);
                            last_progress_emit = Instant::now();
                        }
                    }
                }
            }
        } else if !cat.paths_patterns.is_empty() {
            // Category has specified paths
            for &pattern in cat.paths_patterns {
                if cancel_flag.load(Ordering::Relaxed) {
                    break;
                }

                if let Some(target_path) = expand_tilde(pattern) {
                    if !target_path.exists() {
                        continue;
                    }

                    // Validate symlink
                    if let Ok(meta) = std::fs::symlink_metadata(&target_path) {
                        if meta.file_type().is_symlink() {
                            continue;
                        }
                    }

                    let item_size = calc.calculate_path_size(&target_path, Some(&cancel_flag));
                    if item_size == 0 {
                        continue;
                    }

                    total_bytes += item_size;
                    let item_id = format!("{}:{:x}", cat.id, fxhash(&target_path));

                    let scan_item = ScanItem {
                        id: item_id,
                        category_id: cat.id.to_string(),
                        path: target_path.to_string_lossy().to_string(),
                        bytes: item_size,
                        project_name: None,
                        last_activity: None,
                        stale: None,
                        note: None,
                    };

                    let parent_root = target_path.parent().map(|p| p.to_path_buf());

                    results.push(ScannedItemInternal {
                        item: scan_item,
                        real_path: target_path.clone(),
                        allowed_root: parent_root,
                    });

                    if last_progress_emit.elapsed() >= Duration::from_millis(100) {
                        let progress = ScanProgress {
                            scan_id: scan_id.to_string(),
                            phase: format!("Memindai {}", cat.name),
                            current_path: target_path.to_string_lossy().to_string(),
                            items_found: results.len(),
                            bytes_found: total_bytes,
                        };
                        let _ = app_handle.emit("scan://progress", progress);
                        last_progress_emit = Instant::now();
                    }
                }
            }
        } else if let Some(cli) = cat.cli_name {
            // Command-based category without direct paths (e.g. docker, ios_sim)
            if which::which(cli).is_ok() {
                let item_id = format!("{}:cmd", cat.id);
                let scan_item = ScanItem {
                    id: item_id,
                    category_id: cat.id.to_string(),
                    path: format!("Command: {}", cat.command.unwrap_or(cli)),
                    bytes: 0,
                    project_name: None,
                    last_activity: None,
                    stale: None,
                    note: Some("CLI Command".to_string()),
                };

                results.push(ScannedItemInternal {
                    item: scan_item,
                    real_path: PathBuf::new(),
                    allowed_root: None,
                });
            }
        }
    }

    // Final progress emit
    let _ = app_handle.emit(
        "scan://progress",
        ScanProgress {
            scan_id: scan_id.to_string(),
            phase: if cancel_flag.load(Ordering::Relaxed) {
                "Pemindaian dibatalkan".to_string()
            } else {
                "Pemindaian selesai".to_string()
            },
            current_path: String::new(),
            items_found: results.len(),
            bytes_found: total_bytes,
        },
    );

    Ok(results)
}

fn fxhash(path: &Path) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    hasher.finish()
}
