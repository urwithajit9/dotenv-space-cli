// ============================================================================
// File system utilities
// ============================================================================

pub mod fs {
    use anyhow::Result;
    use std::fs;
    use std::path::Path;

    /// Check if file has secure permissions (600 on Unix)
    #[cfg(unix)]
    pub fn has_secure_permissions(path: &Path) -> bool {
        use std::os::unix::fs::PermissionsExt;

        if let Ok(metadata) = fs::metadata(path) {
            let mode = metadata.permissions().mode();
            (mode & 0o077) == 0 // Only owner has permissions
        } else {
            false
        }
    }

    #[cfg(not(unix))]
    pub fn has_secure_permissions(_path: &Path) -> bool {
        true // Not applicable on non-Unix systems
    }

    /// Set secure permissions (600 on Unix)
    #[cfg(unix)]
    pub fn set_secure_permissions(path: &Path) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(path, perms)?;
        Ok(())
    }

    #[cfg(not(unix))]
    pub fn set_secure_permissions(_path: &Path) -> Result<()> {
        Ok(()) // Not applicable on non-Unix systems
    }

    /// Backup a file before modifying it
    pub fn backup_file(path: &Path) -> Result<()> {
        let backup_path = path.with_extension("backup");
        fs::copy(path, backup_path)?;
        Ok(())
    }

    /// Check if file is text (not binary)
    pub fn is_text_file(path: &Path) -> Result<bool> {
        let content = fs::read(path)?;

        // Check for null bytes (common in binary files)
        Ok(!content.iter().any(|&b| b == 0))
    }

    /// Get file size in human-readable format
    pub fn human_readable_size(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

        let mut size = bytes as f64;
        let mut unit_idx = 0;

        while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
            size /= 1024.0;
            unit_idx += 1;
        }

        format!("{:.2} {}", size, UNITS[unit_idx])
    }

    /// Find files matching a pattern
    pub fn find_files(pattern: &str) -> Result<Vec<String>> {
        use walkdir::WalkDir;

        let mut files = Vec::new();

        for entry in WalkDir::new(".")
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                let path_str = path.to_string_lossy();
                if path_str.contains(pattern) {
                    files.push(path_str.to_string());
                }
            }
        }

        Ok(files)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::io::Write;
        use tempfile::NamedTempFile;

        #[test]
        fn test_human_readable_size() {
            assert_eq!(human_readable_size(500), "500.00 B");
            assert_eq!(human_readable_size(1024), "1.00 KB");
            assert_eq!(human_readable_size(1024 * 1024), "1.00 MB");
        }

        #[test]
        fn test_is_text_file() {
            let mut file = NamedTempFile::new().unwrap();
            file.write_all(b"Hello, world!").unwrap();
            assert!(is_text_file(file.path()).unwrap());
        }

        #[test]
        fn test_backup_file() {
            let mut file = NamedTempFile::new().unwrap();
            file.write_all(b"test content").unwrap();

            backup_file(file.path()).unwrap();

            let backup_path = file.path().with_extension("backup");
            assert!(backup_path.exists());
        }
    }
}
