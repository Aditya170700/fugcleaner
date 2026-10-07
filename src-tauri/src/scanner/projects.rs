use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Runtime};

use crate::categories::{Method, Risk};
use crate::error::AppResult;
use crate::safety::is_path_allowed;
use crate::scanner::global::ScannedItemInternal;
use crate::scanner::size::DiskSizeCalculator;
use crate::scanner::{ScanItem, ScanProgress};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectArtifact {
    pub id: String,
    pub category_id: String,
    pub name: String,
    pub path: String,
    pub bytes: u64,
    pub risk: Risk,
    pub method: Method,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScannedProject {
    pub id: String,
    pub name: String,
    pub path: String,
    pub project_type: String,
    pub last_activity: u64, // unix ms
    pub stale: bool,
    pub total_bytes: u64,
    pub artifacts: Vec<ProjectArtifact>,
}

pub struct ProjectScannerOptions {
    pub roots: Vec<PathBuf>,
    pub stale_threshold_days: u32,
    pub max_depth: usize,
    pub exclude_paths: Vec<PathBuf>,
}

/// Scan project directories for markers and developer artifacts
pub fn scan_projects_in_roots<R: Runtime>(
    app_handle: &AppHandle<R>,
    scan_id: &str,
    options: ProjectScannerOptions,
    cancel_flag: Arc<AtomicBool>,
) -> AppResult<(Vec<ScannedProject>, Vec<ScannedItemInternal>)> {
    let mut calc = DiskSizeCalculator::new();
    let mut scanned_projects = Vec::new();
    let mut scanned_items = Vec::new();

    let mut last_progress_emit = Instant::now() - Duration::from_millis(500);
    let mut total_bytes_found: u64 = 0;
    let mut total_items_found: usize = 0;

    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let stale_ms_threshold = (options.stale_threshold_days as u64) * 24 * 60 * 60 * 1000;

    let exclude_set: HashSet<PathBuf> = options
        .exclude_paths
        .iter()
        .filter_map(|p| p.canonicalize().ok())
        .collect();

    for root in &options.roots {
        if cancel_flag.load(Ordering::Relaxed) {
            break;
        }

        if !root.exists() || !root.is_dir() {
            continue;
        }

        let canonical_root = match root.canonicalize() {
            Ok(r) => r,
            Err(_) => continue,
        };

        if exclude_set.contains(&canonical_root) {
            continue;
        }

        // Walk directories respecting max_depth and pruning
        traverse_and_scan_projects(
            app_handle,
            scan_id,
            &canonical_root,
            &canonical_root,
            0,
            options.max_depth,
            &exclude_set,
            now_ms,
            stale_ms_threshold,
            &mut calc,
            &mut scanned_projects,
            &mut scanned_items,
            &mut total_bytes_found,
            &mut total_items_found,
            &mut last_progress_emit,
            &cancel_flag,
        );
    }

    // Final progress emit
    let _ = app_handle.emit(
        "scan://progress",
        ScanProgress {
            scan_id: scan_id.to_string(),
            phase: if cancel_flag.load(Ordering::Relaxed) {
                "Pemindaian project dibatalkan".to_string()
            } else {
                "Pemindaian project selesai".to_string()
            },
            current_path: String::new(),
            items_found: total_items_found,
            bytes_found: total_bytes_found,
        },
    );

    Ok((scanned_projects, scanned_items))
}

#[allow(clippy::too_many_arguments)]
fn traverse_and_scan_projects<R: Runtime>(
    app_handle: &AppHandle<R>,
    scan_id: &str,
    current_dir: &Path,
    root: &Path,
    current_depth: usize,
    max_depth: usize,
    exclude_set: &HashSet<PathBuf>,
    now_ms: u64,
    stale_ms_threshold: u64,
    calc: &mut DiskSizeCalculator,
    projects_out: &mut Vec<ScannedProject>,
    items_out: &mut Vec<ScannedItemInternal>,
    total_bytes: &mut u64,
    total_items: &mut usize,
    last_emit: &mut Instant,
    cancel_flag: &AtomicBool,
) {
    if cancel_flag.load(Ordering::Relaxed) {
        return;
    }

    if current_depth > max_depth {
        return;
    }

    // Check exclude list
    if let Ok(canon) = current_dir.canonicalize() {
        if exclude_set.contains(&canon) {
            return;
        }
    }

    let dir_name = current_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    // Skip ignored folders
    if dir_name == ".git"
        || dir_name == "node_modules"
        || dir_name == "target"
        || dir_name == "Library"
        || dir_name == ".Trash"
    {
        return;
    }

    // Check if current directory has project markers
    let has_package_json = current_dir.join("package.json").is_file();
    let has_cargo_toml = current_dir.join("Cargo.toml").is_file();
    let has_composer_json = current_dir.join("composer.json").is_file();
    let has_pyproject = current_dir.join("pyproject.toml").is_file()
        || current_dir.join("requirements.txt").is_file();
    let has_go_mod = current_dir.join("go.mod").is_file();

    let is_project = has_package_json
        || has_cargo_toml
        || has_composer_json
        || has_pyproject
        || has_go_mod;

    if is_project {
        let project_name = dir_name.clone();
        let project_type = determine_project_type(
            has_package_json,
            has_cargo_toml,
            has_composer_json,
            has_pyproject,
            has_go_mod,
        );

        let last_activity = calculate_last_activity(current_dir);
        let stale = if last_activity > 0 {
            now_ms.saturating_sub(last_activity) > stale_ms_threshold
        } else {
            true
        };

        // Parse gitignore for build/coverage rules
        let ignored_patterns = parse_project_gitignore(current_dir);

        // Find all artifacts in this project
        let mut artifacts = Vec::new();
        let project_id = format!("proj:{:x}", fxhash(current_dir));

        detect_project_artifacts(
            current_dir,
            &project_id,
            &project_name,
            last_activity,
            stale,
            has_package_json,
            has_cargo_toml,
            has_composer_json,
            has_pyproject,
            &ignored_patterns,
            calc,
            &mut artifacts,
            items_out,
            total_bytes,
            total_items,
            root,
            cancel_flag,
        );

        let project_total_bytes: u64 = artifacts.iter().map(|a| a.bytes).sum();

        if !artifacts.is_empty() {
            projects_out.push(ScannedProject {
                id: project_id,
                name: project_name,
                path: current_dir.to_string_lossy().to_string(),
                project_type,
                last_activity,
                stale,
                total_bytes: project_total_bytes,
                artifacts,
            });
        }

        // Throttled progress event
        if last_emit.elapsed() >= Duration::from_millis(100) {
            let progress = ScanProgress {
                scan_id: scan_id.to_string(),
                phase: format!("Memindai {}", current_dir.display()),
                current_path: current_dir.to_string_lossy().to_string(),
                items_found: *total_items,
                bytes_found: *total_bytes,
            };
            let _ = app_handle.emit("scan://progress", progress);
            *last_emit = Instant::now();
        }

        // PRUNING: After finding a project, we still check subfolders (e.g. monorepo / subpackages)
        // BUT we will NOT enter node_modules, target, .git (filtered above).
    }

    // Traverse subdirectories
    let entries = match fs::read_dir(current_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        if cancel_flag.load(Ordering::Relaxed) {
            break;
        }

        let path = entry.path();
        if let Ok(meta) = fs::symlink_metadata(&path) {
            if meta.is_dir() && !meta.file_type().is_symlink() {
                traverse_and_scan_projects(
                    app_handle,
                    scan_id,
                    &path,
                    root,
                    current_depth + 1,
                    max_depth,
                    exclude_set,
                    now_ms,
                    stale_ms_threshold,
                    calc,
                    projects_out,
                    items_out,
                    total_bytes,
                    total_items,
                    last_emit,
                    cancel_flag,
                );
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn detect_project_artifacts(
    project_dir: &Path,
    project_id: &str,
    project_name: &str,
    last_activity: u64,
    stale: bool,
    has_package_json: bool,
    has_cargo_toml: bool,
    has_composer_json: bool,
    has_pyproject: bool,
    ignored_patterns: &HashSet<String>,
    calc: &mut DiskSizeCalculator,
    artifacts_out: &mut Vec<ProjectArtifact>,
    items_out: &mut Vec<ScannedItemInternal>,
    total_bytes: &mut u64,
    total_items: &mut usize,
    root: &Path,
    cancel_flag: &AtomicBool,
) {
    let pnpm_project = project_dir.join("pnpm-lock.yaml").is_file();

    // 1. node_modules/
    if has_package_json {
        let node_modules = project_dir.join("node_modules");
        if node_modules.is_dir() {
            let bytes = calc.calculate_path_size(&node_modules, Some(cancel_flag));
            if bytes > 0 {
                let note = if pnpm_project {
                    Some("pnpm (hardlinked)".to_string())
                } else {
                    None
                };

                let risk = if stale { Risk::Safe } else { Risk::Caution };
                add_artifact(
                    "node_modules",
                    "node_modules",
                    &node_modules,
                    bytes,
                    risk,
                    Method::Trash,
                    note,
                    project_id,
                    project_name,
                    last_activity,
                    stale,
                    root,
                    artifacts_out,
                    items_out,
                    total_bytes,
                    total_items,
                );
            }
        }
    }

    // 2. target/ (Rust)
    if has_cargo_toml {
        let target_dir = project_dir.join("target");
        if target_dir.is_dir() {
            let bytes = calc.calculate_path_size(&target_dir, Some(cancel_flag));
            if bytes > 0 {
                let risk = if stale { Risk::Safe } else { Risk::Caution };
                add_artifact(
                    "rust_target",
                    "target",
                    &target_dir,
                    bytes,
                    risk,
                    Method::Delete,
                    None,
                    project_id,
                    project_name,
                    last_activity,
                    stale,
                    root,
                    artifacts_out,
                    items_out,
                    total_bytes,
                    total_items,
                );
            }
        }
    }

    // 3. .next/
    if has_package_json {
        check_simple_dir_artifact(
            project_dir,
            ".next",
            "next",
            ".next",
            Risk::Safe,
            Method::Delete,
            None,
            project_id,
            project_name,
            last_activity,
            stale,
            root,
            calc,
            artifacts_out,
            items_out,
            total_bytes,
            total_items,
            cancel_flag,
        );
    }

    // 4. .nuxt/ & .output/
    if has_package_json {
        check_simple_dir_artifact(
            project_dir,
            ".nuxt",
            "nuxt",
            ".nuxt",
            Risk::Safe,
            Method::Delete,
            None,
            project_id,
            project_name,
            last_activity,
            stale,
            root,
            calc,
            artifacts_out,
            items_out,
            total_bytes,
            total_items,
            cancel_flag,
        );
        check_simple_dir_artifact(
            project_dir,
            ".output",
            "nuxt",
            ".output",
            Risk::Safe,
            Method::Delete,
            None,
            project_id,
            project_name,
            last_activity,
            stale,
            root,
            calc,
            artifacts_out,
            items_out,
            total_bytes,
            total_items,
            cancel_flag,
        );
    }

    // 5. .svelte-kit/
    if has_package_json {
        check_simple_dir_artifact(
            project_dir,
            ".svelte-kit",
            "svelte_kit",
            ".svelte-kit",
            Risk::Safe,
            Method::Delete,
            None,
            project_id,
            project_name,
            last_activity,
            stale,
            root,
            calc,
            artifacts_out,
            items_out,
            total_bytes,
            total_items,
            cancel_flag,
        );
    }

    // 6. .turbo/
    check_simple_dir_artifact(
        project_dir,
        ".turbo",
        "turbo",
        ".turbo",
        Risk::Safe,
        Method::Delete,
        None,
        project_id,
        project_name,
        last_activity,
        stale,
        root,
        calc,
        artifacts_out,
        items_out,
        total_bytes,
        total_items,
        cancel_flag,
    );

    // 7. .parcel-cache/
    check_simple_dir_artifact(
        project_dir,
        ".parcel-cache",
        "parcel",
        ".parcel-cache",
        Risk::Safe,
        Method::Delete,
        None,
        project_id,
        project_name,
        last_activity,
        stale,
        root,
        calc,
        artifacts_out,
        items_out,
        total_bytes,
        total_items,
        cancel_flag,
    );

    // 8. dist/, build/, out/ (only if in .gitignore and has package.json)
    if has_package_json {
        for build_name in &["dist", "build", "out"] {
            if ignored_patterns.contains(*build_name) {
                check_simple_dir_artifact(
                    project_dir,
                    build_name,
                    "build_out",
                    build_name,
                    Risk::Caution,
                    Method::Trash,
                    Some("Build Output".to_string()),
                    project_id,
                    project_name,
                    last_activity,
                    stale,
                    root,
                    calc,
                    artifacts_out,
                    items_out,
                    total_bytes,
                    total_items,
                    cancel_flag,
                );
            }
        }
    }

    // 9. coverage/ (if in .gitignore)
    if ignored_patterns.contains("coverage") {
        check_simple_dir_artifact(
            project_dir,
            "coverage",
            "coverage",
            "coverage",
            Risk::Safe,
            Method::Delete,
            None,
            project_id,
            project_name,
            last_activity,
            stale,
            root,
            calc,
            artifacts_out,
            items_out,
            total_bytes,
            total_items,
            cancel_flag,
        );
    }

    // 10. vendor/ (PHP Composer)
    if has_composer_json {
        check_simple_dir_artifact(
            project_dir,
            "vendor",
            "vendor_php",
            "vendor",
            Risk::Caution,
            Method::Trash,
            Some("PHP Vendor".to_string()),
            project_id,
            project_name,
            last_activity,
            stale,
            root,
            calc,
            artifacts_out,
            items_out,
            total_bytes,
            total_items,
            cancel_flag,
        );
    }

    // 11. .venv/ or venv/ (Python)
    if has_pyproject {
        for venv_name in &[".venv", "venv"] {
            check_simple_dir_artifact(
                project_dir,
                venv_name,
                "python_venv",
                venv_name,
                Risk::Caution,
                Method::Trash,
                Some("Python Venv".to_string()),
                project_id,
                project_name,
                last_activity,
                stale,
                root,
                calc,
                artifacts_out,
                items_out,
                total_bytes,
                total_items,
                cancel_flag,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn check_simple_dir_artifact(
    project_dir: &Path,
    sub_dir_name: &str,
    category_id: &str,
    artifact_name: &str,
    risk: Risk,
    method: Method,
    note: Option<String>,
    project_id: &str,
    project_name: &str,
    last_activity: u64,
    stale: bool,
    root: &Path,
    calc: &mut DiskSizeCalculator,
    artifacts_out: &mut Vec<ProjectArtifact>,
    items_out: &mut Vec<ScannedItemInternal>,
    total_bytes: &mut u64,
    total_items: &mut usize,
    cancel_flag: &AtomicBool,
) {
    let target = project_dir.join(sub_dir_name);
    if target.is_dir() {
        let bytes = calc.calculate_path_size(&target, Some(cancel_flag));
        if bytes > 0 {
            add_artifact(
                category_id,
                artifact_name,
                &target,
                bytes,
                risk,
                method,
                note,
                project_id,
                project_name,
                last_activity,
                stale,
                root,
                artifacts_out,
                items_out,
                total_bytes,
                total_items,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn add_artifact(
    category_id: &str,
    artifact_name: &str,
    path: &Path,
    bytes: u64,
    risk: Risk,
    method: Method,
    note: Option<String>,
    project_id: &str,
    project_name: &str,
    last_activity: u64,
    stale: bool,
    root: &Path,
    artifacts_out: &mut Vec<ProjectArtifact>,
    items_out: &mut Vec<ScannedItemInternal>,
    total_bytes: &mut u64,
    total_items: &mut usize,
) {
    // Validate with safety module before adding
    if is_path_allowed(path, &[root.to_path_buf()]).is_err() {
        return;
    }

    let item_id = format!("{}:{}:{:x}", project_id, category_id, fxhash(path));

    let artifact = ProjectArtifact {
        id: item_id.clone(),
        category_id: category_id.to_string(),
        name: artifact_name.to_string(),
        path: path.to_string_lossy().to_string(),
        bytes,
        risk,
        method,
        note: note.clone(),
    };

    let scan_item = ScanItem {
        id: item_id,
        category_id: category_id.to_string(),
        path: path.to_string_lossy().to_string(),
        bytes,
        project_name: Some(project_name.to_string()),
        last_activity: Some(last_activity),
        stale: Some(stale),
        note,
    };

    *total_bytes += bytes;
    *total_items += 1;

    artifacts_out.push(artifact);
    items_out.push(ScannedItemInternal {
        item: scan_item,
        real_path: path.to_path_buf(),
        allowed_root: Some(root.to_path_buf()),
    });
}

fn determine_project_type(
    node: bool,
    rust: bool,
    php: bool,
    python: bool,
    go: bool,
) -> String {
    let mut types = Vec::new();
    if node { types.push("Node"); }
    if rust { types.push("Rust"); }
    if python { types.push("Python"); }
    if go { types.push("Go"); }
    if php { types.push("PHP"); }

    if types.is_empty() {
        "Other".to_string()
    } else {
        types.join("/")
    }
}

/// Calculate the most recent mtime of top-level project files and .git/index
pub fn calculate_last_activity(project_dir: &Path) -> u64 {
    let mut latest_ms: u64 = 0;

    // Check .git/index or .git/HEAD
    let git_index = project_dir.join(".git/index");
    if let Ok(meta) = fs::metadata(&git_index) {
        if let Ok(mtime) = meta.modified() {
            if let Ok(dur) = mtime.duration_since(UNIX_EPOCH) {
                latest_ms = latest_ms.max(dur.as_millis() as u64);
            }
        }
    }

    let entries = match fs::read_dir(project_dir) {
        Ok(e) => e,
        Err(_) => return latest_ms,
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip artifact dirs in root mtime check
        if name == "node_modules"
            || name == "target"
            || name == "dist"
            || name == "build"
            || name == ".next"
            || name == ".git"
            || name == ".turbo"
            || name == ".venv"
            || name == "venv"
            || name == "vendor"
        {
            continue;
        }

        if let Ok(meta) = entry.metadata() {
            if let Ok(mtime) = meta.modified() {
                if let Ok(dur) = mtime.duration_since(UNIX_EPOCH) {
                    latest_ms = latest_ms.max(dur.as_millis() as u64);
                }
            }
        }
    }

    latest_ms
}

/// Parse `.gitignore` file in project root to check for build/coverage rules
pub fn parse_project_gitignore(project_dir: &Path) -> HashSet<String> {
    let mut patterns = HashSet::new();
    let gitignore_path = project_dir.join(".gitignore");

    if let Ok(content) = fs::read_to_string(&gitignore_path) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let cleaned = trimmed
                .trim_start_matches('/')
                .trim_end_matches('/')
                .trim_end_matches("/*");

            patterns.insert(cleaned.to_string());
        }
    }

    patterns
}

fn fxhash(path: &Path) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_project_detection_and_artifacts() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path().join("my-node-app");
        fs::create_dir_all(&project_dir).unwrap();

        // Create package.json
        fs::write(project_dir.join("package.json"), "{}").unwrap();

        // Create node_modules and a file inside
        let nm = project_dir.join("node_modules");
        fs::create_dir(&nm).unwrap();
        fs::write(nm.join("pkg.js"), "module.exports = {};").unwrap();

        // Create .gitignore with dist
        fs::write(project_dir.join(".gitignore"), "dist/\ncoverage\n").unwrap();

        // Create dist folder
        let dist = project_dir.join("dist");
        fs::create_dir(&dist).unwrap();
        fs::write(dist.join("bundle.js"), "console.log(1);").unwrap();

        let patterns = parse_project_gitignore(&project_dir);
        assert!(patterns.contains("dist"));
        assert!(patterns.contains("coverage"));

        let mut calc = DiskSizeCalculator::new();
        let mut artifacts = Vec::new();
        let mut items = Vec::new();
        let mut total_bytes = 0;
        let mut total_items = 0;
        let cancel = AtomicBool::new(false);

        detect_project_artifacts(
            &project_dir,
            "proj:123",
            "my-node-app",
            1000,
            true,
            true,
            false,
            false,
            false,
            &patterns,
            &mut calc,
            &mut artifacts,
            &mut items,
            &mut total_bytes,
            &mut total_items,
            temp_dir.path(),
            &cancel,
        );

        assert_eq!(artifacts.len(), 2, "Should find node_modules and dist");
        let nm_art = artifacts.iter().find(|a| a.name == "node_modules");
        assert!(nm_art.is_some());
        assert_eq!(nm_art.unwrap().risk, Risk::Safe); // stale = true -> safe
        let dist_art = artifacts.iter().find(|a| a.name == "dist");
        assert!(dist_art.is_some());
    }

    #[test]
    fn test_last_activity_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path().join("rust-app");
        fs::create_dir_all(&project_dir).unwrap();

        fs::write(project_dir.join("Cargo.toml"), "[package]\nname = \"rust-app\"\n").unwrap();
        fs::write(project_dir.join("README.md"), "# Rust App").unwrap();

        let last_activity = calculate_last_activity(&project_dir);
        assert!(last_activity > 0, "Last activity must be greater than 0");
    }
}
