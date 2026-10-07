use std::path::{Path, PathBuf};
use crate::error::{AppError, AppResult};

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

/// Hard-deny list of system directories across OSes
pub const HARD_DENY_SYSTEM_UNIX: &[&str] = &[
    "/",
    "/System",
    "/Library",
    "/usr",
    "/bin",
    "/sbin",
    "/Applications",
    "/etc",
    "/var",
    "/dev",
    "/private",
    "/proc",
    "/sys",
    "/root",
    "/boot",
    "/opt",
];

pub const HARD_DENY_SYSTEM_WINDOWS: &[&str] = &[
    "C:\\",
    "C:\\Windows",
    "C:\\Program Files",
    "C:\\Program Files (x86)",
    "C:\\ProgramData",
    "C:\\System Volume Information",
];

/// Get user directories that should NEVER be deleted
pub fn get_user_deny_paths() -> Vec<PathBuf> {
    let mut deny_paths = Vec::new();

    if let Some(home) = dirs::home_dir() {
        deny_paths.push(home.clone());
        deny_paths.push(home.join("Desktop"));
        deny_paths.push(home.join("Documents"));
        deny_paths.push(home.join("Downloads"));
        deny_paths.push(home.join("Pictures"));
        deny_paths.push(home.join("Music"));
        deny_paths.push(home.join("Movies"));
    }

    deny_paths
}

/// Check if a path is in any hard-deny list (system or user personal directories)
pub fn is_hard_denied(path: &Path) -> bool {
    // Check system deny list
    #[cfg(unix)]
    {
        for &deny in HARD_DENY_SYSTEM_UNIX {
            if path == Path::new(deny) {
                return true;
            }
        }
    }

    #[cfg(windows)]
    {
        for &deny in HARD_DENY_SYSTEM_WINDOWS {
            if path == Path::new(deny) {
                return true;
            }
        }
    }

    // Check if path is .git or contains .git component
    for component in path.components() {
        if component.as_os_str() == ".git" {
            return true;
        }
    }

    // Check user protected directories
    for user_deny in get_user_deny_paths() {
        if let Ok(canonical_user) = user_deny.canonicalize() {
            if path == canonical_user {
                return true;
            }
        } else if path == user_deny {
            return true;
        }
    }

    // Protect home directory exact match
    if let Some(home) = dirs::home_dir() {
        if path == home {
            return true;
        }
    }

    false
}

/// Get the device ID (filesystem/mount volume) of a path
#[cfg(unix)]
pub fn get_device_id(path: &Path) -> Option<u64> {
    std::fs::metadata(path).ok().map(|m| m.dev())
}

#[cfg(not(unix))]
pub fn get_device_id(_path: &Path) -> Option<u64> {
    None
}

/// Check whether path_b is on the same filesystem device as path_a
pub fn is_same_filesystem(path_a: &Path, path_b: &Path) -> bool {
    #[cfg(unix)]
    {
        match (get_device_id(path_a), get_device_id(path_b)) {
            (Some(dev_a), Some(dev_b)) => dev_a == dev_b,
            _ => true,
        }
    }
    #[cfg(not(unix))]
    {
        true
    }
}

