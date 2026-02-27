//! evnx — Manage .env files with validation, secret scanning, and format conversion.
//!
//! # Library Usage
//!
//! ```rust
//! use evnx::schema::{loader, resolver, formatter};
//!
//! // Resolve variables for a service
//! let schema = loader::schema()?;
//! let pg = &schema.services.databases["postgresql"];
//! let vars = resolver::resolve_service("postgresql", pg)?;
//! let content = formatter::format_addition(&vars)?;
//! // Write `content` to .env.example
//! # Ok::<_, anyhow::Error>(())
//! ```
//!
//! # Architecture
//!
//! - `schema/` — Reusable core: JSON schema, resolver, formatter
//! - `commands/` — CLI handlers (init, add, validate, etc.)
//! - `utils/` — Shared utilities (file I/O, formatting)

// ─────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────



pub mod cli;
pub mod commands;
pub mod schema;
pub mod utils;
pub mod core;
pub mod formats;

pub use cli::{Cli, Commands, AddTarget};
pub use schema::{
    models::{VarCollection, VarMetadata, VarSource, Schema},
    loader::{schema as load_schema, find_service, find_framework, list_blueprints, get_frameworks_for_language, get_services_grouped},
    resolver::{resolve_blueprint, resolve_service, resolve_framework, resolve_architect_selection},
    formatter::{format_env_example, format_env_template, format_addition, generate_preview},
    query::{search_services, search_frameworks, filter_by_tag, list_tags},
};