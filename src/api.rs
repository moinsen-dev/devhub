//! DevHub REST API for the dashboard
//!
//! Endpoints:
//!   GET  /api/projects          - List all projects with status
//!   GET  /api/projects/:name    - Get single project details
//!   POST /api/projects/:name/start   - Start a project
//!   POST /api/projects/:name/stop    - Stop a project
//!   POST /api/projects/:name/restart - Restart a project
//!   GET  /api/projects/:name/logs    - Get logs
//!   POST /api/projects/:name/open-terminal - Open project in Terminal
//!   POST /api/projects/:name/open-vscode   - Open project in VS Code

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

use crate::caddy;
use crate::manifest::{Manifest, Service};
use crate::process;
use crate::registry::Registry;

/// Shared application state
pub struct AppState {
    pub registry: RwLock<Registry>,
}

/// API response for project status
#[derive(Debug, Serialize)]
pub struct ProjectStatus {
    pub name: String,
    pub path: String,
    pub description: Option<String>,
    pub services: Vec<ServiceStatus>,
    pub any_running: bool,
}

#[derive(Debug, Serialize)]
pub struct ServiceStatus {
    pub name: String,
    pub port: u16,
    pub running: bool,
    pub url: Option<String>,
}

/// Query params for logs endpoint
#[derive(Debug, Deserialize)]
pub struct LogsQuery {
    pub service: Option<String>,
    #[serde(default = "default_lines")]
    pub lines: usize,
}

fn default_lines() -> usize {
    100
}

/// Create the API router
pub fn create_router(registry: Registry) -> Router {
    let state = Arc::new(AppState {
        registry: RwLock::new(registry),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/projects", get(list_projects))
        .route("/api/projects/:name", get(get_project))
        .route("/api/projects/:name/start", post(start_project))
        .route("/api/projects/:name/stop", post(stop_project))
        .route("/api/projects/:name/restart", post(restart_project))
        .route("/api/projects/:name/logs", get(get_logs))
        .route(
            "/api/projects/:name/services/:service/start",
            post(start_service),
        )
        .route(
            "/api/projects/:name/services/:service/stop",
            post(stop_service),
        )
        .route("/api/projects/:name/open-terminal", post(open_terminal))
        .route("/api/projects/:name/open-vscode", post(open_vscode))
        .layer(cors)
        .with_state(state)
}

/// List all projects with their status
async fn list_projects(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ProjectStatus>>, (StatusCode, String)> {
    let registry = state.registry.read().await;
    let projects = registry.list();

    let mut result = Vec::new();

    for (name, entry) in projects {
        let manifest_path = entry.path.join("devhub.toml");
        let manifest = match Manifest::load(&manifest_path) {
            Ok(m) => m,
            Err(_) => continue,
        };

        let mut services = Vec::new();
        let mut any_running = false;

        for svc in &manifest.services {
            let running = process::is_port_in_use(svc.port);
            if running {
                any_running = true;
            }

            let url = if running {
                Some(get_service_url(name, svc))
            } else {
                None
            };

            services.push(ServiceStatus {
                name: svc.name.clone(),
                port: svc.port,
                running,
                url,
            });
        }

        result.push(ProjectStatus {
            name: name.clone(),
            path: entry.path.display().to_string(),
            description: manifest.project.description,
            services,
            any_running,
        });
    }

    Ok(Json(result))
}

/// Get single project details
async fn get_project(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<ProjectStatus>, (StatusCode, String)> {
    let registry = state.registry.read().await;
    let entry = registry.get(&name).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            format!("Project '{}' not found", name),
        )
    })?;

    let manifest_path = entry.path.join("devhub.toml");
    let manifest = Manifest::load(&manifest_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut services = Vec::new();
    let mut any_running = false;

    for svc in &manifest.services {
        let running = process::is_port_in_use(svc.port);
        if running {
            any_running = true;
        }

        let url = if running {
            Some(get_service_url(&name, svc))
        } else {
            None
        };

        services.push(ServiceStatus {
            name: svc.name.clone(),
            port: svc.port,
            running,
            url,
        });
    }

    Ok(Json(ProjectStatus {
        name: name.clone(),
        path: entry.path.display().to_string(),
        description: manifest.project.description,
        services,
        any_running,
    }))
}

/// Start a project
async fn start_project(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<ProjectStatus>, (StatusCode, String)> {
    let registry = state.registry.read().await;
    let entry = registry
        .get(&name)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("Project '{}' not found", name),
            )
        })?
        .clone();
    drop(registry);

    let manifest_path = entry.path.join("devhub.toml");
    let manifest = Manifest::load(&manifest_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Generate Caddy config
    caddy::generate_config(&name, &manifest)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let _ = caddy::reload();

    // Start services
    for svc in &manifest.services {
        if let Err(e) = process::start_service(&name, &entry.path, svc, &manifest).await {
            tracing::error!("Failed to start service {}: {}", svc.name, e);
        }
    }

    // Return updated status
    get_project(State(state), Path(name)).await
}

/// Stop a project
async fn stop_project(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<ProjectStatus>, (StatusCode, String)> {
    let registry = state.registry.read().await;
    let entry = registry
        .get(&name)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("Project '{}' not found", name),
            )
        })?
        .clone();
    drop(registry);

    let manifest_path = entry.path.join("devhub.toml");
    let manifest = Manifest::load(&manifest_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Stop services
    for svc in &manifest.services {
        if let Err(e) = process::stop_service(&name, svc).await {
            tracing::error!("Failed to stop service {}: {}", svc.name, e);
        }
    }

    // Return updated status
    get_project(State(state), Path(name)).await
}

