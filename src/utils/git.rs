// ============================================================================
// Git utilities
// ============================================================================

pub mod git {
    use anyhow::{anyhow, Result};
    use std::process::Command;

    /// Check if .env is in .gitignore
    pub fn is_in_gitignore(file: &str) -> Result<bool> {
        let gitignore = std::fs::read_to_string(".gitignore")?;
        Ok(gitignore.lines().any(|line| {
            let trimmed = line.trim();
            trimmed == file || trimmed.starts_with(file)
        }))
    }

    /// Add file to .gitignore
    pub fn add_to_gitignore(file: &str) -> Result<()> {
        use std::io::Write;

        let mut gitignore = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(".gitignore")?;

        writeln!(gitignore, "\n# Environment variables")?;
        writeln!(gitignore, "{}", file)?;

        Ok(())
    }

    /// Check if file is tracked by Git
    pub fn is_tracked(file: &str) -> bool {
        Command::new("git")
            .args(&["ls-files", "--error-unmatch", file])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Scan Git history for a pattern
    pub fn scan_history(pattern: &str) -> Result<Vec<String>> {
        let output = Command::new("git")
            .args(&["log", "-S", pattern, "--all", "--pretty=format:%H %s"])
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("Git log failed"));
        }

        let stdout = String::from_utf8(output.stdout)?;
        Ok(stdout.lines().map(|s| s.to_string()).collect())
    }

    /// Get current branch
    pub fn current_branch() -> Result<String> {
        let output = Command::new("git")
            .args(&["branch", "--show-current"])
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("Not a git repository"));
        }

        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    /// Check if working directory is clean
    pub fn is_clean() -> bool {
        Command::new("git")
            .args(&["status", "--porcelain"])
            .output()
            .map(|o| o.stdout.is_empty())
            .unwrap_or(false)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_is_in_gitignore() {
            // This test would need a test repository
        }
    }
}
