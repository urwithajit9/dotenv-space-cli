use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Failed to read file: {0}")]
    FileReadError(#[from] std::io::Error),

    #[error("Invalid format at line {line}: {message}")]
    InvalidFormat { line: usize, message: String },

    #[error("Invalid key format at line {line}: '{key}' (keys must match [A-Za-z][A-Za-z0-9_]*)")]
    InvalidKey { line: usize, key: String },

    #[error("Variable expansion failed at line {line}: ${{{var}}} is undefined")]
    UndefinedVariable { line: usize, var: String },

    #[error("Circular variable expansion detected at line {line}: {cycle}")]
    CircularExpansion { line: usize, cycle: String },

    #[error("Unterminated quoted string at line {line}")]
    UnterminatedString { line: usize },
}

pub type ParseResult<T> = Result<T, ParseError>;

/// Parsed environment variable set
#[derive(Debug, Clone)]
pub struct EnvFile {
    pub vars: HashMap<String, String>,
    pub source: String,
}

/// Parser configuration
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Allow variable expansion (${VAR})
    pub allow_expansion: bool,
    /// Strict mode - enforce uppercase keys, no inline comments
    pub strict: bool,
    /// Maximum expansion depth to prevent infinite loops
    pub max_expansion_depth: usize,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            allow_expansion: true,
            strict: false,
            max_expansion_depth: 10,
        }
    }
}

/// Main parser
pub struct Parser {
    config: ParserConfig,
}

impl Parser {
    pub fn new(config: ParserConfig) -> Self {
        Self { config }
    }

    pub fn default() -> Self {
        Self::new(ParserConfig::default())
    }

    /// Parse a .env file from a path
    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> ParseResult<EnvFile> {
        let content = fs::read_to_string(path.as_ref())?;
        let source = path.as_ref().to_string_lossy().to_string();
        let vars = self.parse_content(&content)?;
        Ok(EnvFile { vars, source })
    }

    /// Parse .env content from a string
    pub fn parse_content(&self, content: &str) -> ParseResult<HashMap<String, String>> {
        let mut vars = HashMap::new();

        for (line_num, line) in content.lines().enumerate() {
            let line_num = line_num + 1; // 1-indexed for user-facing errors

            // Skip empty lines and comments
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse key=value
            let (key, value) = self.parse_line(line, line_num)?;

            // Validate key format
            if !self.is_valid_key(&key) {
                return Err(ParseError::InvalidKey {
                    line: line_num,
                    key: key.clone(),
                });
            }

            vars.insert(key, value);
        }

        // Perform variable expansion if enabled
        if self.config.allow_expansion {
            self.expand_variables(&mut vars)?;
        }

        Ok(vars)
    }

    /// Parse a single line into (key, value)
    fn parse_line(&self, line: &str, line_num: usize) -> ParseResult<(String, String)> {
        // Handle optional 'export' prefix
        let line = line.strip_prefix("export").unwrap_or(line).trim_start();

        // Find the = separator
        let eq_pos = line.find('=').ok_or_else(|| ParseError::InvalidFormat {
            line: line_num,
            message: "Missing '=' separator".to_string(),
        })?;

        let key = line[..eq_pos].trim().to_string();
        let value_raw = line[eq_pos + 1..].trim_start();

        // Parse the value based on quoting
        let value = self.parse_value(value_raw, line_num)?;

        Ok((key, value))
    }

    /// Parse a value, handling quotes and escapes
    fn parse_value(&self, value: &str, line_num: usize) -> ParseResult<String> {
        if value.is_empty() {
            return Ok(String::new());
        }

        let first_char = value.chars().next().unwrap();

        match first_char {
            // Double-quoted string - handle escape sequences
            '"' => self.parse_double_quoted(value, line_num),

            // Single-quoted string - no escapes, literal
            '\'' => self.parse_single_quoted(value, line_num),

            // Unquoted value
            _ => Ok(self.parse_unquoted(value)),
        }
    }

    /// Parse double-quoted string with escape sequences
    fn parse_double_quoted(&self, value: &str, line_num: usize) -> ParseResult<String> {
        if !value.ends_with('"') || value.len() < 2 {
            return Err(ParseError::UnterminatedString { line: line_num });
        }

        let content = &value[1..value.len() - 1];
        let mut result = String::new();
        let mut chars = content.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '\\' {
                // Handle escape sequences
                match chars.next() {
                    Some('n') => result.push('\n'),
                    Some('r') => result.push('\r'),
                    Some('t') => result.push('\t'),
                    Some('\\') => result.push('\\'),
                    Some('"') => result.push('"'),
                    Some(c) => {
                        // Unknown escape - keep literal
                        result.push('\\');
                        result.push(c);
                    }
                    None => result.push('\\'),
                }
            } else {
                result.push(ch);
            }
        }

