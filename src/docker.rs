//! Docker network and container management for DevHub.
//!
//! This module provides functionality for managing Docker networks and containers
//! to enable port isolation across multiple projects.
//!
//! Note: This module is foundation code for CR-001 (Docker Network Isolation).
//! Functions are not yet wired into the main CLI but will be in a future release.

#![allow(dead_code)]

use anyhow::{Context, Result};
use bollard::container::{
    Config as ContainerConfig, CreateContainerOptions, ListContainersOptions,
    RemoveContainerOptions, StartContainerOptions, StopContainerOptions,
};
use bollard::network::{CreateNetworkOptions, ListNetworksOptions};
use bollard::Docker;
use colored::Colorize;
use std::collections::HashMap;

use crate::config::Config;
use crate::manifest::{ProjectMode, Service, ServiceMode};

/// Get a Docker client connection
pub async fn get_docker() -> Result<Docker> {
    Docker::connect_with_local_defaults().context("Failed to connect to Docker daemon")
}

/// Check if Docker daemon is available
pub async fn is_docker_available() -> bool {
    if let Ok(docker) = get_docker().await {
        docker.ping().await.is_ok()
    } else {
        false
    }
}

// ============================================================================
// Network Management
// ============================================================================

/// Create the shared proxy network if it doesn't exist
pub async fn ensure_shared_network(config: &Config) -> Result<()> {
    let docker = get_docker().await?;
    let network_name = &config.shared_network;

    // Check if network exists
    let filters: HashMap<String, Vec<String>> = [("name".to_string(), vec![network_name.clone()])]
        .into_iter()
        .collect();

    let networks = docker
        .list_networks(Some(ListNetworksOptions { filters }))
        .await?;

    if networks
        .iter()
        .any(|n| n.name.as_deref() == Some(network_name))
    {
        tracing::debug!("Shared network '{}' already exists", network_name);
        return Ok(());
    }

    // Create the network
    let options = CreateNetworkOptions {
        name: network_name.as_str(),
        driver: "bridge",
        ..Default::default()
    };

    docker.create_network(options).await?;
    println!(
        "  {} Created shared network: {}",
        "✓".green(),
        network_name.cyan()
    );

    Ok(())
}

/// Create a project-specific Docker network
pub async fn create_project_network(project_name: &str, config: &Config) -> Result<String> {
    let docker = get_docker().await?;
    let network_name = format!("{}-{}", config.network_prefix, project_name);

    // Check if network already exists
    let filters: HashMap<String, Vec<String>> = [("name".to_string(), vec![network_name.clone()])]
        .into_iter()
        .collect();

    let networks = docker
        .list_networks(Some(ListNetworksOptions { filters }))
        .await?;

    if networks
        .iter()
        .any(|n| n.name.as_deref() == Some(&network_name))
    {
        tracing::debug!("Project network '{}' already exists", network_name);
        return Ok(network_name);
    }

    // Create the network
    let options = CreateNetworkOptions {
        name: network_name.as_str(),
        driver: "bridge",
        ..Default::default()
    };

    docker.create_network(options).await?;
    println!(
        "  {} Created project network: {}",
        "✓".green(),
        network_name.cyan()
    );

    Ok(network_name)
}

/// Remove a project-specific Docker network
pub async fn remove_project_network(project_name: &str, config: &Config) -> Result<()> {
    let docker = get_docker().await?;
    let network_name = format!("{}-{}", config.network_prefix, project_name);

    // Check if network exists before trying to remove
    let filters: HashMap<String, Vec<String>> = [("name".to_string(), vec![network_name.clone()])]
        .into_iter()
        .collect();

    let networks = docker
        .list_networks(Some(ListNetworksOptions { filters }))
        .await?;

    if networks
        .iter()
        .any(|n| n.name.as_deref() == Some(&network_name))
    {
        docker.remove_network(&network_name).await?;
        println!(
            "  {} Removed project network: {}",
            "✓".green(),
            network_name.cyan()
        );
    }

    Ok(())
}

/// List all DevHub-managed networks
pub async fn list_devhub_networks(config: &Config) -> Result<Vec<String>> {
    let docker = get_docker().await?;

    let networks = docker.list_networks::<String>(None).await?;

    let devhub_networks: Vec<String> = networks
        .into_iter()
        .filter_map(|n| n.name)
        .filter(|name| name.starts_with(&config.network_prefix) || name == &config.shared_network)
        .collect();

    Ok(devhub_networks)
}

// ============================================================================
// Container Management
// ============================================================================

/// Build the container name for a service
pub fn container_name(project_name: &str, service_name: &str) -> String {
    format!("{}-{}", project_name, service_name)
}

