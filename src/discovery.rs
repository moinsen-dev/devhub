//! Auto-discovery for project types
//!
//! Detects project types by scanning for common configuration files:
//! - Cargo.toml → Rust
//! - package.json → Node.js/TypeScript
//! - pubspec.yaml → Flutter/Dart
//! - pyproject.toml / uv.lock → Python (UV)
//! - requirements.txt → Python (pip)
//! - go.mod → Go
//! - docker-compose.yml → Docker

use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use crate::manifest::{Manifest, ProjectInfo, Service, ServiceType};

/// Discovered project information
#[derive(Debug)]
pub struct DiscoveredProject {
    pub name: String,
    pub description: Option<String>,
    pub project_type: ProjectType,
    pub services: Vec<DiscoveredService>,
}

/// Discovered service information
#[derive(Debug)]
pub struct DiscoveredService {
    pub name: String,
    pub service_type: ServiceType,
    pub command: String,
    pub port: Option<u16>,
    pub cwd: Option<String>,
}

/// Project type detection
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectType {
    Rust,
    Node,
    Flutter,
    PythonUv,
    PythonPip,
    Go,
    Docker,
    Unknown,
}

impl std::fmt::Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectType::Rust => write!(f, "Rust"),
            ProjectType::Node => write!(f, "Node.js/TypeScript"),
            ProjectType::Flutter => write!(f, "Flutter/Dart"),
            ProjectType::PythonUv => write!(f, "Python (UV)"),
            ProjectType::PythonPip => write!(f, "Python (pip)"),
            ProjectType::Go => write!(f, "Go"),
            ProjectType::Docker => write!(f, "Docker"),
            ProjectType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Discover project type and services from a directory
pub fn discover_project(path: &Path) -> Result<DiscoveredProject> {
    let project_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .to_string();

    // Detect project type (priority order)
    let (project_type, services, description) = if path.join("Cargo.toml").exists() {
        discover_rust(path, &project_name)?
    } else if path.join("pubspec.yaml").exists() {
        discover_flutter(path, &project_name)?
    } else if path.join("package.json").exists() {
        discover_node(path, &project_name)?
    } else if path.join("pyproject.toml").exists() || path.join("uv.lock").exists() {
        discover_python_uv(path, &project_name)?
    } else if path.join("requirements.txt").exists() {
        discover_python_pip(path, &project_name)?
    } else if path.join("go.mod").exists() {
        discover_go(path, &project_name)?
    } else if path.join("docker-compose.yml").exists() || path.join("docker-compose.yaml").exists()
    {
        discover_docker(path, &project_name)?
    } else {
        (ProjectType::Unknown, Vec::new(), None)
    };

    Ok(DiscoveredProject {
        name: project_name,
        description,
        project_type,
        services,
    })
}

