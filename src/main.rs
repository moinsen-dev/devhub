use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;

mod api;
mod caddy;
mod config;
mod discovery;
mod manifest;
mod process;
mod registry;

use registry::Registry;

#[derive(Parser)]
#[command(name = "devhub")]
#[command(about = "Multi-project development environment manager")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new devhub.toml in the current directory
    Init {
        /// Force overwrite existing devhub.toml
        #[arg(short, long)]
        force: bool,
    },

    /// Register a project with DevHub
    Register {
        /// Path to the project (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// Unregister a project from DevHub
    Unregister {
        /// Project name to unregister
        name: String,
    },

    /// List all registered projects
    List,

    /// Show status of all projects and services
    Status,

    /// Start project services
    Start {
        /// Project name (or current directory if not specified)
        project: Option<String>,

        /// Start specific service only
        #[arg(short, long)]
        service: Option<String>,
    },

    /// Stop project services
    Stop {
        /// Project name (or current directory if not specified)
        project: Option<String>,

        /// Stop specific service only
        #[arg(short, long)]
        service: Option<String>,
    },

    /// Restart project services
    Restart {
        /// Project name (or current directory if not specified)
        project: Option<String>,

        /// Restart specific service only
        #[arg(short, long)]
        service: Option<String>,
    },

    /// Stream logs from project services
    Logs {
        /// Project name
        project: String,

        /// Service name (optional, shows all if not specified)
        service: Option<String>,

        /// Follow log output
        #[arg(short, long)]
        follow: bool,
    },

    /// Auto-discover project type and generate devhub.toml
    Discover {
        /// Path to the project (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// Run the DevHub daemon (API server for dashboard)
    Daemon {
        /// Port to listen on
        #[arg(short, long, default_value = "9876")]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("devhub=info".parse().unwrap()),
        )
        .init();

    let cli = Cli::parse();
    let mut registry = Registry::load()?;

    match cli.command {
        Commands::Init { force } => cmd_init(force).await,
        Commands::Register { path } => cmd_register(&mut registry, path).await,
        Commands::Unregister { name } => cmd_unregister(&mut registry, &name).await,
        Commands::List => cmd_list(&registry).await,
        Commands::Status => cmd_status(&registry).await,
        Commands::Start { project, service } => cmd_start(&registry, project, service).await,
        Commands::Stop { project, service } => cmd_stop(&registry, project, service).await,
        Commands::Restart { project, service } => cmd_restart(&registry, project, service).await,
        Commands::Logs {
            project,
            service,
            follow,
        } => cmd_logs(&registry, &project, service, follow).await,
        Commands::Discover { path } => cmd_discover(path).await,
        Commands::Daemon { port } => cmd_daemon(port).await,
    }
}

async fn cmd_init(force: bool) -> Result<()> {
    let manifest_path = std::env::current_dir()?.join("devhub.toml");

    if manifest_path.exists() && !force {
        anyhow::bail!(
            "devhub.toml already exists. Use --force to overwrite."
        );
    }

    // Try to auto-discover project info
    let current_dir = std::env::current_dir()?;
    let project_name = current_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my-project")
        .to_string();

    let template = manifest::Manifest::template(&project_name);
    let content = toml::to_string_pretty(&template)?;

    std::fs::write(&manifest_path, content)?;
    println!(
        "{} Created {}",
        "✓".green(),
        manifest_path.display().to_string().cyan()
    );
    println!("  Edit the file to configure your services, then run:");
    println!("  {} devhub register", "$".dimmed());

    Ok(())
}

async fn cmd_register(registry: &mut Registry, path: PathBuf) -> Result<()> {
    let path = path.canonicalize()?;
    let manifest_path = path.join("devhub.toml");

    if !manifest_path.exists() {
        anyhow::bail!(
            "No devhub.toml found at {}. Run 'devhub init' first.",
            path.display()
        );
    }

    let manifest = manifest::Manifest::load(&manifest_path)?;
    let project_name = manifest.project.name.clone();

    registry.register(&project_name, &path)?;
    registry.save()?;

    println!(
        "{} Registered project '{}' at {}",
        "✓".green(),
        project_name.cyan(),
        path.display().to_string().dimmed()
    );

    // Show services
    if !manifest.services.is_empty() {
        println!("  Services:");
        for service in &manifest.services {
            let subdomain = if service.main {
                format!("{}.localhost", project_name)
            } else {
                format!(
                    "{}.{}.localhost",
                    service.subdomain.as_ref().unwrap_or(&service.name),
                    project_name
                )
            };
            println!(
                "    {} {} → http://{}",
                "•".dimmed(),
                service.name.yellow(),
                subdomain
            );
        }
    }

    Ok(())
}

async fn cmd_unregister(registry: &mut Registry, name: &str) -> Result<()> {
    if registry.unregister(name) {
        registry.save()?;
        println!("{} Unregistered project '{}'", "✓".green(), name.cyan());
    } else {
        println!(
            "{} Project '{}' not found",
            "!".yellow(),
            name
        );
    }
    Ok(())
}

async fn cmd_list(registry: &Registry) -> Result<()> {
    let projects = registry.list();

    if projects.is_empty() {
        println!("No projects registered.");
        println!("  Run 'devhub init' in a project directory, then 'devhub register'.");
        return Ok(());
    }

    println!("{}", "Registered Projects:".bold());
    println!();

    for (name, entry) in projects {
        println!("  {} {}", "●".green(), name.cyan().bold());
        println!("    Path: {}", entry.path.display().to_string().dimmed());
        println!(
            "    Registered: {}",
            entry.registered_at.format("%Y-%m-%d %H:%M").to_string().dimmed()
        );

        // Try to load manifest for service info
        let manifest_path = entry.path.join("devhub.toml");
        if let Ok(manifest) = manifest::Manifest::load(&manifest_path) {
            if !manifest.services.is_empty() {
                println!("    Services:");
                for service in &manifest.services {
                    println!(
                        "      {} {} (port {})",
                        "•".dimmed(),
                        service.name,
                        service.port
                    );
                }
            }
        }
        println!();
    }

    Ok(())
}

async fn cmd_status(registry: &Registry) -> Result<()> {
    let projects = registry.list();

    if projects.is_empty() {
        println!("No projects registered.");
        return Ok(());
    }

    println!("{}", "Project Status:".bold());
    println!();

    for (name, entry) in projects {
        let manifest_path = entry.path.join("devhub.toml");
        let manifest = match manifest::Manifest::load(&manifest_path) {
            Ok(m) => m,
            Err(_) => {
                println!("  {} {} (manifest error)", "○".red(), name.cyan());
                continue;
            }
        };

        // Check if any services are running
        let mut any_running = false;
        let mut service_statuses = Vec::new();

        for service in &manifest.services {
            let is_running = process::is_port_in_use(service.port);
            if is_running {
                any_running = true;
            }
            service_statuses.push((service.name.clone(), service.port, is_running));
        }

        let status_icon = if any_running { "●".green() } else { "○".dimmed() };
        println!("  {} {}", status_icon, name.cyan().bold());

        for (svc_name, port, running) in service_statuses {
            let svc_icon = if running { "●".green() } else { "○".dimmed() };
            let port_str = if running {
                format!(":{}", port).green().to_string()
            } else {
                format!(":{}", port).dimmed().to_string()
            };
            println!("    {} {}{}", svc_icon, svc_name, port_str);
        }
        println!();
    }

    Ok(())
}

async fn cmd_start(
    registry: &Registry,
    project: Option<String>,
    service: Option<String>,
) -> Result<()> {
    let (name, entry) = resolve_project(registry, project)?;
    let manifest_path = entry.path.join("devhub.toml");
    let manifest = manifest::Manifest::load(&manifest_path)?;

    println!(
        "{} Starting {}...",
        "→".blue(),
        name.cyan()
    );

    // Generate Caddy config
    caddy::generate_config(&name, &manifest)?;
    caddy::reload()?;

    // Start services
    let services_to_start: Vec<_> = if let Some(ref svc_name) = service {
        manifest
            .services
            .iter()
            .filter(|s| s.name == *svc_name)
            .collect()
    } else {
        manifest.services.iter().collect()
    };

    if services_to_start.is_empty() {
        anyhow::bail!("No matching services found");
    }

    for svc in services_to_start {
        process::start_service(&name, &entry.path, svc, &manifest.environment).await?;
    }

    println!("{} Started {}", "✓".green(), name.cyan());

    Ok(())
}

async fn cmd_stop(
    registry: &Registry,
    project: Option<String>,
    service: Option<String>,
) -> Result<()> {
    let (name, entry) = resolve_project(registry, project)?;
    let manifest_path = entry.path.join("devhub.toml");
    let manifest = manifest::Manifest::load(&manifest_path)?;

    println!(
        "{} Stopping {}...",
        "→".blue(),
        name.cyan()
    );

    let services_to_stop: Vec<_> = if let Some(ref svc_name) = service {
        manifest
            .services
            .iter()
            .filter(|s| s.name == *svc_name)
            .collect()
    } else {
        manifest.services.iter().collect()
    };

    for svc in services_to_stop {
        process::stop_service(&name, svc).await?;
    }

    println!("{} Stopped {}", "✓".green(), name.cyan());

    Ok(())
}

async fn cmd_restart(
    registry: &Registry,
    project: Option<String>,
    service: Option<String>,
) -> Result<()> {
    cmd_stop(registry, project.clone(), service.clone()).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    cmd_start(registry, project, service).await?;
    Ok(())
}

async fn cmd_logs(
    registry: &Registry,
    project: &str,
    service: Option<String>,
    follow: bool,
) -> Result<()> {
    let entry = registry
        .get(project)
        .ok_or_else(|| anyhow::anyhow!("Project '{}' not found", project))?;

    let manifest_path = entry.path.join("devhub.toml");
    let manifest = manifest::Manifest::load(&manifest_path)?;

    // Determine which services to show logs for
    let services: Vec<_> = if let Some(ref svc_name) = service {
        manifest
            .services
            .iter()
            .filter(|s| s.name == *svc_name)
            .collect()
    } else {
        manifest.services.iter().collect()
    };

    if services.is_empty() {
        anyhow::bail!("No matching services found");
    }

    if follow {
        // Follow mode - use tail -f
        let mut handles = Vec::new();

        for svc in &services {
            let log_path = process::tail_service_logs(project, &svc.name, true).await?;
            if log_path.exists() {
                let svc_name = svc.name.clone();
                let handle = tokio::spawn(async move {
                    let mut child = tokio::process::Command::new("tail")
                        .args(["-f", log_path.to_str().unwrap()])
                        .stdout(std::process::Stdio::piped())
                        .spawn()
                        .expect("Failed to spawn tail");

                    use tokio::io::{AsyncBufReadExt, BufReader};
                    let stdout = child.stdout.take().unwrap();
                    let mut reader = BufReader::new(stdout).lines();

                    while let Ok(Some(line)) = reader.next_line().await {
                        println!("{} {}", format!("[{}]", svc_name).cyan(), line);
                    }
                });
                handles.push(handle);
            }
        }

        if handles.is_empty() {
            println!("{} No log files found. Start the services first.", "!".yellow());
            return Ok(());
        }

        println!(
            "{} Following logs for {} (Ctrl+C to stop)...",
            "→".blue(),
            project.cyan()
        );
        println!();

        // Wait for Ctrl+C
        tokio::signal::ctrl_c().await?;

        // Kill all tail processes
        for handle in handles {
            handle.abort();
        }
    } else {
        // Show recent logs
        println!("{}", format!("Logs for {}:", project).bold());
        println!();

        for svc in services {
            println!("{}", format!("=== {} (stderr) ===", svc.name).cyan().bold());

            let lines = process::read_service_logs(project, &svc.name, 50, true)?;
            if lines.is_empty() {
                println!("{}", "(no logs)".dimmed());
            } else {
                for line in lines {
                    println!("{}", line);
                }
            }
            println!();

            println!("{}", format!("=== {} (stdout) ===", svc.name).cyan().bold());
            let lines = process::read_service_logs(project, &svc.name, 50, false)?;
            if lines.is_empty() {
                println!("{}", "(no logs)".dimmed());
            } else {
                for line in lines {
                    println!("{}", line);
                }
            }
            println!();
        }
    }

    Ok(())
}

async fn cmd_discover(path: PathBuf) -> Result<()> {
    let path = path.canonicalize()?;

    println!(
        "{} Discovering project at {}...",
        "→".blue(),
        path.display().to_string().cyan()
    );

    let discovered = discovery::discover_project(&path)?;

    println!();
    println!("{}", "Discovered Project:".bold());
    println!("  Name: {}", discovered.name.cyan());
    println!("  Type: {}", discovered.project_type.to_string().yellow());
    if let Some(ref desc) = discovered.description {
        println!("  Description: {}", desc);
    }

    if discovered.services.is_empty() {
        println!();
        println!(
            "{} No services detected. You may need to create devhub.toml manually.",
            "!".yellow()
        );
        return Ok(());
    }

    println!();
    println!("{}", "Detected Services:".bold());
    for svc in &discovered.services {
        println!(
            "  {} {} ({})",
            "•".dimmed(),
            svc.name.yellow(),
            svc.service_type.to_string()
        );
        println!("    Command: {}", svc.command.dimmed());
        if let Some(port) = svc.port {
            println!("    Port: {}", port);
        }
    }

    // Ask to generate devhub.toml
    println!();
    let manifest_path = path.join("devhub.toml");
    if manifest_path.exists() {
        println!(
            "{} devhub.toml already exists at {}",
            "!".yellow(),
            manifest_path.display()
        );
        println!("  Use 'devhub init --force' to overwrite");
    } else {
        // Generate manifest
        let manifest = discovery::to_manifest(&discovered);
        let content = toml::to_string_pretty(&manifest)?;
        std::fs::write(&manifest_path, &content)?;

        println!(
            "{} Generated {}",
            "✓".green(),
            manifest_path.display().to_string().cyan()
        );
        println!();
        println!("  Review the file and run:");
        println!("  {} devhub register", "$".dimmed());
    }

    Ok(())
}

async fn cmd_daemon(port: u16) -> Result<()> {
    let registry = Registry::load()?;

    println!(
        "{}",
        r#"
╔═══════════════════════════════════════════════════════════╗
║                                                           ║
║     ⚡ DevHub Daemon                                       ║
║                                                           ║
╚═══════════════════════════════════════════════════════════╝
"#
        .green()
    );

    println!(
        "  {} API:       http://localhost:{}",
        "→".blue(),
        port
    );
    println!(
        "  {} Dashboard: http://devhub.localhost",
        "→".blue(),
    );
    println!();
    println!("  Press {} to stop", "Ctrl+C".yellow());
    println!();

    // Create API router
    let app = api::create_router(registry);

    // Create listener
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!(
        "  {} Server listening on {}",
        "✓".green(),
        addr.to_string().cyan()
    );

    // Run server
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    println!();
    println!("{} Daemon stopped", "✓".green());

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
}

/// Resolve project name to registry entry
fn resolve_project(
    registry: &Registry,
    project: Option<String>,
) -> Result<(String, registry::ProjectEntry)> {
    match project {
        Some(name) => {
            let entry = registry
                .get(&name)
                .ok_or_else(|| anyhow::anyhow!("Project '{}' not found", name))?;
            Ok((name, entry.clone()))
        }
        None => {
            // Try to find project by current directory
            let current_dir = std::env::current_dir()?.canonicalize()?;
            for (name, entry) in registry.list() {
                if entry.path == current_dir {
                    return Ok((name.clone(), entry.clone()));
                }
            }
            anyhow::bail!(
                "No project found for current directory. Specify project name or run from a registered project directory."
            )
        }
    }
}
