use anyhow::{Context, Result};
use colored::*;
use dialoguer::{Confirm, MultiSelect, Select};
use std::path::Path;

use crate::schema::{loader, resolver,formatter};
use super::shared::write_env_files;

/// Handle Architect mode: step-by-step custom stack building
pub fn handle(path: String, yes: bool, verbose: bool) -> Result<()> {
    // Step 1: Select Language
    let languages: Vec<_> = loader::schema()?
        .languages
        .keys()
        .map(|k| k.as_str())
        .collect();

    let lang_display: Vec<String> = languages
        .iter()
        .map(|id| format_language_display(id))
        .collect();

    let lang_idx = if yes {
        0
    } else {
        Select::new()
            .with_prompt("Select your primary language:")
            .items(&lang_display)
            .default(0)
            .interact()?
    };
    let language_id = languages[lang_idx];

    // Step 2: Select Framework (filtered by language)
    let frameworks = loader::get_frameworks_for_language(language_id)
        .context(format!("No frameworks for language: {}", language_id))?;

    let fw_display: Vec<String> = frameworks
        .iter()
        .map(|(_, name)| name.to_string())
        .collect();

    let fw_idx = if yes {
        0
    } else {
        Select::new()
            .with_prompt("Select your framework:")
            .items(&fw_display)
            .default(0)
            .interact()?
    };
    let framework_id = frameworks[fw_idx].0;

    // Step 3: Select Services (MultiSelect, grouped)
    let service_groups = loader::get_services_grouped();
    let (service_items, service_ids): (Vec<String>, Vec<String>) = service_groups
        .iter()
        .flat_map(|(group, services)| {
            services.iter().map(move |(id, name)| {
                (format!("[{}] {}", group, name), id.to_string())
            })
        })
        .unzip();

    let selected_svc_indices = if yes {
        // Default: select first service from first group
        vec![0]
    } else {
        MultiSelect::new()
            .with_prompt("Select services you'll use (Space to toggle, Enter to confirm):")
            .items(&service_items)
            .interact()?
    };

    let selected_services: Vec<String> = selected_svc_indices
        .iter()
        .filter_map(|&idx| service_ids.get(idx).cloned())
        .collect();

    // Step 4: Select Infrastructure (optional, MultiSelect)
    let infra_items: Vec<_> = loader::schema()?
        .infrastructure
        .keys()
        .map(|k| (k.as_str(), format!("[Infra] {}", k)))
        .collect();

    let infra_display: Vec<String> = infra_items.iter().map(|(_, d)| d.clone()).collect();
    let infra_ids: Vec<String> = infra_items.iter().map(|(id, _)| id.to_string()).collect();

    let selected_infra_indices = if yes {
        vec![] // Default: no infra in non-interactive
    } else {
        MultiSelect::new()
            .with_prompt("Select deployment/infrastructure (optional):")
            .items(&infra_display)
            .interact()?
    };

    let selected_infra: Vec<String> = selected_infra_indices
        .iter()
        .filter_map(|&idx| infra_ids.get(idx).cloned())
        .collect();

    if verbose {
        println!("{}", "[DEBUG] Selection summary:".dimmed());
        println!("  Language: {}", language_id);
        println!("  Framework: {}", framework_id);
        println!("  Services: {:?}", selected_services);
        println!("  Infrastructure: {:?}", selected_infra);
    }

    // Resolve variables
    let vars = resolver::resolve_architect_selection(
        language_id,
        framework_id,
        &selected_services,
        &selected_infra,
    )?;

    // Show preview
    println!("\n{}", "📋 Preview:".bold());
    println!("{}", formatter::generate_preview(&vars).dimmed());

    if !yes {
        let confirm = Confirm::new()
            .with_prompt("Generate .env files with these variables?")
            .default(true)
            .interact()?;

        if !confirm {
            println!("{}", "Aborted.".yellow());
            return Ok(());
        }
    }

    // Format and write
    let example_content = formatter::format_env_example(&vars, true)?;
    let template_content = formatter::format_env_template(&vars)?;

    let output_path = Path::new(&path);
    write_env_files(output_path, &example_content, &template_content)?;

    println!(
        "{} Created .env.example with {} variables",
        "✓".green(),
        vars.vars.len()
    );

    Ok(())
}

/// Handle architect mode with pre-filled defaults (for legacy flag compatibility)
pub fn handle_with_defaults(
    stack: Option<String>,
    services: Option<String>,
    path: String,
    yes: bool,
    verbose: bool,
) -> Result<()> {
    // Map legacy stack names to language/framework
    let (language_id, framework_id) = match stack.as_deref().unwrap_or("python") {
        "python" => ("python", "django"),
        "nodejs" => ("javascript_typescript", "express_fastify"),
        "rust" => ("rust", "axum_actix"),
        "go" => ("go", "gin_echo"),
        "php" => ("php", "laravel"),
        _ => ("python", "django"),
    };

    // Parse services
    let selected_services: Vec<String> = services
        .as_ref()
        .map(|s| s.split(',').map(|x| x.trim().to_string()).collect())
        .unwrap_or_default();

    // Resolve and generate (same as interactive flow)
    let vars = resolver::resolve_architect_selection(
        language_id,
        framework_id,
        &selected_services,
        &[], // No infra in legacy mode
    )?;

    let example_content = formatter::format_env_example(&vars, true)?;
    let template_content = formatter::format_env_template(&vars)?;

    let output_path = Path::new(&path);
    super::shared::write_env_files(output_path, &example_content, &template_content)?;

    println!(
        "{} Created .env.example with {} variables",
        "✓".green(),
        vars.vars.len()
    );

    Ok(())
}

fn format_language_display(id: &str) -> String {
    match id {
        "javascript_typescript" => "JavaScript / TypeScript".to_string(),
        "java_kotlin" => "Java / Kotlin".to_string(),
        other => {
            // Capitalize first letter
            let mut chars = other.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        }
    }
}