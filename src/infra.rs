//! Shared infrastructure management for DevHub.
//!
//! This module manages shared infrastructure services (PostgreSQL, Redis, MinIO)
//! that can be used by multiple projects running in container mode.
//!
//! Note: This module is foundation code for CR-001 (Docker Network Isolation).
//! Functions are not yet wired into the main CLI but will be in a future release.

#![allow(dead_code)]

use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::HashMap;
use std::process::Command;

use crate::config::Config;
use crate::docker;
use crate::manifest::InfraService;

/// Status of an infrastructure service
#[derive(Debug, Clone)]
pub struct InfraServiceStatus {
    pub name: String,
    pub enabled: bool,
    pub running: bool,
    pub container_name: String,
    pub image: String,
}

/// Overall infrastructure status
#[derive(Debug, Clone)]
pub struct InfraStatus {
    pub shared_network_exists: bool,
    pub services: Vec<InfraServiceStatus>,
}

/// Get the status of all infrastructure services
pub async fn get_status(config: &Config) -> Result<InfraStatus> {
    let services = vec![
        InfraServiceStatus {
            name: "postgres".to_string(),
            enabled: config.infrastructure.postgres,
            running: docker::is_container_running("devhub-postgres").await,
            container_name: "devhub-postgres".to_string(),
            image: config.infrastructure.postgres_image.clone(),
        },
        InfraServiceStatus {
            name: "redis".to_string(),
            enabled: config.infrastructure.redis,
            running: docker::is_container_running("devhub-redis").await,
            container_name: "devhub-redis".to_string(),
            image: config.infrastructure.redis_image.clone(),
        },
        InfraServiceStatus {
            name: "minio".to_string(),
            enabled: config.infrastructure.minio,
            running: docker::is_container_running("devhub-minio").await,
            container_name: "devhub-minio".to_string(),
            image: config.infrastructure.minio_image.clone(),
        },
    ];

    // Check if shared network exists
    let networks = docker::list_devhub_networks(config)
        .await
        .unwrap_or_default();
    let shared_network_exists = networks.contains(&config.shared_network);

    Ok(InfraStatus {
        shared_network_exists,
        services,
    })
}

/// Initialize infrastructure directory and docker-compose file
pub fn init_infrastructure(config: &Config, force: bool) -> Result<()> {
    let infra_dir = &config.infrastructure.dir;
    let compose_path = infra_dir.join("docker-compose.yml");

    // Create directory if needed
    std::fs::create_dir_all(infra_dir)?;

    // Check if compose file exists
    if compose_path.exists() && !force {
        println!(
            "  {} Infrastructure already initialized at {}",
            "".yellow(),
            infra_dir.display()
        );
        println!("  Use --force to overwrite");
        return Ok(());
    }

    // Generate docker-compose.yml
    let compose_content = generate_compose_file(config);
    std::fs::write(&compose_path, compose_content)?;

    // Create init scripts directory
    let init_scripts_dir = infra_dir.join("init-scripts");
    std::fs::create_dir_all(&init_scripts_dir)?;

    // Create PostgreSQL init script for multi-tenant database setup
    let pg_init_script = generate_postgres_init_script();
    std::fs::write(init_scripts_dir.join("01-init-devhub.sql"), pg_init_script)?;

    println!(
        "  {} Infrastructure initialized at {}",
        "✓".green(),
        infra_dir.display()
    );

    Ok(())
}

/// Generate the docker-compose.yml content for infrastructure
fn generate_compose_file(config: &Config) -> String {
    let mut services = String::new();

    if config.infrastructure.postgres {
        services.push_str(&format!(
            r#"
  postgres:
    image: {}
    container_name: devhub-postgres
    restart: unless-stopped
    environment:
      POSTGRES_USER: devhub
      POSTGRES_PASSWORD: devhub
      POSTGRES_DB: devhub
    volumes:
      - devhub-postgres-data:/var/lib/postgresql/data
      - ./init-scripts:/docker-entrypoint-initdb.d:ro
    networks:
      - {}
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U devhub"]
      interval: 10s
      timeout: 5s
      retries: 5
"#,
            config.infrastructure.postgres_image, config.shared_network
        ));
    }

    if config.infrastructure.redis {
        services.push_str(&format!(
            r#"
  redis:
    image: {}
    container_name: devhub-redis
    restart: unless-stopped
    volumes:
      - devhub-redis-data:/data
    networks:
      - {}
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5
"#,
            config.infrastructure.redis_image, config.shared_network
        ));
    }

    if config.infrastructure.minio {
        services.push_str(&format!(
            r#"
  minio:
    image: {}
    container_name: devhub-minio
    restart: unless-stopped
    command: server /data --console-address ":9001"
    environment:
      MINIO_ROOT_USER: devhub
      MINIO_ROOT_PASSWORD: devhub123
    volumes:
      - devhub-minio-data:/data
    networks:
      - {}
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:9000/minio/health/live"]
      interval: 30s
      timeout: 20s
      retries: 3
"#,
            config.infrastructure.minio_image, config.shared_network
        ));
    }

    // Build volumes section
    let mut volumes = String::new();
    if config.infrastructure.postgres {
        volumes.push_str("  devhub-postgres-data:\n");
    }
    if config.infrastructure.redis {
        volumes.push_str("  devhub-redis-data:\n");
    }
    if config.infrastructure.minio {
        volumes.push_str("  devhub-minio-data:\n");
    }

    format!(
        r#"# DevHub Shared Infrastructure
# Auto-generated by DevHub - do not edit directly
# Configure via ~/.devhub/config.toml

services:{}

volumes:
{}
networks:
  {}:
    external: true
"#,
        services, volumes, config.shared_network
    )
}

