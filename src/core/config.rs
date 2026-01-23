use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub server_port: u16,
    pub exclude_patterns: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server_port: 3000,
            exclude_patterns: vec![
                "target".to_string(),
                ".git".to_string(),
                "node_modules".to_string(),
                ".amdb".to_string(),
                ".fastembed_cache".to_string(),
                "__pycache__".to_string(),
            ],
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        let config_path = Path::new("amdb.toml");
        if config_path.exists() {
            if let Ok(content) = fs::read_to_string(config_path) {
                if let Ok(config) = toml::from_str(&content) {
                    return config;
                }
            }
        }
        Self::default()
    }
}