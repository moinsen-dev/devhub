use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ServiceType {
    RustBinary,
    Node,
    Python,
    Go,
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
            ServiceType::DockerCompose => write!(f, "docker-compose"),
            ServiceType::Shell => write!(f, "shell"),
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
                },
            ],
            environment: HashMap::new(),
        };

        let ordered = manifest.services_in_order();
        assert_eq!(ordered[0].name, "api"); // api first (no deps)
        assert_eq!(ordered[1].name, "web"); // web second (depends on api)
    }
}
