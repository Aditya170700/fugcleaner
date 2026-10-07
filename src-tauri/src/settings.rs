use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub project_roots: Vec<String>,
    pub stale_threshold_days: u32,
    pub enabled_categories: Vec<String>,
    pub exclude_paths: Vec<String>,
    pub max_scan_depth: usize,
    pub dry_run: bool,
}

impl Default for Settings {
    fn default() -> Self {
        // By default on macOS, suggest ~/Projects if it exists
        let mut default_roots = Vec::new();
        if let Some(home) = dirs::home_dir() {
            let projects_dir = home.join("Projects");
            if projects_dir.exists() {
                default_roots.push(projects_dir.to_string_lossy().to_string());
            }
        }

        Self {
            project_roots: default_roots,
            stale_threshold_days: 30,
            enabled_categories: Vec::new(),
            exclude_paths: Vec::new(),
            max_scan_depth: 6,
            dry_run: cfg!(debug_assertions),
        }
    }
}

pub fn get_settings_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("fugcleaner/settings.json"))
}

pub fn load_settings() -> Settings {
    if let Some(path) = get_settings_path() {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(settings) = serde_json::from_str::<Settings>(&content) {
                    return settings;
                }
            }
        }
    }
    Settings::default()
}

pub fn save_settings_to_disk(settings: &Settings) -> AppResult<()> {
    if let Some(path) = get_settings_path() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                AppError::Settings(format!("Gagal membuat direktori config: {}", e))
            })?;
        }

        let json = serde_json::to_string_pretty(settings)
            .map_err(|e| AppError::Settings(format!("Gagal serialisasi settings: {}", e)))?;

        fs::write(&path, json)
            .map_err(|e| AppError::Settings(format!("Gagal menulis settings.json: {}", e)))?;

        Ok(())
    } else {
        Err(AppError::Settings("Config directory tidak ditemukan di sistem.".to_string()))
    }
}
