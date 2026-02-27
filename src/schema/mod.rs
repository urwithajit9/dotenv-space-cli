/// Schema module: reusable infrastructure for all commands.

pub mod models;
pub mod loader;
pub mod resolver;
pub mod formatter;
pub mod query;

// Re-export commonly used types
pub use models::{VarCollection, VarMetadata, VarSource, Schema};
pub use loader::{schema, find_service, find_framework, list_blueprints, get_frameworks_for_language, get_services_grouped};
pub use resolver::{resolve_blueprint, resolve_service, resolve_framework, resolve_architect_selection};
pub use formatter::{format_env_example, format_env_template, format_addition, generate_preview};
pub use query::{search_services, search_frameworks, filter_by_tag, list_tags};