// ============================================================================
// Heroku Config
// ============================================================================

use crate::core::converter::{ConvertOptions, Converter};
use anyhow::Result;
use std::collections::HashMap;

pub struct HerokuConfigConverter {
    pub app_name: Option<String>,
}

impl Default for HerokuConfigConverter {
    fn default() -> Self {
        Self { app_name: None }
    }
}

impl Converter for HerokuConfigConverter {
    fn convert(&self, vars: &HashMap<String, String>, options: &ConvertOptions) -> Result<String> {
        let filtered = options.filter_vars(vars);

        let mut output = String::new();
        output.push_str("#!/bin/bash\n");
        output.push_str("# Set Heroku config vars\n\n");

        let app_flag = self
            .app_name
            .as_ref()
            .map(|app| format!("--app {} ", app))
            .unwrap_or_default();

        for (k, v) in filtered.iter() {
            let key = options.transform_key(k);
            let value = options.transform_value(v);

            output.push_str(&format!(
                "heroku config:set {}{}='{}'\n",
                app_flag, key, value
            ));
        }

        Ok(output)
    }

    fn name(&self) -> &str {
        "heroku"
    }

    fn description(&self) -> &str {
        "Heroku Config Vars"
    }
}