/// Generate PostgreSQL init script for multi-tenant setup
fn generate_postgres_init_script() -> String {
    r#"-- DevHub PostgreSQL Init Script
-- Creates a function to easily provision per-project databases

-- Function to create a database for a project
CREATE OR REPLACE FUNCTION create_project_database(project_name TEXT)
RETURNS VOID AS $$
DECLARE
    db_name TEXT;
    user_name TEXT;
BEGIN
    -- Sanitize project name for database/user naming
    db_name := regexp_replace(project_name, '[^a-zA-Z0-9_]', '_', 'g');
    user_name := db_name;

    -- Create user if not exists
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = user_name) THEN
        EXECUTE format('CREATE USER %I WITH PASSWORD ''devhub''', user_name);
    END IF;

    -- Create database if not exists
    IF NOT EXISTS (SELECT FROM pg_database WHERE datname = db_name) THEN
        EXECUTE format('CREATE DATABASE %I OWNER %I', db_name, user_name);
    END IF;

    -- Grant privileges
    EXECUTE format('GRANT ALL PRIVILEGES ON DATABASE %I TO %I', db_name, user_name);
END;
$$ LANGUAGE plpgsql;

-- Grant execute to devhub user
GRANT EXECUTE ON FUNCTION create_project_database(TEXT) TO devhub;
"#
    .to_string()
}

/// Start infrastructure services
pub async fn start(config: &Config, services: Option<Vec<String>>) -> Result<()> {
    // Ensure shared network exists
    docker::ensure_shared_network(config).await?;

    // Initialize infrastructure if not already done
    let compose_path = config.infrastructure.dir.join("docker-compose.yml");
    if !compose_path.exists() {
        init_infrastructure(config, false)?;
    }

    // Run docker compose up
    let mut cmd = Command::new("docker");
    cmd.arg("compose")
        .arg("-f")
        .arg(&compose_path)
        .arg("up")
        .arg("-d");

    // Add specific services if provided
    if let Some(svcs) = services {
        for svc in svcs {
            cmd.arg(&svc);
        }
    }

    let output = cmd.output().context("Failed to run docker compose")?;

    if output.status.success() {
        println!("  {} Infrastructure services started", "✓".green());
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to start infrastructure: {}", stderr);
    }

    Ok(())
}

/// Stop infrastructure services
pub async fn stop(config: &Config, services: Option<Vec<String>>) -> Result<()> {
    let compose_path = config.infrastructure.dir.join("docker-compose.yml");

    if !compose_path.exists() {
        println!("  {} Infrastructure not initialized", "".yellow());
        return Ok(());
    }

    let mut cmd = Command::new("docker");
    cmd.arg("compose").arg("-f").arg(&compose_path).arg("stop");

    // Add specific services if provided
    if let Some(svcs) = services {
        for svc in svcs {
            cmd.arg(&svc);
        }
    }

    let output = cmd.output().context("Failed to run docker compose")?;

    if output.status.success() {
        println!("  {} Infrastructure services stopped", "✓".green());
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to stop infrastructure: {}", stderr);
    }

    Ok(())
}

/// Restart infrastructure services
pub async fn restart(config: &Config, services: Option<Vec<String>>) -> Result<()> {
    stop(config, services.clone()).await?;
    start(config, services).await?;
    Ok(())
}

/// Get logs from infrastructure services
pub fn logs(config: &Config, service: Option<&str>, follow: bool, lines: usize) -> Result<()> {
    let compose_path = config.infrastructure.dir.join("docker-compose.yml");

    if !compose_path.exists() {
        println!("  {} Infrastructure not initialized", "".yellow());
        return Ok(());
    }

    let mut cmd = Command::new("docker");
    cmd.arg("compose")
        .arg("-f")
        .arg(&compose_path)
        .arg("logs")
        .arg("--tail")
        .arg(lines.to_string());

    if follow {
        cmd.arg("-f");
    }

    if let Some(svc) = service {
        cmd.arg(svc);
    }

    // Run interactively
    let status = cmd.status().context("Failed to run docker compose logs")?;

    if !status.success() {
        anyhow::bail!("Failed to get logs");
    }

    Ok(())
}

