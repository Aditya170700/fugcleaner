use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod linux;
pub mod macos;
pub mod windows;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Risk {
    Safe,
    Caution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Method {
    Trash,
    Delete,
    Command,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CategoryGroup {
    Global,
    Project,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: String,
    pub name: String,
    pub description: String,
    pub group: CategoryGroup,
    pub method: Method,
    pub risk: Risk,
    pub available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unavailable_reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CategoryDef {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub group: CategoryGroup,
    pub method: Method,
    pub risk: Risk,
    pub paths_patterns: &'static [&'static str],
    pub command: Option<&'static str>,
    pub cli_name: Option<&'static str>,
    pub split_subfolders: bool,
}

/// Expand `~` in path pattern to user's home directory
pub fn expand_tilde(pattern: &str) -> Option<PathBuf> {
    if pattern.starts_with("~/") {
        dirs::home_dir().map(|home| home.join(&pattern[2..]))
    } else if pattern == "~" {
        dirs::home_dir()
    } else {
        Some(PathBuf::from(pattern))
    }
}

/// Get all defined category definitions for the current operating system
pub fn get_os_categories() -> Vec<CategoryDef> {
    #[cfg(target_os = "macos")]
    {
        macos::get_macos_categories()
    }
    #[cfg(target_os = "linux")]
    {
        linux::get_linux_categories()
    }
    #[cfg(target_os = "windows")]
    {
        windows::get_windows_categories()
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Vec::new()
    }
}

/// Resolve CategoryDefs to frontend Category objects with `available` and `unavailableReason` evaluated
pub fn list_available_categories() -> Vec<Category> {
    let defs = get_os_categories();
    let mut result = Vec::new();

    for def in defs {
        let (available, unavailable_reason) = check_category_availability(&def);

        result.push(Category {
            id: def.id.to_string(),
            name: def.name.to_string(),
            description: def.description.to_string(),
            group: def.group,
            method: def.method,
            risk: def.risk,
            available,
            unavailable_reason,
        });
    }

    result
}

fn check_category_availability(def: &CategoryDef) -> (bool, Option<String>) {
    // 1. If category relies on a CLI tool (e.g. docker, brew, pnpm, xcrun)
    if let Some(cli) = def.cli_name {
        if which::which(cli).is_err() {
            return (
                false,
                Some(format!("Tool CLI '{}' tidak ditemukan di sistem.", cli)),
            );
        }

        // Special check for docker daemon if CLI exists
        if cli == "docker" {
            let docker_check = std::process::Command::new("docker")
                .arg("info")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();

            if !docker_check.map(|s| s.success()).unwrap_or(false) {
                return (false, Some("Docker daemon tidak sedang berjalan.".to_string()));
            }
        }
    }

    // 2. If category has path patterns, check if at least one exists
    if !def.paths_patterns.is_empty() {
        let mut any_path_exists = false;
        for &pattern in def.paths_patterns {
            if let Some(path) = expand_tilde(pattern) {
                if path.exists() {
                    any_path_exists = true;
                    break;
                }
            }
        }

        if !any_path_exists && def.command.is_none() {
            return (
                false,
                Some("Direktori cache belum dibuat atau kosong.".to_string()),
            );
        }
    }

    (true, None)
}
