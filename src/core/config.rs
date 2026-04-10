use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub backend: BackendConfig,
    pub coding: CodingConfig,
    pub platform: PlatformConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openrouter_api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openrouter_base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openrouter_model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    pub coordinator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodingConfig {
    pub agent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    #[serde(rename = "type")]
    pub platform_type: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            backend: BackendConfig {
                coordinator: "ironclaw".to_string(),
            },
            coding: CodingConfig {
                agent: "opencode".to_string(),
            },
            platform: PlatformConfig {
                platform_type: "omarchy".to_string(),
            },
            openrouter_api_key: None,
            openrouter_base_url: None,
            openrouter_model: None,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("octopod/config.toml");

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    pub fn load_global() -> Result<Self> {
        Self::load()
    }
}
