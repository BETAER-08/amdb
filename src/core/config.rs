use serde::Deserialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Config {
    pub db_path: PathBuf,
    pub ignore_patterns: Vec<String>,
}

#[derive(Deserialize)]
struct TomlConfig {
    db_path: Option<String>,
    ignore_patterns: Option<Vec<String>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            db_path: PathBuf::from(".database"),
            ignore_patterns: vec![
                "target".to_string(),
                ".git".to_string(),
                "node_modules".to_string(),
                ".amdb".to_string(),
                ".fastembed_cache".to_string(),
                "__pycache__".to_string(),
                ".database".to_string(),
            ],
        }
    }
}

impl Config {
    pub fn load(root: &str) -> Self {
        let mut config = Self::default();
        let config_path = Path::new(root).join("amdb.toml");

        if let Ok(content) = fs::read_to_string(config_path) {
            if let Ok(toml_config) = toml::from_str::<TomlConfig>(&content) {
                if let Some(dp) = toml_config.db_path {
                    config.db_path = PathBuf::from(dp);
                }
                if let Some(ip) = toml_config.ignore_patterns {
                    config.ignore_patterns = ip;
                }
            }
        }

        if let Ok(env_db_path) = env::var("AMDB_DB_PATH") {
            config.db_path = PathBuf::from(env_db_path);
        }

        config
    }
}