/// Validate that a path is safe and allowed to be scanned or cleaned.
///
/// Rules:
/// 1. Reject symlinks (must use symlink_metadata, not metadata)
/// 2. Canonicalize path to eliminate `..` and relative traversals
/// 3. Check against hard-deny list (System, home, Documents, .git, etc)
/// 4. If allowed_roots is specified, ensure canonical path is strictly inside one of the roots (cannot be the root itself)
pub fn is_path_allowed(target: &Path, allowed_roots: &[PathBuf]) -> AppResult<PathBuf> {
    // 1. Check if exists and is NOT a symlink
    let meta = std::fs::symlink_metadata(target).map_err(|e| {
        AppError::SafetyViolation(format!(
            "Path '{}' tidak dapat diakses atau tidak ada: {}",
            target.display(),
            e
        ))
    })?;

    if meta.file_type().is_symlink() {
        return Err(AppError::SafetyViolation(format!(
            "Target '{}' adalah symlink. Operasi terhadap symlink ditolak demi keamanan.",
            target.display()
        )));
    }

    // 2. Canonicalize to resolve any `..` or relative components
    let canonical = target.canonicalize().map_err(|e| {
        AppError::SafetyViolation(format!(
            "Gagal melakukan kanonikalisasi path '{}': {}",
            target.display(),
            e
        ))
    })?;

    // 3. Check hard-deny list
    if is_hard_denied(&canonical) {
        return Err(AppError::SafetyViolation(format!(
            "Path '{}' termasuk dalam hard-deny list keamanan sistem.",
            canonical.display()
        )));
    }

    // 4. Validate allowed roots (if provided)
    if !allowed_roots.is_empty() {
        let mut inside_allowed_root = false;

        for root in allowed_roots {
            if let Ok(canonical_root) = root.canonicalize() {
                // Must be inside the root, but CANNOT be the root folder itself
                if canonical.starts_with(&canonical_root) && canonical != canonical_root {
                    inside_allowed_root = true;
                    break;
                }
            } else if canonical.starts_with(root) && canonical != *root {
                inside_allowed_root = true;
                break;
            }
        }

        if !inside_allowed_root {
            return Err(AppError::SafetyViolation(format!(
                "Path '{}' berada di luar root folder yang diizinkan.",
                canonical.display()
            )));
        }
    }

    Ok(canonical)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_hard_deny_root() {
        assert!(is_hard_denied(Path::new("/")));
        #[cfg(target_os = "macos")]
        {
            assert!(is_hard_denied(Path::new("/System")));
            assert!(is_hard_denied(Path::new("/Library")));
            assert!(is_hard_denied(Path::new("/usr")));
            assert!(is_hard_denied(Path::new("/Applications")));
        }
    }

    #[test]
    fn test_hard_deny_user_home() {
        if let Some(home) = dirs::home_dir() {
            assert!(is_hard_denied(&home));
            assert!(is_hard_denied(&home.join("Documents")));
            assert!(is_hard_denied(&home.join("Desktop")));
            assert!(is_hard_denied(&home.join("Downloads")));
        }
    }

    #[test]
    fn test_hard_deny_git() {
        assert!(is_hard_denied(Path::new("/some/project/.git")));
        assert!(is_hard_denied(Path::new("/some/project/.git/objects")));
    }

    #[test]
    fn test_symlink_rejection() {
        let temp_dir = TempDir::new().unwrap();
        let target_dir = temp_dir.path().join("real_folder");
        std::fs::create_dir(&target_dir).unwrap();

        let link_path = temp_dir.path().join("symlink_folder");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&target_dir, &link_path).unwrap();

        #[cfg(unix)]
        {
            let res = is_path_allowed(&link_path, &[temp_dir.path().to_path_buf()]);
            assert!(res.is_err(), "Symlink must be rejected");
            let err_msg = res.err().unwrap().to_string();
            assert!(err_msg.contains("symlink"));
        }
    }

    #[test]
    fn test_path_traversal_detection() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path().join("project");
        let safe_child = project_dir.join("node_modules");
        std::fs::create_dir_all(&safe_child).unwrap();

        // Safe child should succeed
        let ok_res = is_path_allowed(&safe_child, &[project_dir.clone()]);
        assert!(ok_res.is_ok());

        // Root project folder itself must be rejected
        let root_res = is_path_allowed(&project_dir, &[project_dir.clone()]);
        assert!(root_res.is_err());

        // Traversing outside root using ../ must be rejected
        let traversal_path = safe_child.join("../../outside");
        let outside_dir = temp_dir.path().join("outside");
        std::fs::create_dir_all(&outside_dir).unwrap();

        let trav_res = is_path_allowed(&traversal_path, &[project_dir.clone()]);
        assert!(trav_res.is_err(), "Traversing outside root must fail");
    }
}
