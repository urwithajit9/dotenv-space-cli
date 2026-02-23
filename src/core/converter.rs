use anyhow::Result;
/// Core converter - transform .env to different formats
///
/// Provides a trait-based system for converting environment variables
/// to various output formats (JSON, YAML, shell scripts, etc.)
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Converter trait - implement this for each output format
pub trait Converter {
    /// Convert variables to the target format
    fn convert(&self, vars: &HashMap<String, String>, options: &ConvertOptions) -> Result<String>;

    /// Format name
    fn name(&self) -> &str;

    /// Format description
    fn description(&self) -> &str;
}

/// Conversion options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertOptions {
    /// Include only variables matching this pattern (glob)
    pub include_pattern: Option<String>,

    /// Exclude variables matching this pattern (glob)
    pub exclude_pattern: Option<String>,

    /// Base64-encode all values
    pub base64: bool,

    /// Add prefix to all keys
    pub prefix: Option<String>,

    /// Transform key names
    pub transform: Option<KeyTransform>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyTransform {
    Uppercase,
    Lowercase,
    CamelCase,
    SnakeCase,
}

impl Default for ConvertOptions {
    fn default() -> Self {
        Self {
            include_pattern: None,
            exclude_pattern: None,
            base64: false,
            prefix: None,
            transform: None,
        }
    }
}

impl ConvertOptions {
    /// Filter variables based on include/exclude patterns
    pub fn filter_vars(&self, vars: &HashMap<String, String>) -> HashMap<String, String> {
        let mut filtered = vars.clone();

        // Apply include pattern
        if let Some(pattern) = &self.include_pattern {
            filtered.retain(|k, _| matches_pattern(k, pattern));
        }

        // Apply exclude pattern
        if let Some(pattern) = &self.exclude_pattern {
            filtered.retain(|k, _| !matches_pattern(k, pattern));
        }

        filtered
    }

    /// Transform a key according to the transformation rule
    pub fn transform_key(&self, key: &str) -> String {
        let key = if let Some(prefix) = &self.prefix {
            format!("{}{}", prefix, key)
        } else {
            key.to_string()
        };

        match &self.transform {
            Some(KeyTransform::Uppercase) => key.to_uppercase(),
            Some(KeyTransform::Lowercase) => key.to_lowercase(),
            Some(KeyTransform::CamelCase) => to_camel_case(&key),
            Some(KeyTransform::SnakeCase) => to_snake_case(&key),
            None => key,
        }
    }

    /// Transform a value (apply base64 encoding if enabled)
    pub fn transform_value(&self, value: &str) -> String {
        if self.base64 {
            general_purpose::STANDARD.encode(value)
        } else {
            value.to_string()
        }
    }
}

/// Simple glob pattern matching
fn matches_pattern(key: &str, pattern: &str) -> bool {
    // Handle wildcards
    if pattern.contains('*') {
        // Split pattern by *
        let parts: Vec<&str> = pattern.split('*').collect();

        if parts.len() == 2 {
            // Pattern like "AWS_*"
            if parts[0].is_empty() {
                key.ends_with(parts[1])
            } else if parts[1].is_empty() {
                key.starts_with(parts[0])
            } else {
                key.starts_with(parts[0]) && key.ends_with(parts[1])
            }
        } else {
            // Multiple wildcards - simple check
            parts.iter().all(|p| p.is_empty() || key.contains(p))
        }
    } else {
        // Exact match or comma-separated list
        pattern.split(',').any(|p| key == p.trim())
    }
}

/// Convert to camelCase
fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for (i, ch) in s.chars().enumerate() {
        if ch == '_' {
            capitalize_next = true;
        } else if capitalize_next || (i == 0) {
            result.push(if i == 0 {
                ch.to_lowercase().next().unwrap()
            } else {
                ch.to_uppercase().next().unwrap()
            });
            capitalize_next = false;
        } else {
            result.push(ch.to_lowercase().next().unwrap());
        }
    }

    result
}

/// Convert to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();

    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
            result.push(ch.to_lowercase().next().unwrap());
        } else {
            result.push(ch.to_lowercase().next().unwrap());
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_pattern() {
        assert!(matches_pattern("AWS_ACCESS_KEY", "AWS_*"));
        assert!(matches_pattern("DATABASE_URL", "*_URL"));
        assert!(matches_pattern("SECRET_KEY", "SECRET_KEY"));
        assert!(!matches_pattern("OTHER_KEY", "AWS_*"));

        // Comma-separated
        assert!(matches_pattern("KEY1", "KEY1,KEY2,KEY3"));
        assert!(matches_pattern("KEY2", "KEY1,KEY2,KEY3"));
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("DATABASE_URL"), "databaseUrl");
        assert_eq!(to_camel_case("SECRET_KEY"), "secretKey");
        assert_eq!(to_camel_case("AWS_ACCESS_KEY_ID"), "awsAccessKeyId");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("DatabaseURL"), "database_url");
        assert_eq!(to_snake_case("SecretKey"), "secret_key");
        assert_eq!(to_snake_case("AWSAccessKeyID"), "a_w_s_access_key_i_d");
    }

    #[test]
    fn test_filter_vars() {
        let mut vars = HashMap::new();
        vars.insert("AWS_KEY".to_string(), "val1".to_string());
        vars.insert("DB_URL".to_string(), "val2".to_string());
        vars.insert("AWS_SECRET".to_string(), "val3".to_string());

        let mut options = ConvertOptions::default();
        options.include_pattern = Some("AWS_*".to_string());

        let filtered = options.filter_vars(&vars);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.contains_key("AWS_KEY"));
        assert!(filtered.contains_key("AWS_SECRET"));
    }

    #[test]
    fn test_transform_key() {
        let mut options = ConvertOptions::default();
        options.prefix = Some("APP_".to_string());
        options.transform = Some(KeyTransform::Lowercase);

        assert_eq!(options.transform_key("DATABASE_URL"), "app_database_url");
    }
}
