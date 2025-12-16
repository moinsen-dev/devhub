use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Global DevHub configuration stored at ~/.devhub/config.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Path to the Caddy sites.d directory
    #[serde(default = "default_caddy_sites_dir")]
    pub caddy_sites_dir: PathBuf,

    /// Docker exec command to reload Caddy
    #[serde(default = "default_caddy_reload_command")]
    pub caddy_reload_command: String,

    /// Default process manager to use
    #[serde(default)]
    pub process_manager: ProcessManager,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ProcessManager {
    /// Native process spawning (tokio::process)
    #[default]
    Native,
    /// Use PM2 for process management
    Pm2,
    /// Use Overmind for process management
    Overmind,
}

fn default_caddy_sites_dir() -> PathBuf {
    // Try to find the reverse-proxy directory
    let home = directories::BaseDirs::new()
        .map(|d| d.home_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("~"));

    home.join("docker/reverse-proxy/sites.d")
}

fn default_caddy_reload_command() -> String {
    "docker exec caddy-proxy caddy reload --config /etc/caddy/Caddyfile".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Config {
            caddy_sites_dir: default_caddy_sites_dir(),
            caddy_reload_command: default_caddy_reload_command(),
            process_manager: ProcessManager::default(),
        }
    }
}

impl Config {
    /// Load config from default location (~/.devhub/config.toml)
    pub fn load() -> Result<Self> {
        let config_path = get_config_path()?;

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    /// Save config to disk
    pub fn save(&self) -> Result<()> {
        let config_path = get_config_path()?;

        // Ensure directory exists
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;
        Ok(())
    }
}

fn get_config_path() -> Result<PathBuf> {
    let base_dirs = directories::BaseDirs::new()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

    Ok(base_dirs.home_dir().join(".devhub/config.toml"))
}
