use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Project execution mode
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProjectMode {
    /// Run services directly on host (current behavior)
    #[default]
    Native,
    /// Run all services in Docker containers with network isolation
    Container,
    /// Mix of native and containerized services (per-service override)
    Hybrid,
}

/// Service execution mode (for hybrid projects)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ServiceMode {
    Native,
    Container,
}

/// Shared infrastructure services that can be depended on
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum InfraService {
    Postgres,
    Redis,
    Minio,
    Kafka,
    Elasticsearch,
}

/// Project manifest (devhub.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub project: ProjectInfo,

    #[serde(default)]
    pub services: Vec<Service>,

    #[serde(default)]
    pub environment: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,

    #[serde(default)]
    pub description: Option<String>,

    #[serde(default)]
    pub tags: Vec<String>,

    /// Environment files to load for all services (e.g., [".env", ".env.local"])
    /// Loaded in order, later files override earlier ones
    #[serde(default)]
    pub env_files: Vec<String>,

    /// Project execution mode: native (default), container, or hybrid
    #[serde(default)]
    pub mode: ProjectMode,

    /// Shared infrastructure dependencies (postgres, redis, minio, etc.)
    #[serde(default)]
    pub depends_on_infra: Vec<InfraService>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub name: String,

    #[serde(rename = "type")]
    pub service_type: ServiceType,

    pub command: String,

    pub port: u16,

    /// Working directory relative to project root
    #[serde(default)]
    pub cwd: Option<String>,

    /// Health check endpoint (e.g., "/health" or "/api/v1/health")
    #[serde(default)]
    pub health_check: Option<String>,

    /// Subdomain for Caddy routing (e.g., "api" → api.project.localhost)
    #[serde(default)]
    pub subdomain: Option<String>,

    /// Is this the main service? (uses project.localhost without subdomain)
    #[serde(default)]
    pub main: bool,

    /// Services this service depends on (started first)
    #[serde(default)]
    pub depends_on: Vec<String>,

    /// Environment variables specific to this service
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Environment file to load for this service (relative to project root or service cwd)
    #[serde(default)]
    pub env_file: Option<String>,

    // === Container mode fields ===

    /// Per-service mode override (for hybrid projects)
    #[serde(default)]
    pub mode: Option<ServiceMode>,

    /// Docker image for containerization (e.g., "dart:3.5", "node:22-alpine")
    #[serde(default)]
    pub image: Option<String>,

    /// Path to Dockerfile (relative to project root)
    #[serde(default)]
    pub dockerfile: Option<String>,

    /// Build context for Dockerfile (relative to project root)
    #[serde(default)]
    pub build_context: Option<String>,

    /// Internal port for container mode (defaults to `port` field)
    /// This is the port the service listens on inside the container
    #[serde(default)]
    pub internal_port: Option<u16>,

    /// Additional Docker networks to attach this service to
    #[serde(default)]
    pub networks: Vec<String>,

    /// Volume mounts for the container (e.g., ["./data:/app/data"])
    #[serde(default)]
    pub volumes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ServiceType {
    RustBinary,
    Node,
    Python,
    Go,
    Dart,
    DockerCompose,
    Shell,
}

impl std::fmt::Display for ServiceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceType::RustBinary => write!(f, "rust-binary"),
            ServiceType::Node => write!(f, "node"),
            ServiceType::Python => write!(f, "python"),
            ServiceType::Go => write!(f, "go"),
            ServiceType::Dart => write!(f, "dart"),
            ServiceType::DockerCompose => write!(f, "docker-compose"),
            ServiceType::Shell => write!(f, "shell"),
        }
    }
}

impl std::fmt::Display for ProjectMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectMode::Native => write!(f, "native"),
            ProjectMode::Container => write!(f, "container"),
            ProjectMode::Hybrid => write!(f, "hybrid"),
        }
    }
}

