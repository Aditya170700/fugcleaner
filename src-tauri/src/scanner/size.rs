use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

/// Calculator for accurate disk usage, with hardlink deduplication and symlink protection
#[derive(Debug, Default)]
pub struct DiskSizeCalculator {
    #[cfg(unix)]
    visited_inodes: HashSet<(u64, u64)>, // (dev, ino)
}

impl DiskSizeCalculator {
    pub fn new() -> Self {
        Self {
            #[cfg(unix)]
            visited_inodes: HashSet::new(),
        }
    }

    /// Clear visited inodes (e.g. for a new scan)
    pub fn clear(&mut self) {
        #[cfg(unix)]
        self.visited_inodes.clear();
    }

    /// Calculate the physical disk size of a single file or directory.
    /// Deduplicates hardlinks so shared inodes are only counted once.
    /// Does NOT follow symlinks.
    pub fn calculate_path_size(&mut self, path: &Path, cancel_flag: Option<&AtomicBool>) -> u64 {
        if let Some(cancel) = cancel_flag {
            if cancel.load(Ordering::Relaxed) {
                return 0;
            }
        }

        let meta = match std::fs::symlink_metadata(path) {
            Ok(m) => m,
            Err(_) => return 0,
        };

        // NEVER follow symlinks
        if meta.file_type().is_symlink() {
            return 0;
        }

        if meta.is_file() {
            return self.get_file_size(&meta);
        }

        if meta.is_dir() {
            return self.calculate_dir_size(path, cancel_flag);
        }

        0
    }

    fn get_file_size(&mut self, meta: &std::fs::Metadata) -> u64 {
        #[cfg(unix)]
        {
            let dev = meta.dev();
            let ino = meta.ino();
            let nlink = meta.nlink();

            // If file has multiple links, deduplicate by (dev, ino)
            if nlink > 1 && !self.visited_inodes.insert((dev, ino)) {
                // Already counted this inode
                return 0;
            }

            // On Unix, metadata.blocks() returns 512-byte blocks allocated on disk
            meta.blocks() * 512
        }

        #[cfg(not(unix))]
        {
            meta.len()
        }
    }

    fn calculate_dir_size(&mut self, dir_path: &Path, cancel_flag: Option<&AtomicBool>) -> u64 {
        let mut total_bytes: u64 = 0;

        // Use jwalk or standard read_dir for single directory tree calculation
        // WalkDir with follow_links(false)
        let walker = jwalk::WalkDir::new(dir_path)
            .follow_links(false)
            .skip_hidden(false);

        for entry in walker {
            if let Some(cancel) = cancel_flag {
                if cancel.load(Ordering::Relaxed) {
                    break;
                }
            }

            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let entry_path = entry.path();
            let meta = match std::fs::symlink_metadata(&entry_path) {
                Ok(m) => m,
                Err(_) => continue,
            };

            if meta.file_type().is_symlink() {
                continue;
            }

            if meta.is_file() {
                total_bytes += self.get_file_size(&meta);
            }
        }

        total_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_file_and_hardlink_deduplication() {
        let temp_dir = TempDir::new().unwrap();
        let file1_path = temp_dir.path().join("file1.txt");

        // Write 4096 bytes
        {
            let mut f = std::fs::File::create(&file1_path).unwrap();
            f.write_all(&vec![0u8; 4096]).unwrap();
        }

        let mut calc = DiskSizeCalculator::new();
        let size1 = calc.calculate_path_size(&file1_path, None);
        assert!(size1 >= 4096, "File size should be at least 4096 bytes");

        #[cfg(unix)]
        {
            let hardlink_path = temp_dir.path().join("file1_hardlink.txt");
            std::fs::hard_link(&file1_path, &hardlink_path).unwrap();

            // When scanning directory containing both hardlinks, total size should equal single file size
            let mut dir_calc = DiskSizeCalculator::new();
            let dir_size = dir_calc.calculate_path_size(temp_dir.path(), None);
            assert_eq!(
                dir_size, size1,
                "Directory with hardlinked file must count file size only once"
            );
        }
    }

    #[test]
    fn test_symlinks_not_counted() {
        let temp_dir = TempDir::new().unwrap();
        let real_file = temp_dir.path().join("real.txt");
        {
            let mut f = std::fs::File::create(&real_file).unwrap();
            f.write_all(&vec![0u8; 8192]).unwrap();
        }

        let link_file = temp_dir.path().join("link.txt");
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&real_file, &link_file).unwrap();
            let mut calc = DiskSizeCalculator::new();
            let symlink_size = calc.calculate_path_size(&link_file, None);
            assert_eq!(symlink_size, 0, "Symlink target size must not be counted");
        }
    }
}
