use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_service")]
    pub service: String,
    #[serde(default)]
    pub default_handle: Option<String>,
    #[serde(default)]
    pub prefer_app_password: bool,
}

fn default_service() -> String {
    "https://bsky.social".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            service: default_service(),
            default_handle: None,
            prefer_app_password: false,
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let path = config_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let config: AppConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(AppConfig::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }
}

fn config_path() -> PathBuf {
    dirs_path().join("config.toml")
}

fn dirs_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config").join("skyscraper")
}
