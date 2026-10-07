use serde::{Deserialize, Serialize};

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
        Self {
            project_roots: Vec::new(),
            stale_threshold_days: 30,
            enabled_categories: Vec::new(),
            exclude_paths: Vec::new(),
            max_scan_depth: 6,
            dry_run: cfg!(debug_assertions),
        }
    }
}
