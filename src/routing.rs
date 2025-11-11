use std::{collections::HashMap, fs};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServiceConfig {
    pub host: String,
    pub port: u16,
    pub prefix: String,
    pub api_version: u16,
    pub strip_prefix: bool,
    pub skip_prefix_for: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RoutingConfig {
    pub services: HashMap<String, ServiceConfig>,
}

impl RoutingConfig {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: RoutingConfig = serde_json::from_str(&content)?;

        Ok(config)
    }

    pub fn find_service(&self, path: &str) -> Option<(&String, &ServiceConfig)> {
        self.services
            .iter()
            .find(|(_, config)| path.starts_with(&config.prefix))
    }
}
