// ============================================================================
// Doppler Format
// ============================================================================

use crate::core::converter::{ConvertOptions, Converter};
use anyhow::Result;
use std::collections::HashMap;

pub struct DopplerConverter;

impl Converter for DopplerConverter {
    fn convert(&self, vars: &HashMap<String, String>, options: &ConvertOptions) -> Result<String> {
        let filtered = options.filter_vars(vars);

        let transformed: HashMap<String, serde_json::Value> = filtered
            .iter()
            .map(|(k, v)| {
                let key = options.transform_key(k);
                let value = options.transform_value(v);
                (
                    key,
                    serde_json::json!({
                        "value": value,
                        "computed": false
                    }),
                )
            })
            .collect();

        let json = serde_json::to_string_pretty(&transformed)?;
        Ok(json)
    }

    fn name(&self) -> &str {
        "doppler"
    }

    fn description(&self) -> &str {
        "Doppler secrets JSON format"
    }
}