/// Create a database for a project in shared PostgreSQL
pub async fn create_project_database(project_name: &str) -> Result<()> {
    // Sanitize project name
    let db_name = project_name.replace('-', "_").to_lowercase();

    let output = Command::new("docker")
        .args([
            "exec",
            "devhub-postgres",
            "psql",
            "-U",
            "devhub",
            "-c",
            &format!("SELECT create_project_database('{}')", db_name),
        ])
        .output()
        .context("Failed to create project database")?;

    if output.status.success() {
        println!("  {} Created database: {}", "✓".green(), db_name.cyan());
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Check if it's just a "already exists" error
        if !stderr.contains("already exists") {
            anyhow::bail!("Failed to create database: {}", stderr);
        }
    }

    Ok(())
}

/// Get environment variables for infrastructure connections
pub fn get_infra_env(project_name: &str, depends_on: &[InfraService]) -> HashMap<String, String> {
    let mut env = HashMap::new();
    let db_name = project_name.replace('-', "_").to_lowercase();

    for service in depends_on {
        match service {
            InfraService::Postgres => {
                env.insert("POSTGRES_HOST".to_string(), "devhub-postgres".to_string());
                env.insert("POSTGRES_PORT".to_string(), "5432".to_string());
                env.insert("POSTGRES_USER".to_string(), db_name.clone());
                env.insert("POSTGRES_PASSWORD".to_string(), "devhub".to_string());
                env.insert("POSTGRES_DB".to_string(), db_name.clone());
                env.insert(
                    "DATABASE_URL".to_string(),
                    format!(
                        "postgres://{}:devhub@devhub-postgres:5432/{}",
                        db_name, db_name
                    ),
                );
            }
            InfraService::Redis => {
                env.insert(
                    "REDIS_URL".to_string(),
                    "redis://devhub-redis:6379".to_string(),
                );
                env.insert("REDIS_HOST".to_string(), "devhub-redis".to_string());
                env.insert("REDIS_PORT".to_string(), "6379".to_string());
            }
            InfraService::Minio => {
                env.insert(
                    "MINIO_ENDPOINT".to_string(),
                    "devhub-minio:9000".to_string(),
                );
                env.insert("MINIO_ACCESS_KEY".to_string(), "devhub".to_string());
                env.insert("MINIO_SECRET_KEY".to_string(), "devhub123".to_string());
                env.insert(
                    "S3_ENDPOINT".to_string(),
                    "http://devhub-minio:9000".to_string(),
                );
            }
            InfraService::Kafka => {
                env.insert(
                    "KAFKA_BOOTSTRAP_SERVERS".to_string(),
                    "devhub-kafka:9092".to_string(),
                );
            }
            InfraService::Elasticsearch => {
                env.insert(
                    "ELASTICSEARCH_URL".to_string(),
                    "http://devhub-elasticsearch:9200".to_string(),
                );
            }
        }
    }

    env
}

/// Print infrastructure status in a nice format
pub fn print_status(status: &InfraStatus) {
    println!("\n{}", "Infrastructure Status".bold());
    println!("{}", "─".repeat(50));

    // Network status
    if status.shared_network_exists {
        println!(
            "  {} Shared network: {}",
            "✓".green(),
            "shared_proxy".cyan()
        );
    } else {
        println!(
            "  {} Shared network: {} (not created)",
            "✗".red(),
            "shared_proxy".dimmed()
        );
    }

    println!();
    println!("  {}", "Services:".bold());

    for svc in &status.services {
        let status_icon = if !svc.enabled {
            "○".dimmed()
        } else if svc.running {
            "●".green()
        } else {
            "○".red()
        };

        let status_text = if !svc.enabled {
            "disabled".dimmed().to_string()
        } else if svc.running {
            "running".green().to_string()
        } else {
            "stopped".red().to_string()
        };

        println!(
            "    {} {:<12} {} ({})",
            status_icon,
            svc.name,
            status_text,
            svc.image.dimmed()
        );
    }

    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_infra_env_postgres() {
        let env = get_infra_env("my-project", &[InfraService::Postgres]);

        assert_eq!(
            env.get("POSTGRES_HOST"),
            Some(&"devhub-postgres".to_string())
        );
        assert_eq!(env.get("POSTGRES_DB"), Some(&"my_project".to_string()));
        assert!(env.get("DATABASE_URL").unwrap().contains("my_project"));
    }

    #[test]
    fn test_get_infra_env_redis() {
        let env = get_infra_env("test", &[InfraService::Redis]);

        assert_eq!(
            env.get("REDIS_URL"),
            Some(&"redis://devhub-redis:6379".to_string())
        );
    }

    #[test]
    fn test_get_infra_env_multiple() {
        let env = get_infra_env(
            "full-stack",
            &[
                InfraService::Postgres,
                InfraService::Redis,
                InfraService::Minio,
            ],
        );

        assert!(env.contains_key("POSTGRES_HOST"));
        assert!(env.contains_key("REDIS_URL"));
        assert!(env.contains_key("MINIO_ENDPOINT"));
    }
}
