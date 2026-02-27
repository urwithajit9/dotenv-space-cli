//! Interactive project setup — generates .env.example.
//!
//! Implements breadth-first selection:
//! 1. Choose mode: Blank / Blueprint / Architect
//! 2. If Blueprint: select pre-combined stack
//! 3. If Architect: step through language → framework → services → infra
//! 4. Generate .env.example and .env with deduplicated, categorized variables

use anyhow::Result;
use colored::*;
use dialoguer::Select;
// use std::path::Path;

mod blank;
mod blueprint;
mod architect;
mod shared;

pub use shared::write_env_files;

/// Main entry point for `evnx init`
pub fn run(path: String, yes: bool, verbose: bool) -> Result<()> {
    if verbose {
        println!("{}", "Running init in verbose mode".dimmed());
    }

    print_header();

    // Step 1: Select mode
    let mode = if yes {
        // Non-interactive: default to Blueprint for best UX
        Mode::Blueprint
    } else {
        let modes = [
            "📄 Blank (create empty .env files)",
            "🔷 Blueprint (use pre-configured stack)",
            "🏗️  Architect (build custom stack)",
        ];

        let selection = Select::new()
            .with_prompt("How do you want to start?")
            .items(&modes)
            .default(1)
            .interact()?;

        match selection {
            0 => Mode::Blank,
            1 => Mode::Blueprint,
            2 => Mode::Architect,
            _ => Mode::Blank,
        }
    };

    // Step 2: Route to handler
    match mode {
        Mode::Blank => blank::handle(path, yes, verbose)?,
        Mode::Blueprint => blueprint::handle(path, yes, verbose)?,
        Mode::Architect => architect::handle(path, yes, verbose)?,
    }

    print_next_steps();
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Mode {
    Blank,
    Blueprint,
    Architect,
}

fn print_header() {
    println!(
        "\n{}",
        "┌─ evnx init ─────────────────────────────────────┐".cyan()
    );
    println!(
        "{}",
        "│ Set up environment variables for your project  │".cyan()
    );
    println!(
        "{}\n",
        "└────────────────────────────────────────────────┘".cyan()
    );
}

fn print_next_steps() {
    println!("\n{}", "Next steps:".bold());
    println!("  1. Edit .env and replace placeholder values");
    println!("  2. Never commit .env to version control");
    println!("  3. Run 'evnx validate' to check configuration");
    println!("  4. Use 'evnx add' to add more variables later");
}