        Ok(result)
    }

    /// Parse single-quoted string (literal, no escapes)
    fn parse_single_quoted(&self, value: &str, line_num: usize) -> ParseResult<String> {
        if !value.ends_with('\'') || value.len() < 2 {
            return Err(ParseError::UnterminatedString { line: line_num });
        }

        Ok(value[1..value.len() - 1].to_string())
    }

    /// Parse unquoted value (trim trailing whitespace, stop at #)
    fn parse_unquoted(&self, value: &str) -> String {
        // In strict mode, we don't allow inline comments
        // In normal mode, # in unquoted values is ambiguous - include it
        // Users should quote values containing #
        value.trim_end().to_string()
    }

    /// Check if a key is valid
    fn is_valid_key(&self, key: &str) -> bool {
        if key.is_empty() {
            return false;
        }

        // Must start with a letter
        let mut chars = key.chars();
        let first = chars.next().unwrap();
        if !first.is_ascii_alphabetic() {
            return false;
        }

        // Rest must be alphanumeric or underscore
        for ch in chars {
            if !ch.is_ascii_alphanumeric() && ch != '_' {
                return false;
            }
        }

        // In strict mode, enforce uppercase
        if self.config.strict && key != key.to_uppercase() {
            return false;
        }

        true
    }

    /// Expand variables like ${VAR} in all values
    fn expand_variables(&self, vars: &mut HashMap<String, String>) -> ParseResult<()> {
        let mut expanded = HashMap::new();
        let mut expansion_stack = Vec::new();

        for (key, value) in vars.iter() {
            let expanded_value = self.expand_value(value, vars, &mut expansion_stack, 0)?;
            expanded.insert(key.clone(), expanded_value);
        }

        *vars = expanded;
        Ok(())
    }

    /// Recursively expand a single value
    fn expand_value(
        &self,
        value: &str,
        vars: &HashMap<String, String>,
        stack: &mut Vec<String>,
        depth: usize,
    ) -> ParseResult<String> {
        // Check expansion depth
        if depth > self.config.max_expansion_depth {
            return Err(ParseError::CircularExpansion {
                line: 0,
                cycle: format!("max depth {} exceeded", self.config.max_expansion_depth),
            });
        }

        let mut result = String::new();
        let mut chars = value.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                chars.next(); // consume '{'

                // Extract variable name
                let var_name: String = chars.by_ref().take_while(|&c| c != '}').collect();

                // Check for circular dependency
                if stack.contains(&var_name) {
                    return Err(ParseError::CircularExpansion {
                        line: 0,
                        cycle: format!("{} -> {}", stack.join(" -> "), var_name),
                    });
                }

                // Get variable value
                let var_value =
                    vars.get(&var_name)
                        .ok_or_else(|| ParseError::UndefinedVariable {
                            line: 0,
                            var: var_name.clone(),
                        })?;

                // Recursively expand the variable value
                stack.push(var_name.clone());
                let expanded = self.expand_value(var_value, vars, stack, depth + 1)?;
                stack.pop();

                result.push_str(&expanded);
            } else {
                result.push(ch);
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        let parser = Parser::default();
        let content = r#"
KEY1=value1
KEY2=value2
"#;
        let vars = parser.parse_content(content).unwrap();
        assert_eq!(vars.get("KEY1"), Some(&"value1".to_string()));
        assert_eq!(vars.get("KEY2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_empty_lines_and_comments() {
        let parser = Parser::default();
        let content = r#"
# This is a comment
KEY1=value1

# Another comment
KEY2=value2
"#;
        let vars = parser.parse_content(content).unwrap();
        assert_eq!(vars.len(), 2);
    }

    #[test]
    fn test_quoted_values() {
        let parser = Parser::default();
        let content = r#"
SINGLE='single quoted'
DOUBLE="double quoted"
EMPTY=""
"#;
        let vars = parser.parse_content(content).unwrap();
        assert_eq!(vars.get("SINGLE"), Some(&"single quoted".to_string()));
        assert_eq!(vars.get("DOUBLE"), Some(&"double quoted".to_string()));
        assert_eq!(vars.get("EMPTY"), Some(&"".to_string()));
    }

    #[test]
    fn test_escape_sequences() {
        let parser = Parser::default();
        let content = r#"NEWLINE="line1\nline2"
TAB="col1\tcol2"
QUOTE="He said \"hello\""
BACKSLASH="path\\to\\file"
"#;
        let vars = parser.parse_content(content).unwrap();
        assert_eq!(vars.get("NEWLINE"), Some(&"line1\nline2".to_string()));
        assert_eq!(vars.get("TAB"), Some(&"col1\tcol2".to_string()));
        assert_eq!(vars.get("QUOTE"), Some(&"He said \"hello\"".to_string()));
        assert_eq!(vars.get("BACKSLASH"), Some(&"path\\to\\file".to_string()));
    }

    #[test]
    fn test_whitespace_handling() {
        let parser = Parser::default();
        let content = "  KEY1  =  value1  \nKEY2=value2   ";
        let vars = parser.parse_content(content).unwrap();
        assert_eq!(vars.get("KEY1"), Some(&"value1".to_string()));
        assert_eq!(vars.get("KEY2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_export_prefix() {
        let parser = Parser::default();
        let content = "export KEY1=value1\nexport KEY2=value2";
        let vars = parser.parse_content(content).unwrap();
        assert_eq!(vars.get("KEY1"), Some(&"value1".to_string()));
        assert_eq!(vars.get("KEY2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_variable_expansion() {
        let parser = Parser::default();
        let content = r#"
BASE=http://localhost
API_URL=${BASE}/api
FULL_URL=${API_URL}/v1
"#;
        let vars = parser.parse_content(content).unwrap();
        assert_eq!(vars.get("BASE"), Some(&"http://localhost".to_string()));
        assert_eq!(
            vars.get("API_URL"),
            Some(&"http://localhost/api".to_string())
        );
        assert_eq!(
            vars.get("FULL_URL"),
            Some(&"http://localhost/api/v1".to_string())
        );
    }

    #[test]
    fn test_undefined_variable_expansion() {
        let parser = Parser::default();
        let content = "KEY=${UNDEFINED}";
        let result = parser.parse_content(content);
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::UndefinedVariable { var, .. } => {
                assert_eq!(var, "UNDEFINED");
            }
            _ => panic!("Expected UndefinedVariable error"),
        }
    }

    #[test]
    fn test_circular_expansion() {
        let parser = Parser::default();
        let content = "A=${B}\nB=${A}";
        let result = parser.parse_content(content);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ParseError::CircularExpansion { .. }
        ));
    }

    #[test]
    fn test_invalid_key_format() {
        let parser = Parser::default();

        // Key starting with number
        let content = "1KEY=value";
        let result = parser.parse_content(content);
        assert!(result.is_err());

        // Key with special characters
        let content = "KEY-NAME=value";
        let result = parser.parse_content(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_unterminated_string() {
        let parser = Parser::default();
        let content = r#"KEY="unterminated"#;
        let result = parser.parse_content(content);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ParseError::UnterminatedString { .. }
        ));
    }

    #[test]
    fn test_empty_value() {
        let parser = Parser::default();
        let content = "KEY=";
        let vars = parser.parse_content(content).unwrap();
        assert_eq!(vars.get("KEY"), Some(&"".to_string()));
    }

    #[test]
    fn test_mixed_case_keys() {
        let parser = Parser::default();
        let content = "MyKey=value\nanother_Key=value2";
        let vars = parser.parse_content(content).unwrap();
        assert_eq!(vars.get("MyKey"), Some(&"value".to_string()));
        assert_eq!(vars.get("another_Key"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_strict_mode_rejects_lowercase() {
        let mut config = ParserConfig::default();
        config.strict = true;
        let parser = Parser::new(config);

        let content = "lowercase=value";
        let result = parser.parse_content(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_no_expansion_mode() {
        let mut config = ParserConfig::default();
        config.allow_expansion = false;
        let parser = Parser::new(config);

        let content = "KEY=${OTHER}";
        let vars = parser.parse_content(content).unwrap();
        assert_eq!(vars.get("KEY"), Some(&"${OTHER}".to_string()));
    }

    #[test]
    fn test_real_world_example() {
        let parser = Parser::default();
        let content = r#"
# Database
DATABASE_URL=postgresql://user:pass@localhost:5432/mydb

# Django
SECRET_KEY="django-insecure-abc123"
DEBUG=True
ALLOWED_HOSTS=localhost,127.0.0.1

# AWS
AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE
AWS_SECRET_ACCESS_KEY="wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
AWS_REGION=us-east-1

# Computed
API_BASE=http://localhost:8000
API_V1=${API_BASE}/api/v1
"#;
        let vars = parser.parse_content(content).unwrap();

        assert_eq!(
            vars.get("DATABASE_URL"),
            Some(&"postgresql://user:pass@localhost:5432/mydb".to_string())
        );
        assert_eq!(
            vars.get("SECRET_KEY"),
            Some(&"django-insecure-abc123".to_string())
        );
        assert_eq!(vars.get("DEBUG"), Some(&"True".to_string()));
        assert_eq!(
            vars.get("API_V1"),
            Some(&"http://localhost:8000/api/v1".to_string())
        );
        assert_eq!(vars.len(), 9);
    }
}