/// Check if a container is running
pub async fn is_container_running(name: &str) -> bool {
    if let Ok(docker) = get_docker().await {
        let filters: HashMap<String, Vec<String>> = [
            ("name".to_string(), vec![format!("^/{}$", name)]),
            ("status".to_string(), vec!["running".to_string()]),
        ]
        .into_iter()
        .collect();

        if let Ok(containers) = docker
            .list_containers(Some(ListContainersOptions {
                filters,
                ..Default::default()
            }))
            .await
        {
            return !containers.is_empty();
        }
    }
    false
}

/// Check if a container exists (regardless of state)
pub async fn container_exists(name: &str) -> bool {
    if let Ok(docker) = get_docker().await {
        let filters: HashMap<String, Vec<String>> =
            [("name".to_string(), vec![format!("^/{}$", name)])]
                .into_iter()
                .collect();

        if let Ok(containers) = docker
            .list_containers(Some(ListContainersOptions {
                all: true,
                filters,
                ..Default::default()
            }))
            .await
        {
            return !containers.is_empty();
        }
    }
    false
}

/// Start a container for a service
pub async fn start_container(
    project_name: &str,
    service: &Service,
    networks: Vec<String>,
    env: HashMap<String, String>,
    config: &Config,
) -> Result<()> {
    let docker = get_docker().await?;
    let name = container_name(project_name, &service.name);

    // Check if already running
    if is_container_running(&name).await {
        println!(
            "  {} Container {} already running",
            "".yellow(),
            name.cyan()
        );
        return Ok(());
    }

    // Get the image to use
    let image = service
        .image
        .clone()
        .unwrap_or_else(|| get_default_image(&service.service_type));

    // Build environment variables
    let env_vec: Vec<String> = env
        .into_iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect();

    // Get the internal port (defaults to service port)
    let internal_port = service.internal_port.unwrap_or(service.port);

    // Build exposed ports
    let exposed_ports: HashMap<String, HashMap<(), ()>> =
        [(format!("{}/tcp", internal_port), HashMap::new())]
            .into_iter()
            .collect();

    // Create container config
    let container_config = ContainerConfig {
        image: Some(image.clone()),
        cmd: Some(
            service
                .command
                .split_whitespace()
                .map(String::from)
                .collect(),
        ),
        env: Some(env_vec),
        exposed_ports: Some(exposed_ports),
        working_dir: service.cwd.clone(),
        ..Default::default()
    };

    // Remove existing container if it exists but isn't running
    if container_exists(&name).await {
        let _ = docker
            .remove_container(
                &name,
                Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await;
    }

    // Create the container
    docker
        .create_container(
            Some(CreateContainerOptions {
                name: name.clone(),
                platform: None,
            }),
            container_config,
        )
        .await
        .context(format!("Failed to create container {}", name))?;

    // Connect to networks
    for network in networks {
        let _ = docker
            .connect_network(
                &network,
                bollard::network::ConnectNetworkOptions {
                    container: name.clone(),
                    ..Default::default()
                },
            )
            .await;
    }

    // Also connect to shared network
    let _ = docker
        .connect_network(
            &config.shared_network,
            bollard::network::ConnectNetworkOptions {
                container: name.clone(),
                ..Default::default()
            },
        )
        .await;

    // Start the container
    docker
        .start_container(&name, None::<StartContainerOptions<String>>)
        .await
        .context(format!("Failed to start container {}", name))?;

    println!(
        "  {} Started container: {} ({})",
        "✓".green(),
        name.cyan(),
        image.dimmed()
    );

    Ok(())
}

/// Stop a container
pub async fn stop_container(project_name: &str, service_name: &str) -> Result<()> {
    let docker = get_docker().await?;
    let name = container_name(project_name, service_name);

    if !is_container_running(&name).await {
        println!("  {} Container {} not running", "".dimmed(), name.cyan());
        return Ok(());
    }

    // Stop the container
    docker
        .stop_container(&name, Some(StopContainerOptions { t: 10 }))
        .await
        .context(format!("Failed to stop container {}", name))?;

    // Remove the container
    docker
        .remove_container(
            &name,
            Some(RemoveContainerOptions {
                force: false,
                ..Default::default()
            }),
        )
        .await
        .context(format!("Failed to remove container {}", name))?;

    println!("  {} Stopped container: {}", "✓".green(), name.cyan());

    Ok(())
}

/// Get container logs
pub async fn get_container_logs(
    project_name: &str,
    service_name: &str,
    lines: usize,
) -> Result<Vec<String>> {
    use bollard::container::LogsOptions;
    use futures_util::StreamExt;

    let docker = get_docker().await?;
    let name = container_name(project_name, service_name);

    let options = LogsOptions::<String> {
        stdout: true,
        stderr: true,
        tail: lines.to_string(),
        ..Default::default()
    };

    let mut logs = docker.logs(&name, Some(options));
    let mut result = Vec::new();

    while let Some(log_result) = logs.next().await {
        if let Ok(log) = log_result {
            result.push(log.to_string());
        }
    }

    Ok(result)
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get the effective mode for a service
pub fn get_effective_mode(service: &Service, project_mode: &ProjectMode) -> ProjectMode {
    match (&service.mode, project_mode) {
        (Some(ServiceMode::Container), _) => ProjectMode::Container,
        (Some(ServiceMode::Native), _) => ProjectMode::Native,
        (None, mode) => mode.clone(),
    }
}

/// Check if a service should run in container mode
pub fn should_run_in_container(service: &Service, project_mode: &ProjectMode) -> bool {
    matches!(
        get_effective_mode(service, project_mode),
        ProjectMode::Container
    )
}

/// Get the default Docker image for a service type
pub fn get_default_image(service_type: &crate::manifest::ServiceType) -> String {
    use crate::manifest::ServiceType;

    match service_type {
        ServiceType::Node => "node:22-alpine".to_string(),
        ServiceType::Python => "python:3.12-slim".to_string(),
        ServiceType::Go => "golang:1.22-alpine".to_string(),
        ServiceType::Dart => "dart:3.5".to_string(),
        ServiceType::RustBinary => "rust:1.75-alpine".to_string(),
        ServiceType::DockerCompose => "alpine:latest".to_string(),
        ServiceType::Shell => "alpine:latest".to_string(),
    }
}

/// List all containers for a project
pub async fn list_project_containers(project_name: &str) -> Result<Vec<(String, bool)>> {
    let docker = get_docker().await?;

    let filters: HashMap<String, Vec<String>> =
        [("name".to_string(), vec![format!("^/{}-", project_name)])]
            .into_iter()
            .collect();

    let containers = docker
        .list_containers(Some(ListContainersOptions {
            all: true,
            filters,
            ..Default::default()
        }))
        .await?;

    let result: Vec<(String, bool)> = containers
        .into_iter()
        .filter_map(|c| {
            let name = c.names?.first()?.trim_start_matches('/').to_string();
            let running = c.state.as_deref() == Some("running");
            Some((name, running))
        })
        .collect();

    Ok(result)
}

/// Clean up orphaned project networks (networks with no containers)
pub async fn cleanup_orphaned_networks(config: &Config) -> Result<usize> {
    let docker = get_docker().await?;
    let networks = list_devhub_networks(config).await?;
    let mut removed = 0;

    for network_name in networks {
        // Skip the shared proxy network
        if network_name == config.shared_network {
            continue;
        }

        // Try to remove the network (will fail if containers are attached)
        if docker.remove_network(&network_name).await.is_ok() {
            println!(
                "  {} Removed orphaned network: {}",
                "✓".green(),
                network_name.cyan()
            );
            removed += 1;
        }
    }

    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_name() {
        assert_eq!(container_name("my-project", "api"), "my-project-api");
        assert_eq!(container_name("test", "web"), "test-web");
    }

    #[test]
    fn test_get_default_image() {
        use crate::manifest::ServiceType;

        assert_eq!(get_default_image(&ServiceType::Node), "node:22-alpine");
        assert_eq!(get_default_image(&ServiceType::Dart), "dart:3.5");
        assert_eq!(get_default_image(&ServiceType::Python), "python:3.12-slim");
    }

    #[test]
    fn test_get_effective_mode() {
        use crate::manifest::ServiceType;

        let service_native = Service {
            name: "test".to_string(),
            service_type: ServiceType::Node,
            command: "npm start".to_string(),
            port: 3000,
            cwd: None,
            health_check: None,
            subdomain: None,
            main: true,
            depends_on: vec![],
            env: HashMap::new(),
            env_file: None,
            mode: Some(ServiceMode::Native),
            image: None,
            dockerfile: None,
            build_context: None,
            internal_port: None,
            networks: vec![],
            volumes: vec![],
        };

        // Service override should take precedence
        assert_eq!(
            get_effective_mode(&service_native, &ProjectMode::Container),
            ProjectMode::Native
        );

        let service_default = Service {
            mode: None,
            ..service_native.clone()
        };

        // Project mode should apply when no service override
        assert_eq!(
            get_effective_mode(&service_default, &ProjectMode::Container),
            ProjectMode::Container
        );
    }
}
