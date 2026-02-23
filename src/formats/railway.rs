// ============================================================================
// Railway Variables
// ============================================================================

use crate::core::converter::{ConvertOptions, Converter};
use anyhow::Result;
use std::collections::HashMap;

pub struct RailwayConverter;

impl Converter for RailwayConverter {
    fn convert(&self, vars: &HashMap<String, String>, options: &ConvertOptions) -> Result<String> {
        let filtered = options.filter_vars(vars);

        let transformed: HashMap<String, String> = filtered
            .iter()
            .map(|(k, v)| {
                let key = options.transform_key(k);
                let value = options.transform_value(v);
                (key, value)
            })
            .collect();

        let json = serde_json::to_string_pretty(&transformed)?;
        Ok(json)
    }

    fn name(&self) -> &str {
        "railway"
    }

    fn description(&self) -> &str {
        "Railway JSON format"
    }
}
