use anyhow::{Context, Result};
use colored::*;
use std::collections::HashSet;

use crate::core::{Parser, ParserConfig};

/// Validate .env against .env.example
pub fn run(
    env: String,
    example: String,
    strict: bool,
    fix: bool,
    format: String,
    exit_zero: bool,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("{}", "Running validate in verbose mode".dimmed());
    }

    println!("\n{}", "┌─ Validating environment variables ──────────────────┐".cyan());
    println!("{}", "│ Comparing .env against .env.example                 │".cyan());
    println!("{}\n", "└──────────────────────────────────────────────────────┘".cyan());

    // Parse both files
    let mut parser_config = ParserConfig::default();
    if strict {
        parser_config.strict = true;
    }
    let parser = Parser::new(parser_config);

    // Parse .env.example first (this is the source of truth)
    let example_file = match parser.parse_file(&example) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("{} Failed to parse {}: {}", "✗".red(), example, e);
            if !exit_zero {
                std::process::exit(1);
            }
            return Ok(());
        }
    };

    // Parse .env
    let env_file = match parser.parse_file(&env) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("{} Failed to parse {}: {}", "✗".red(), env, e);
            if !exit_zero {
                std::process::exit(1);
            }
            return Ok(());
        }
    };

    if verbose {
        println!("Parsed {} variables from {}", example_file.vars.len(), example);
        println!("Parsed {} variables from {}", env_file.vars.len(), env);
    }

    // Validation checks
    let mut issues = Vec::new();
    let mut errors = 0;
    let mut warnings = 0;

    // Check 1: Are all required variables present?
    let example_keys: HashSet<_> = example_file.vars.keys().collect();
    let env_keys: HashSet<_> = env_file.vars.keys().collect();
    
    let missing: Vec<_> = example_keys.difference(&env_keys).collect();
    let extra: Vec<_> = env_keys.difference(&example_keys).collect();

    if missing.is_empty() {
        println!("{} All required variables present ({}/{})", 
            "✓".green(), 
            env_file.vars.len(), 
            example_file.vars.len()
        );
    } else {
        println!("{} Missing {} required variables", "✗".red(), missing.len());
        for key in &missing {
            issues.push(format!("  🚨 Missing required variable: {}", key.bold()));
            errors += 1;
        }
    }

    // Check 2: Detect placeholder values
    for (key, value) in &env_file.vars {
        if is_placeholder(value) {
            issues.push(format!("  🚨 {} looks like a placeholder", key.bold()));
            issues.push(format!("     → Value: {}", value.dimmed()));
            errors += 1;
        }
    }

    // Check 3: Boolean string trap
    for (key, value) in &env_file.vars {
        if value == "False" || value == "True" {
            issues.push(format!("  ⚠️  {} is set to \"{}\" (string)", key.bold(), value));
            issues.push(format!("     → This is truthy in Python — did you mean {} or 0?", 
                if value == "False" { "False" } else { "True" }));
            warnings += 1;
        }
    }

    // Check 4: localhost in Docker context
    if std::path::Path::new("docker-compose.yml").exists() 
        || std::path::Path::new("docker-compose.yaml").exists() {
        for (key, value) in &env_file.vars {
            if value.contains("localhost") && (key.contains("URL") || key.contains("HOST")) {
                issues.push(format!("  ⚠️  {} uses localhost", key.bold()));
                issues.push(format!("     → In Docker, use service name instead (e.g., db:5432)"));
                warnings += 1;
            }
        }
    }

    // Print issues
    if !issues.is_empty() {
        println!("{} Found {} issues\n", "✗".red(), errors + warnings);
        println!("Issues:");
        for issue in issues {
            println!("{}", issue);
        }
    }

    // Summary
    println!("\n{}", "Summary:".bold());
    if errors > 0 {
        println!("  {} {} critical issues (placeholder values)", "🚨".to_string(), errors);
    }
    if warnings > 0 {
        println!("  {} {} warnings", "⚠️".to_string(), warnings);
    }
    if errors == 0 && warnings == 0 {
        println!("  {} 0 issues found", "✓".green());
    }

    if fix && errors > 0 {
        println!("\nRun 'dotenv-space validate --fix' to auto-fix safe issues");
    }

    // Exit code
    if !exit_zero && errors > 0 {
        std::process::exit(1);
    }

    Ok(())
}

/// Check if a value looks like a placeholder
fn is_placeholder(value: &str) -> bool {
    let lower = value.to_lowercase();
    
    // Common placeholder patterns
    let placeholders = [
        "your_key_here",
        "your_secret_here",
        "your_token_here",
        "change_me",
        "changeme",
        "replace_me",
        "example",
        "xxx",
        "todo",
        "generate-with",
    ];

    placeholders.iter().any(|p| lower.contains(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_placeholder() {
        assert!(is_placeholder("YOUR_KEY_HERE"));
        assert!(is_placeholder("sk_test_CHANGE_ME"));
        assert!(is_placeholder("generate-with-openssl"));
        assert!(!is_placeholder("sk_test_abc123def456"));
        assert!(!is_placeholder("postgresql://localhost:5432/db"));
    }
}