// ============================================================================
// String utilities
// ============================================================================

pub mod string {
    /// Truncate string to max length
    pub fn truncate(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len - 3])
        }
    }

    /// Redact sensitive values
    pub fn redact(s: &str) -> String {
        if s.len() <= 8 {
            "*".repeat(s.len())
        } else {
            format!("{}...{}", &s[..4], "*".repeat(4))
        }
    }

    /// Pluralize word based on count
    pub fn pluralize(count: usize, singular: &str, plural: &str) -> String {
        if count == 1 {
            format!("{} {}", count, singular)
        } else {
            format!("{} {}", count, plural)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_truncate() {
            assert_eq!(truncate("hello", 10), "hello");
            assert_eq!(truncate("hello world", 8), "hello...");
        }

        #[test]
        fn test_redact() {
            assert_eq!(redact("secret"), "******");
            assert_eq!(redact("secretkey123"), "secr...****");
        }

        #[test]
        fn test_pluralize() {
            assert_eq!(pluralize(1, "file", "files"), "1 file");
            assert_eq!(pluralize(2, "file", "files"), "2 files");
        }
    }
}
