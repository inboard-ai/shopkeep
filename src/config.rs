use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Bind address (default: "0.0.0.0")
    #[serde(default = "default_bind")]
    pub bind: String,
    /// Port number (default: 8080)
    #[serde(default = "default_port")]
    pub port: u16,
    /// Registry configuration
    pub registry: RegistryConfig,
}

fn default_bind() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8080
}

/// Registry backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RegistryConfig {
    Filesystem { path: PathBuf },
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bind: default_bind(),
            port: default_port(),
            registry: RegistryConfig::Filesystem {
                path: PathBuf::from("./registry"),
            },
        }
    }
}

impl Config {
    /// Load configuration from file, environment, and CLI arguments
    pub fn load(
        config_path: Option<&PathBuf>,
        cli_bind: Option<&str>,
        cli_port: Option<u16>,
        cli_registry_path: Option<&PathBuf>,
    ) -> anyhow::Result<Self> {
        // Start with default config
        let mut config = if let Some(path) = config_path {
            let content = std::fs::read_to_string(path)?;
            toml::from_str(&content)?
        } else {
            // Try default config file
            if let Ok(content) = std::fs::read_to_string("shopkeep.toml") {
                toml::from_str(&content)?
            } else {
                Config::default()
            }
        };

        // Override with environment variables
        if let Ok(bind) = std::env::var("SHOPKEEP_BIND") {
            config.bind = bind;
        }
        if let Ok(port) = std::env::var("SHOPKEEP_PORT") {
            if let Ok(p) = port.parse() {
                config.port = p;
            }
        }
        if let Ok(path) = std::env::var("SHOPKEEP_REGISTRY_PATH") {
            config.registry = RegistryConfig::Filesystem {
                path: PathBuf::from(path),
            };
        }

        // Override with CLI arguments
        if let Some(bind) = cli_bind {
            config.bind = bind.to_string();
        }
        if let Some(port) = cli_port {
            config.port = port;
        }
        if let Some(path) = cli_registry_path {
            config.registry = RegistryConfig::Filesystem {
                path: path.clone(),
            };
        }

        Ok(config)
    }
}
