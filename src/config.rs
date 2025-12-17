use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::manifest::ProjectMode;

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

    /// Shared infrastructure configuration
    #[serde(default)]
    pub infrastructure: InfrastructureConfig,

    /// Default mode for new projects
    #[serde(default)]
    pub default_mode: ProjectMode,

    /// Docker network name prefix for project networks
    #[serde(default = "default_network_prefix")]
    pub network_prefix: String,

    /// Name of the shared proxy network
    #[serde(default = "default_shared_network")]
    pub shared_network: String,
}

/// Configuration for shared infrastructure services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfrastructureConfig {
    /// Enable shared PostgreSQL
    #[serde(default = "default_true")]
    pub postgres: bool,

    /// PostgreSQL image
    #[serde(default = "default_postgres_image")]
    pub postgres_image: String,

    /// Enable shared Redis
    #[serde(default)]
    pub redis: bool,

    /// Redis image
    #[serde(default = "default_redis_image")]
    pub redis_image: String,

    /// Enable shared MinIO
    #[serde(default)]
    pub minio: bool,

    /// MinIO image
    #[serde(default = "default_minio_image")]
    pub minio_image: String,

    /// Directory for infrastructure docker-compose and data
    #[serde(default = "default_infra_dir")]
    pub dir: PathBuf,
}

fn default_true() -> bool {
    true
}

fn default_postgres_image() -> String {
    "postgres:16-alpine".to_string()
}

fn default_redis_image() -> String {
    "redis:7-alpine".to_string()
}

fn default_minio_image() -> String {
    "minio/minio:latest".to_string()
}

fn default_infra_dir() -> PathBuf {
    let home = directories::BaseDirs::new()
        .map(|d| d.home_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("~"));
    home.join(".devhub/infrastructure")
}

fn default_network_prefix() -> String {
    "devhub".to_string()
}

fn default_shared_network() -> String {
    "shared_proxy".to_string()
}

impl Default for InfrastructureConfig {
    fn default() -> Self {
        InfrastructureConfig {
            postgres: true,
            postgres_image: default_postgres_image(),
            redis: false,
            redis_image: default_redis_image(),
            minio: false,
            minio_image: default_minio_image(),
            dir: default_infra_dir(),
        }
    }
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
            infrastructure: InfrastructureConfig::default(),
            default_mode: ProjectMode::default(),
            network_prefix: default_network_prefix(),
            shared_network: default_shared_network(),
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
    #[allow(dead_code)]
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
