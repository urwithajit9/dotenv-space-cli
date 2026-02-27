use anyhow::Result;
use colored::*;
use std::path::Path;

use super::shared::write_env_files;

/// Handle Blank mode: create minimal .env files
pub fn handle(path: String, yes: bool, _verbose: bool) -> Result<()> {
    if !yes {
        println!("{}", "Creating minimal .env files...".dimmed());
    }

    let example_content = "# Add your environment variables here\n# Format: KEY=value\n\n";
    let template_content = "# TODO: Replace with real values\n# Format: KEY=value\n\n";

    let output_path = Path::new(&path);
    write_env_files(output_path, example_content, template_content)?;

    println!("{} Created empty .env.example", "✓".green());
    println!(
        "{} Tip: Run 'evnx add' to add variables interactively",
        "💡".yellow()
    );

    Ok(())
}