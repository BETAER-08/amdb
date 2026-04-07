use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(default = "default_db_path")]
    pub db_path: String,
    #[serde(default = "default_ignore_patterns")]
    pub ignore_patterns: Vec<String>,
}

fn default_db_path() -> String {
    ".database".to_string()
}

fn default_ignore_patterns() -> Vec<String> {
    vec![
        "target".to_string(),
        ".git".to_string(),
        "node_modules".to_string(),
        ".amdb".to_string(),
        ".fastembed_cache".to_string(),
        "__pycache__".to_string(),
        ".database".to_string(),
    ]
}

impl Config {
    pub fn load(root: &str) -> Self {
        let mut config = Config::default();
        let config_path = Path::new(root).join("amdb.toml");

        if let Ok(content) = fs::read_to_string(config_path) {
            if let Ok(parsed) = toml::from_str::<Config>(&content) {
                config = parsed;
            }
        }

        if let Ok(env_db_path) = env::var("AMDB_DB_PATH") {
            config.db_path = env_db_path;
        }

        config
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            db_path: default_db_path(),
            ignore_patterns: default_ignore_patterns(),
        }
    }
}