/// Discover Rust project from Cargo.toml
fn discover_rust(
    path: &Path,
    project_name: &str,
) -> Result<(ProjectType, Vec<DiscoveredService>, Option<String>)> {
    let cargo_toml = std::fs::read_to_string(path.join("Cargo.toml"))?;

    #[derive(Deserialize)]
    struct CargoToml {
        package: Option<CargoPackage>,
        workspace: Option<CargoWorkspace>,
        bin: Option<Vec<CargoBin>>,
    }

    #[derive(Deserialize)]
    struct CargoPackage {
        name: Option<String>,
        description: Option<String>,
    }

    #[derive(Deserialize)]
    struct CargoWorkspace {
        members: Option<Vec<String>>,
    }

    #[derive(Deserialize)]
    struct CargoBin {
        name: String,
    }

    let cargo: CargoToml = toml::from_str(&cargo_toml)?;

    let description = cargo.package.as_ref().and_then(|p| p.description.clone());

    let mut services = Vec::new();

    // Check for binary targets in main Cargo.toml
    if let Some(bins) = cargo.bin {
        for (i, bin) in bins.iter().enumerate() {
            services.push(DiscoveredService {
                name: bin.name.clone(),
                service_type: ServiceType::RustBinary,
                command: format!("cargo run --bin {} --release", bin.name),
                port: Some(8080 + i as u16),
                cwd: None,
            });
        }
    } else if cargo.package.is_some() && cargo.workspace.is_none() {
        // Single binary with package name (non-workspace)
        let bin_name = cargo
            .package
            .as_ref()
            .and_then(|p| p.name.clone())
            .unwrap_or_else(|| project_name.to_string());

        services.push(DiscoveredService {
            name: bin_name.clone(),
            service_type: ServiceType::RustBinary,
            command: format!("cargo run --bin {} --release", bin_name),
            port: Some(8080),
            cwd: None,
        });
    }

    // Check workspace members for binaries
    if let Some(workspace) = cargo.workspace {
        if let Some(members) = workspace.members {
            for member in members {
                // Handle glob patterns
                let member_paths: Vec<_> = if member.contains('*') {
                    // Simple glob handling
                    let base = member.trim_end_matches("/*").trim_end_matches("/**");
                    let base_path = path.join(base);
                    if let Ok(entries) = std::fs::read_dir(&base_path) {
                        entries
                            .filter_map(|e| e.ok())
                            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                            .map(|e| path.join(base).join(e.file_name()))
                            .collect()
                    } else {
                        continue;
                    }
                } else {
                    vec![path.join(&member)]
                };

                for member_path in member_paths {
                    let member_cargo = member_path.join("Cargo.toml");
                    if !member_cargo.exists() {
                        continue;
                    }

                    if let Ok(content) = std::fs::read_to_string(&member_cargo) {
                        #[derive(Deserialize)]
                        #[allow(dead_code)]
                        struct MemberCargo {
                            package: Option<CargoPackage>,
                            bin: Option<Vec<CargoBin>>,
                        }

                        if let Ok(member) = toml::from_str::<MemberCargo>(&content) {
                            // Check for [[bin]] sections
                            if let Some(bins) = member.bin {
                                for bin in bins {
                                    // Check if binary name suggests it's a server/studio/api
                                    let is_server = bin.name.contains("server")
                                        || bin.name.contains("studio")
                                        || bin.name.contains("api")
                                        || bin.name.contains("daemon");

                                    if is_server {
                                        services.push(DiscoveredService {
                                            name: bin.name.clone(),
                                            service_type: ServiceType::RustBinary,
                                            command: format!("cargo run --bin {}", bin.name),
                                            port: Some(8080 + services.len() as u16),
                                            cwd: None,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok((ProjectType::Rust, services, description))
}

/// Discover Node.js/TypeScript project from package.json
fn discover_node(
    path: &Path,
    _project_name: &str,
) -> Result<(ProjectType, Vec<DiscoveredService>, Option<String>)> {
    let package_json = std::fs::read_to_string(path.join("package.json"))?;

    #[derive(Deserialize)]
    struct PackageJson {
        name: Option<String>,
        description: Option<String>,
        scripts: Option<HashMap<String, String>>,
    }

    let pkg: PackageJson = serde_json::from_str(&package_json)?;

    let description = pkg.description.clone();
    let mut services = Vec::new();

    if let Some(scripts) = &pkg.scripts {
        // Check for common dev/start scripts
        let dev_scripts = ["dev", "start", "serve", "start:dev"];

        for script_name in dev_scripts {
            if scripts.contains_key(script_name) {
                // Try to detect port from script
                let script_content = scripts.get(script_name).unwrap();
                let port = detect_port_from_script(script_content).unwrap_or(3000);

                // Detect package manager
                let cmd = if path.join("pnpm-lock.yaml").exists() {
                    format!("pnpm {}", script_name)
                } else if path.join("yarn.lock").exists() {
                    format!("yarn {}", script_name)
                } else if path.join("bun.lockb").exists() {
                    format!("bun run {}", script_name)
                } else {
                    format!("npm run {}", script_name)
                };

                services.push(DiscoveredService {
                    name: pkg.name.clone().unwrap_or_else(|| "app".to_string()),
                    service_type: ServiceType::Node,
                    command: cmd,
                    port: Some(port),
                    cwd: None,
                });
                break; // Only use first matching script
            }
        }
    }

    Ok((ProjectType::Node, services, description))
}

/// Discover Flutter/Dart project from pubspec.yaml
fn discover_flutter(
    path: &Path,
    _project_name: &str,
) -> Result<(ProjectType, Vec<DiscoveredService>, Option<String>)> {
    let pubspec = std::fs::read_to_string(path.join("pubspec.yaml"))?;

    #[derive(Deserialize)]
    struct Pubspec {
        name: Option<String>,
        description: Option<String>,
    }

    let spec: Pubspec = serde_yaml::from_str(&pubspec)?;

    let description = spec.description.clone();
    let mut services = Vec::new();

    // Check if it's a web-capable Flutter project
    let web_dir = path.join("web");
    if web_dir.exists() {
        services.push(DiscoveredService {
            name: spec
                .name
                .clone()
                .unwrap_or_else(|| "flutter_app".to_string()),
            service_type: ServiceType::Shell,
            command: "flutter run -d chrome --web-port 3000".to_string(),
            port: Some(3000),
            cwd: None,
        });
    }

    // Check for Dart server (serverpod, shelf, dart_frog)
    if path.join("bin").exists() {
        let bin_dir = std::fs::read_dir(path.join("bin"))?;
        let mut found_server = false;

        for entry in bin_dir.flatten() {
            let file_name = entry.file_name();
            let name = file_name.to_string_lossy();
            if name.ends_with(".dart") {
                let dart_name = name.trim_end_matches(".dart");

                // main.dart or server.dart suggests this is a backend server
                if dart_name == "main" || dart_name == "server" {
                    let project_name = spec.name.clone().unwrap_or_else(|| "server".to_string());

                    // Check if it's a Serverpod project (has generated directory)
                    let _is_serverpod =
                        path.join("lib/src/generated").exists() || path.join("generated").exists();

                    services.clear(); // Remove web service if this is a backend
                    services.push(DiscoveredService {
                        name: project_name,
                        service_type: ServiceType::Shell,
                        command: format!("dart run bin/{}.dart", dart_name),
                        port: Some(8080),
                        cwd: None,
                    });
                    found_server = true;
                    break;
                } else if dart_name != "main" {
                    // Other dart files in bin/ are likely services
                    services.push(DiscoveredService {
                        name: dart_name.to_string(),
                        service_type: ServiceType::Shell,
                        command: format!("dart run bin/{}.dart", dart_name),
                        port: Some(8080 + services.len() as u16),
                        cwd: None,
                    });
                    found_server = true;
                }
            }
        }

        // If we found a backend server, don't also add Flutter web
        if found_server {
            // Filter out any web service that was added
            services.retain(|s| !s.command.contains("flutter run"));
        }
    }

    Ok((ProjectType::Flutter, services, description))
}

/// Discover Python UV project
fn discover_python_uv(
    path: &Path,
    project_name: &str,
) -> Result<(ProjectType, Vec<DiscoveredService>, Option<String>)> {
    let mut description = None;
    let mut services = Vec::new();

    // Try to parse pyproject.toml for project info
    if let Ok(content) = std::fs::read_to_string(path.join("pyproject.toml")) {
        #[derive(Deserialize)]
        struct PyProject {
            project: Option<PyProjectInfo>,
            tool: Option<PyProjectTool>,
        }

        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct PyProjectInfo {
            name: Option<String>,
            description: Option<String>,
            scripts: Option<HashMap<String, String>>,
        }

        #[derive(Deserialize)]
        struct PyProjectTool {
            uv: Option<UvTool>,
        }

        #[derive(Deserialize)]
        struct UvTool {
            scripts: Option<HashMap<String, String>>,
        }

        if let Ok(pyproject) = toml::from_str::<PyProject>(&content) {
            description = pyproject
                .project
                .as_ref()
                .and_then(|p| p.description.clone());

            // Check for UV scripts
            if let Some(uv) = pyproject.tool.and_then(|t| t.uv) {
                if let Some(scripts) = uv.scripts {
                    for (name, cmd) in scripts {
                        let port = detect_port_from_script(&cmd);
                        services.push(DiscoveredService {
                            name,
                            service_type: ServiceType::Python,
                            command: format!("uv run {}", cmd),
                            port,
                            cwd: None,
                        });
                    }
                }
            }

            // Check for project scripts
            if let Some(scripts) = pyproject.project.and_then(|p| p.scripts) {
                for (name, _) in scripts {
                    services.push(DiscoveredService {
                        name: name.clone(),
                        service_type: ServiceType::Python,
                        command: format!("uv run {}", name),
                        port: Some(8000),
                        cwd: None,
                    });
                }
            }
        }
    }

    // Common Python web frameworks
    let common_entrypoints = [
        ("main.py", "uv run python main.py", 8000),
        ("app.py", "uv run python app.py", 5000),
        ("server.py", "uv run python server.py", 8000),
        ("manage.py", "uv run python manage.py runserver", 8000), // Django
        ("app/__init__.py", "uv run uvicorn app:app --reload", 8000), // FastAPI
    ];

    if services.is_empty() {
        for (file, cmd, port) in common_entrypoints {
            if path.join(file).exists() {
                services.push(DiscoveredService {
                    name: project_name.to_string(),
                    service_type: ServiceType::Python,
                    command: cmd.to_string(),
                    port: Some(port),
                    cwd: None,
                });
                break;
            }
        }
    }

    Ok((ProjectType::PythonUv, services, description))
}

/// Discover Python pip project
fn discover_python_pip(
    path: &Path,
    project_name: &str,
) -> Result<(ProjectType, Vec<DiscoveredService>, Option<String>)> {
    let mut services = Vec::new();

    // Common Python web frameworks
    let common_entrypoints = [
        ("main.py", "python main.py", 8000),
        ("app.py", "python app.py", 5000),
        ("server.py", "python server.py", 8000),
        ("manage.py", "python manage.py runserver", 8000),
    ];

    for (file, cmd, port) in common_entrypoints {
        if path.join(file).exists() {
            services.push(DiscoveredService {
                name: project_name.to_string(),
                service_type: ServiceType::Python,
                command: cmd.to_string(),
                port: Some(port),
                cwd: None,
            });
            break;
        }
    }

    Ok((ProjectType::PythonPip, services, None))
}

/// Discover Go project
fn discover_go(
    path: &Path,
    project_name: &str,
) -> Result<(ProjectType, Vec<DiscoveredService>, Option<String>)> {
    let mut services = Vec::new();

    // Check for main.go or cmd directory
    if path.join("main.go").exists() {
        services.push(DiscoveredService {
            name: project_name.to_string(),
            service_type: ServiceType::Go,
            command: "go run .".to_string(),
            port: Some(8080),
            cwd: None,
        });
    } else if path.join("cmd").exists() {
        // Check cmd/ directory for binaries
        if let Ok(entries) = std::fs::read_dir(path.join("cmd")) {
            for (i, entry) in entries.flatten().enumerate() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    let name = entry.file_name().to_string_lossy().to_string();
                    services.push(DiscoveredService {
                        name: name.clone(),
                        service_type: ServiceType::Go,
                        command: format!("go run ./cmd/{}", name),
                        port: Some(8080 + i as u16),
                        cwd: None,
                    });
                }
            }
        }
    }

    Ok((ProjectType::Go, services, None))
}

/// Discover Docker Compose project
fn discover_docker(
    path: &Path,
    project_name: &str,
) -> Result<(ProjectType, Vec<DiscoveredService>, Option<String>)> {
    let compose_file = if path.join("docker-compose.yml").exists() {
        "docker-compose.yml"
    } else {
        "docker-compose.yaml"
    };

    let mut services = Vec::new();

    // For now, just add a single docker-compose service
    // TODO: Parse docker-compose.yml to extract individual services
    services.push(DiscoveredService {
        name: project_name.to_string(),
        service_type: ServiceType::DockerCompose,
        command: format!("docker compose -f {} up", compose_file),
        port: None,
        cwd: None,
    });

    Ok((ProjectType::Docker, services, None))
}

/// Try to detect port from a script command
fn detect_port_from_script(script: &str) -> Option<u16> {
    // Common patterns: --port 3000, -p 8080, :3000, PORT=3000
    let patterns = [
        r"--port[=\s]+(\d+)",
        r"-p[=\s]+(\d+)",
        r":(\d{4,5})",
        r"PORT[=\s]+(\d+)",
        r"--web-port[=\s]+(\d+)",
    ];

    for pattern in patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(caps) = re.captures(script) {
                if let Some(port_str) = caps.get(1) {
                    if let Ok(port) = port_str.as_str().parse::<u16>() {
                        if port >= 1024 {
                            return Some(port);
                        }
                    }
                }
            }
        }
    }

    None
}

/// Convert discovered project to a manifest
pub fn to_manifest(discovered: &DiscoveredProject) -> Manifest {
    let services: Vec<Service> = discovered
        .services
        .iter()
        .enumerate()
        .map(|(i, svc)| Service {
            name: svc.name.clone(),
            service_type: svc.service_type.clone(),
            command: svc.command.clone(),
            port: svc.port.unwrap_or(3000 + i as u16),
            cwd: svc.cwd.clone(),
            health_check: Some("/".to_string()),
            subdomain: if i == 0 { None } else { Some(svc.name.clone()) },
            main: i == 0,
            depends_on: Vec::new(),
            env: HashMap::new(),
        })
        .collect();

    Manifest {
        project: ProjectInfo {
            name: discovered.name.clone(),
            description: Some(
                discovered
                    .description
                    .clone()
                    .unwrap_or_else(|| format!("{} project", discovered.project_type)),
            ),
            tags: vec![discovered.project_type.to_string().to_lowercase()],
        },
        services,
        environment: HashMap::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_port_from_script() {
        assert_eq!(detect_port_from_script("next dev --port 3001"), Some(3001));
        assert_eq!(detect_port_from_script("vite -p 5173"), Some(5173));
        assert_eq!(
            detect_port_from_script("uvicorn app:app --host 0.0.0.0 --port 8000"),
            Some(8000)
        );
        assert_eq!(
            detect_port_from_script("PORT=3000 node server.js"),
            Some(3000)
        );
    }
}