/// Restart a project
async fn restart_project(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<ProjectStatus>, (StatusCode, String)> {
    let _ = stop_project(State(state.clone()), Path(name.clone())).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    start_project(State(state), Path(name)).await
}

/// Start a specific service
async fn start_service(
    State(state): State<Arc<AppState>>,
    Path((name, service_name)): Path<(String, String)>,
) -> Result<Json<ProjectStatus>, (StatusCode, String)> {
    let registry = state.registry.read().await;
    let entry = registry
        .get(&name)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("Project '{}' not found", name),
            )
        })?
        .clone();
    drop(registry);

    let manifest_path = entry.path.join("devhub.toml");
    let manifest = Manifest::load(&manifest_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let svc = manifest
        .services
        .iter()
        .find(|s| s.name == service_name)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("Service '{}' not found", service_name),
            )
        })?;

    process::start_service(&name, &entry.path, svc, &manifest)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    get_project(State(state), Path(name)).await
}

/// Stop a specific service
async fn stop_service(
    State(state): State<Arc<AppState>>,
    Path((name, service_name)): Path<(String, String)>,
) -> Result<Json<ProjectStatus>, (StatusCode, String)> {
    let registry = state.registry.read().await;
    let entry = registry
        .get(&name)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("Project '{}' not found", name),
            )
        })?
        .clone();
    drop(registry);

    let manifest_path = entry.path.join("devhub.toml");
    let manifest = Manifest::load(&manifest_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let svc = manifest
        .services
        .iter()
        .find(|s| s.name == service_name)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("Service '{}' not found", service_name),
            )
        })?;

    process::stop_service(&name, svc)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    get_project(State(state), Path(name)).await
}

/// Get logs for a project
async fn get_logs(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Query(query): Query<LogsQuery>,
) -> Result<Json<Vec<String>>, (StatusCode, String)> {
    let registry = state.registry.read().await;
    let entry = registry
        .get(&name)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("Project '{}' not found", name),
            )
        })?
        .clone();
    drop(registry);

    let manifest_path = entry.path.join("devhub.toml");
    let manifest = Manifest::load(&manifest_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut all_logs = Vec::new();

    let services_to_check: Vec<_> = if let Some(ref service_name) = query.service {
        manifest
            .services
            .iter()
            .filter(|s| s.name == *service_name)
            .collect()
    } else {
        manifest.services.iter().collect()
    };

    for svc in services_to_check {
        // Get stderr logs
        if let Ok(lines) = process::read_service_logs(&name, &svc.name, query.lines, true) {
            for line in lines {
                all_logs.push(format!("[{}] {}", svc.name, line));
            }
        }
    }

    Ok(Json(all_logs))
}

/// Get the URL for a service based on Caddy routing
fn get_service_url(project_name: &str, service: &Service) -> String {
    if service.main {
        format!("http://{}.localhost", project_name)
    } else if let Some(ref subdomain) = service.subdomain {
        format!("http://{}.{}.localhost", subdomain, project_name)
    } else {
        format!("http://{}.{}.localhost", service.name, project_name)
    }
}

/// Response for open actions
#[derive(Debug, Serialize)]
pub struct OpenResponse {
    pub success: bool,
    pub message: String,
}

/// Open project in Terminal.app (macOS)
async fn open_terminal(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<OpenResponse>, (StatusCode, String)> {
    let registry = state.registry.read().await;
    let entry = registry.get(&name).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            format!("Project '{}' not found", name),
        )
    })?;

    let path = entry.path.display().to_string();

    // Use AppleScript to open Terminal and cd to the project directory
    let script = format!(
        r#"tell application "Terminal"
            activate
            do script "cd '{}' && clear"
        end tell"#,
        path
    );

    let output = std::process::Command::new("osascript")
        .args(["-e", &script])
        .output()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if output.status.success() {
        Ok(Json(OpenResponse {
            success: true,
            message: format!("Opened Terminal at {}", path),
        }))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to open Terminal: {}", stderr),
        ))
    }
}

/// Open project in VS Code
async fn open_vscode(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<OpenResponse>, (StatusCode, String)> {
    let registry = state.registry.read().await;
    let entry = registry.get(&name).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            format!("Project '{}' not found", name),
        )
    })?;

    let path = entry.path.display().to_string();

    // Try 'code' command first (VS Code CLI)
    let output = std::process::Command::new("code")
        .arg(&path)
        .output()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if output.status.success() {
        Ok(Json(OpenResponse {
            success: true,
            message: format!("Opened VS Code at {}", path),
        }))
    } else {
        // Fallback: Try opening via 'open' command on macOS
        let fallback = std::process::Command::new("open")
            .args(["-a", "Visual Studio Code", &path])
            .output()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        if fallback.status.success() {
            Ok(Json(OpenResponse {
                success: true,
                message: format!("Opened VS Code at {}", path),
            }))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to open VS Code: {}", stderr),
            ))
        }
    }
}
