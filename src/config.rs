use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry {
    pub name: String,
    pub url: String,
    pub priority: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub install_dir: String,
    pub auto_update: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub registries: Vec<Registry>,
    pub default_registry: String,
    pub cache_ttl: u64,
    pub settings: Settings,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            registries: vec![Registry {
                name: "default".to_string(),
                url: "github.com/Grip-Packages/Grip-Packages".to_string(),
                priority: 100,
            }],
            default_registry: "github.com/Grip-Packages/Grip-Packages".to_string(),
            cache_ttl: 3600,
            settings: Settings::default(),
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            install_dir: "$HOME/.local/bin".to_string(),
            auto_update: true,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to get config directory"))?
            .join("grip")
            .join("registries.json");

        if config_path.exists() {
            let content = std::fs::read_to_string(config_path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            let config = Config::default();
            std::fs::create_dir_all(config_path.parent().unwrap())?;
            std::fs::write(
                &config_path,
                serde_json::to_string_pretty(&config)?,
            )?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to get config directory"))?
            .join("grip")
            .join("registries.json");

        std::fs::write(
            &config_path,
            serde_json::to_string_pretty(&self)?,
        )?;

        Ok(())
    }
}