impl Manifest {
    /// Load manifest from a file
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let manifest: Manifest = toml::from_str(&content)?;
        Ok(manifest)
    }

    /// Create a template manifest for a new project
    pub fn template(project_name: &str) -> Self {
        Manifest {
            project: ProjectInfo {
                name: project_name.to_string(),
                description: Some("My awesome project".to_string()),
                tags: vec![],
                env_files: vec![],
                mode: ProjectMode::default(),
                depends_on_infra: vec![],
            },
            services: vec![Service {
                name: "web".to_string(),
                service_type: ServiceType::Node,
                command: "npm run dev".to_string(),
                port: 3000,
                cwd: None,
                health_check: Some("/".to_string()),
                subdomain: None,
                main: true,
                depends_on: vec![],
                env: HashMap::new(),
                env_file: None,
                // Container mode fields (all optional)
                mode: None,
                image: None,
                dockerfile: None,
                build_context: None,
                internal_port: None,
                networks: vec![],
                volumes: vec![],
            }],
            environment: HashMap::new(),
        }
    }

    /// Get services in dependency order (services with no deps first)
    #[allow(dead_code)]
    pub fn services_in_order(&self) -> Vec<&Service> {
        let mut result = Vec::new();
        let mut remaining: Vec<&Service> = self.services.iter().collect();

        while !remaining.is_empty() {
            let before_len = remaining.len();

            remaining.retain(|svc| {
                let deps_satisfied = svc
                    .depends_on
                    .iter()
                    .all(|dep| result.iter().any(|s: &&Service| s.name == *dep));

                if deps_satisfied {
                    result.push(*svc);
                    false // Remove from remaining
                } else {
                    true // Keep in remaining
                }
            });

            // If we didn't make progress, there's a cycle
            if remaining.len() == before_len && !remaining.is_empty() {
                // Just add the rest in order (circular dependency)
                result.append(&mut remaining);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_manifest() {
        let toml = r#"
[project]
name = "test-project"
description = "A test project"

[[services]]
name = "api"
type = "rust-binary"
command = "cargo run"
port = 8080
subdomain = "api"

[[services]]
name = "web"
type = "node"
command = "npm run dev"
port = 3000
main = true
depends_on = ["api"]
"#;

        let manifest: Manifest = toml::from_str(toml).unwrap();
        assert_eq!(manifest.project.name, "test-project");
        assert_eq!(manifest.services.len(), 2);
        assert_eq!(manifest.services[0].name, "api");
        assert_eq!(manifest.services[1].depends_on, vec!["api"]);
    }

    #[test]
    fn test_dependency_order() {
        let manifest = Manifest {
            project: ProjectInfo {
                name: "test".to_string(),
                description: None,
                tags: vec![],
                env_files: vec![],
                mode: ProjectMode::default(),
                depends_on_infra: vec![],
            },
            services: vec![
                Service {
                    name: "web".to_string(),
                    service_type: ServiceType::Node,
                    command: "npm run dev".to_string(),
                    port: 3000,
                    cwd: None,
                    health_check: None,
                    subdomain: None,
                    main: true,
                    depends_on: vec!["api".to_string()],
                    env: HashMap::new(),
                    env_file: None,
                    mode: None,
                    image: None,
                    dockerfile: None,
                    build_context: None,
                    internal_port: None,
                    networks: vec![],
                    volumes: vec![],
                },
                Service {
                    name: "api".to_string(),
                    service_type: ServiceType::RustBinary,
                    command: "cargo run".to_string(),
                    port: 8080,
                    cwd: None,
                    health_check: None,
                    subdomain: Some("api".to_string()),
                    main: false,
                    depends_on: vec![],
                    env: HashMap::new(),
                    env_file: None,
                    mode: None,
                    image: None,
                    dockerfile: None,
                    build_context: None,
                    internal_port: None,
                    networks: vec![],
                    volumes: vec![],
                },
            ],
            environment: HashMap::new(),
        };

        let ordered = manifest.services_in_order();
        assert_eq!(ordered[0].name, "api"); // api first (no deps)
        assert_eq!(ordered[1].name, "web"); // web second (depends on api)
    }

    #[test]
    fn test_parse_manifest_with_env_files() {
        let toml = r#"
[project]
name = "test-project"
env_files = [".env", ".env.local"]

[[services]]
name = "api"
type = "rust-binary"
command = "cargo run"
port = 8080
env_file = "api/.env"
"#;

        let manifest: Manifest = toml::from_str(toml).unwrap();
        assert_eq!(manifest.project.env_files, vec![".env", ".env.local"]);
        assert_eq!(manifest.services[0].env_file, Some("api/.env".to_string()));
    }

    #[test]
    fn test_parse_container_mode_manifest() {
        let toml = r#"
[project]
name = "container-project"
mode = "container"
depends_on_infra = ["postgres", "redis"]

[[services]]
name = "api"
type = "dart"
command = "dart run bin/server.dart"
port = 8080
image = "dart:3.5"
subdomain = "api"

[[services]]
name = "web"
type = "node"
command = "npm run dev"
port = 3000
mode = "native"
main = true
"#;

        let manifest: Manifest = toml::from_str(toml).unwrap();
        assert_eq!(manifest.project.name, "container-project");
        assert_eq!(manifest.project.mode, ProjectMode::Container);
        assert_eq!(manifest.project.depends_on_infra.len(), 2);
        assert!(manifest.project.depends_on_infra.contains(&InfraService::Postgres));
        assert!(manifest.project.depends_on_infra.contains(&InfraService::Redis));

        // First service uses project default (container)
        assert_eq!(manifest.services[0].image, Some("dart:3.5".to_string()));
        assert_eq!(manifest.services[0].mode, None);

        // Second service overrides to native
        assert_eq!(manifest.services[1].mode, Some(ServiceMode::Native));
    }

    #[test]
    fn test_parse_hybrid_mode_manifest() {
        let toml = r#"
[project]
name = "hybrid-project"
mode = "hybrid"

[[services]]
name = "api"
type = "rust-binary"
command = "cargo run"
port = 8080
mode = "native"

[[services]]
name = "worker"
type = "node"
command = "npm run worker"
port = 8081
mode = "container"
image = "node:22-alpine"
dockerfile = "worker/Dockerfile"
volumes = ["./data:/app/data"]
"#;

        let manifest: Manifest = toml::from_str(toml).unwrap();
        assert_eq!(manifest.project.mode, ProjectMode::Hybrid);

        // API runs native
        assert_eq!(manifest.services[0].mode, Some(ServiceMode::Native));

        // Worker runs in container
        assert_eq!(manifest.services[1].mode, Some(ServiceMode::Container));
        assert_eq!(manifest.services[1].image, Some("node:22-alpine".to_string()));
        assert_eq!(
            manifest.services[1].dockerfile,
            Some("worker/Dockerfile".to_string())
        );
        assert_eq!(manifest.services[1].volumes, vec!["./data:/app/data"]);
    }
}
