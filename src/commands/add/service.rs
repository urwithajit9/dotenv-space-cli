use anyhow::{Context, Result};
use colored::*;
use dialoguer::Confirm;
use std::fs;
use std::path::Path;

use crate::schema::{loader, resolver, formatter};

/// Handle `evnx add service <service_id>`
pub fn handle(service_id: &str, path: &str, yes: bool, verbose: bool) -> Result<()> {
    // 1. Find service in schema
    let (_, service) = loader::find_service(service_id)
        .context(format!("Unknown service: '{}'. Run 'evnx add service --help' for options.", service_id))?;

    // 2. Resolve to variables
    let vars = resolver::resolve_service(service_id, service)
        .context("Failed to resolve service variables")?;

    // 3. Show preview
    println!("\n{}", "📋 Preview:".bold());
    println!("{}", formatter::generate_preview(&vars).dimmed());

    // 4. Confirm (unless --yes)
    if !yes {
        let confirm = Confirm::new()
            .with_prompt(format!("Add these {} variables to .env.example?", vars.vars.len()))
            .default(true)
            .interact()?;

        if !confirm {
            println!("{}", "Aborted.".yellow());
            return Ok(());
        }
    }

    // 5. Format as addition
    let addition = formatter::format_addition(&vars)?;

    // 6. Append to .env.example
    let example_path = Path::new(path).join(".env.example");

    if example_path.exists() {
        let existing = fs::read_to_string(&example_path)
            .context("Failed to read existing .env.example")?;

        let updated = format!("{}\n{}", existing.trim_end(), addition);
        fs::write(&example_path, updated)
            .context("Failed to write updated .env.example")?;

        println!("{} Appended {} variables to .env.example", "✓".green(), vars.vars.len());
    } else {
        // Create new file if doesn't exist
        fs::write(&example_path, addition.trim())
            .context("Failed to create .env.example")?;
        println!("{} Created .env.example with {} variables", "✓".green(), vars.vars.len());
    }

    // 7. Also update .env if it exists
    let env_path = Path::new(path).join(".env");
    if env_path.exists() {
        let existing = fs::read_to_string(&env_path)?;
        // Add TODO comment for new vars
        let todo_addition = addition.lines()
            .map(|line| {
                if line.starts_with('#') || line.trim().is_empty() {
                    line.to_string()
                } else {
                    format!("# TODO: {}  # <-- Fill in real value", line)
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        let updated = format!("{}\n\n{}", existing.trim_end(), todo_addition);
        fs::write(&env_path, updated)?;
        println!("{} Updated .env with TODO placeholders", "✓".green());
    }

    Ok(())
}