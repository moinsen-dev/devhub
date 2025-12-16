use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use colored::Colorize;
use std::io;
use std::path::PathBuf;

mod api;
mod caddy;
mod config;
mod discovery;
mod manifest;
mod ports;
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

        /// Start all registered projects
        #[arg(long)]
        all: bool,

        /// Start only favorite projects
        #[arg(long)]
        favorites: bool,
    },

    /// Stop project services
    Stop {
        /// Project name (or current directory if not specified)
        project: Option<String>,

        /// Stop specific service only
        #[arg(short, long)]
        service: Option<String>,

        /// Stop all running projects
        #[arg(long)]
        all: bool,

        /// Stop only favorite projects
        #[arg(long)]
        favorites: bool,
    },

    /// Restart project services
    Restart {
        /// Project name (or current directory if not specified)
        project: Option<String>,

        /// Restart specific service only
        #[arg(short, long)]
        service: Option<String>,

        /// Restart all running projects
        #[arg(long)]
        all: bool,

        /// Restart only favorite projects
        #[arg(long)]
        favorites: bool,
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

        /// Only show what would be discovered (don't write devhub.toml)
        #[arg(long)]
        dry_run: bool,
    },

    /// Show resolved environment variables for a project or service
    Env {
        /// Project name
        project: String,

        /// Service name (optional, shows all services if not specified)
        #[arg(short, long)]
        service: Option<String>,

        /// Output format (table or export)
        #[arg(short, long, default_value = "table")]
        format: String,
    },

    /// Scan directories for projects and bulk register them
    Scan {
        /// Directories to scan (defaults to ~/work/moinsen/{ideas,opensource,apps})
        #[arg(num_args = 0..)]
        paths: Vec<PathBuf>,

        /// Maximum depth to scan (default: 2)
        #[arg(short, long, default_value = "2")]
        depth: usize,

        /// Auto-register discovered projects without prompting
        #[arg(short, long)]
        auto_register: bool,

        /// Only show what would be discovered (dry run)
        #[arg(long)]
        dry_run: bool,
    },

    /// Show port allocations across all projects
    Ports {
        /// Check for conflicts
        #[arg(short, long)]
        check: bool,

        /// Suggest resolutions for conflicts
        #[arg(short, long)]
        resolve: bool,
    },

    /// Run the DevHub daemon (API server for dashboard)
    Daemon {
        /// Port to listen on
        #[arg(short, long, default_value = "9876")]
        port: u16,
    },

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Open a project in the default browser
    Open {
        /// Project name
        project: String,

        /// Service name (optional, opens main service if not specified)
        #[arg(short, long)]
        service: Option<String>,
    },

    /// Open a project in VS Code
    Code {
        /// Project name
        project: String,
    },

    /// Print a project's path (for cd integration)
    Path {
        /// Project name
        project: String,
    },

    /// Search for projects (fuzzy matching)
    Search {
        /// Search query
        query: String,

        /// Maximum number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Manage favorite projects
    Fav {
        #[command(subcommand)]
        action: FavAction,
    },

    /// Show recently used projects
    Recent {
        /// Maximum number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
}

#[derive(Subcommand)]
enum FavAction {
    /// Add project to favorites
    Add {
        /// Project name
        project: String,
    },
    /// Remove project from favorites
    Remove {
        /// Project name
        project: String,
    },
    /// List favorite projects
    List,
    /// Toggle favorite status
    Toggle {
        /// Project name
        project: String,
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
        Commands::Start {
            project,
            service,
            all,
            favorites,
        } => cmd_start(&mut registry, project, service, all, favorites).await,
        Commands::Stop {
            project,
            service,
            all,
            favorites,
        } => cmd_stop(&mut registry, project, service, all, favorites).await,
        Commands::Restart {
            project,
            service,
            all,
            favorites,
        } => cmd_restart(&mut registry, project, service, all, favorites).await,
        Commands::Logs {
            project,
            service,
            follow,
        } => cmd_logs(&registry, &project, service, follow).await,
        Commands::Discover { path, dry_run } => cmd_discover(path, dry_run).await,
        Commands::Env {
            project,
            service,
            format,
        } => cmd_env(project, service, format).await,
        Commands::Scan {
            paths,
            depth,
            auto_register,
            dry_run,
        } => cmd_scan(&mut registry, paths, depth, auto_register, dry_run).await,
        Commands::Ports { check, resolve } => cmd_ports(&registry, check, resolve).await,
        Commands::Daemon { port } => cmd_daemon(port).await,
        Commands::Completions { shell } => cmd_completions(shell),
        Commands::Open { project, service } => cmd_open(&registry, &project, service).await,
        Commands::Code { project } => cmd_code(&registry, &project).await,
        Commands::Path { project } => cmd_path(&registry, &project).await,
        Commands::Search { query, limit } => cmd_search(&registry, &query, limit).await,
        Commands::Fav { action } => cmd_fav(&mut registry, action).await,
        Commands::Recent { limit } => cmd_recent(&registry, limit).await,
    }
}

async fn cmd_init(force: bool) -> Result<()> {
    let manifest_path = std::env::current_dir()?.join("devhub.toml");

    if manifest_path.exists() && !force {
        anyhow::bail!("devhub.toml already exists. Use --force to overwrite.");
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
        println!("{} Project '{}' not found", "!".yellow(), name);
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
            entry
                .registered_at
                .format("%Y-%m-%d %H:%M")
                .to_string()
                .dimmed()
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

        let status_icon = if any_running {
            "●".green()
        } else {
            "○".dimmed()
        };
        println!("  {} {}", status_icon, name.cyan().bold());

        for (svc_name, port, running) in service_statuses {
            let svc_icon = if running {
                "●".green()
            } else {
                "○".dimmed()
            };
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
    registry: &mut Registry,
    project: Option<String>,
    service: Option<String>,
    all: bool,
    favorites_only: bool,
) -> Result<()> {
    // Handle batch operations
    if all || favorites_only {
        let projects_to_start: Vec<_> = if favorites_only {
            registry
                .favorites()
                .into_iter()
                .map(|(n, e)| (n.clone(), e.clone()))
                .collect()
        } else {
            registry
                .list()
                .into_iter()
                .map(|(n, e)| (n.clone(), e.clone()))
                .collect()
        };

        if projects_to_start.is_empty() {
            println!(
                "{}",
                if favorites_only {
                    "No favorite projects found."
                } else {
                    "No projects registered."
                }
            );
            return Ok(());
        }

        let label = if favorites_only {
            "favorite projects"
        } else {
            "all projects"
        };
        println!(
            "{} Starting {} ({})...",
            "→".blue(),
            label,
            projects_to_start.len()
        );
        println!();

        for (name, entry) in projects_to_start {
            let manifest_path = entry.path.join("devhub.toml");
            if let Ok(manifest) = manifest::Manifest::load(&manifest_path) {
                println!("{} {}...", "→".blue(), name.cyan());
                caddy::generate_config(&name, &manifest)?;

                for svc in &manifest.services {
                    let _ = process::start_service(&name, &entry.path, svc, &manifest)
                        .await;
                }

                // Track as recently used
                registry.touch(&name);
            }
        }

        registry.save()?;
        caddy::reload()?;
        println!();
        println!("{} Started {}", "✓".green(), label);
        return Ok(());
    }

    // Single project start
    let (name, entry) = resolve_project(registry, project)?;
    let manifest_path = entry.path.join("devhub.toml");
    let manifest = manifest::Manifest::load(&manifest_path)?;

    println!("{} Starting {}...", "→".blue(), name.cyan());

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
        process::start_service(&name, &entry.path, svc, &manifest).await?;
    }

    // Track as recently used
    registry.touch(&name);
    registry.save()?;

    println!("{} Started {}", "✓".green(), name.cyan());

    Ok(())
}

async fn cmd_stop(
    registry: &mut Registry,
    project: Option<String>,
    service: Option<String>,
    all: bool,
    favorites_only: bool,
) -> Result<()> {
    // Handle batch operations
    if all || favorites_only {
        let projects_to_stop: Vec<_> = if favorites_only {
            registry
                .favorites()
                .into_iter()
                .map(|(n, e)| (n.clone(), e.clone()))
                .collect()
        } else {
            registry
                .list()
                .into_iter()
                .map(|(n, e)| (n.clone(), e.clone()))
                .collect()
        };

        if projects_to_stop.is_empty() {
            println!(
                "{}",
                if favorites_only {
                    "No favorite projects found."
                } else {
                    "No projects registered."
                }
            );
            return Ok(());
        }

        let label = if favorites_only {
            "favorite projects"
        } else {
            "all projects"
        };
        println!(
            "{} Stopping {} ({})...",
            "→".blue(),
            label,
            projects_to_stop.len()
        );
        println!();

        for (name, entry) in projects_to_stop {
            let manifest_path = entry.path.join("devhub.toml");
            if let Ok(manifest) = manifest::Manifest::load(&manifest_path) {
                // Check if any service is running
                let any_running = manifest
                    .services
                    .iter()
                    .any(|s| process::is_port_in_use(s.port));

                if any_running {
                    println!("{} {}...", "→".blue(), name.cyan());
                    for svc in &manifest.services {
                        let _ = process::stop_service(&name, svc).await;
                    }
                }
            }
        }

        println!();
        println!("{} Stopped {}", "✓".green(), label);
        return Ok(());
    }

    // Single project stop
    let (name, entry) = resolve_project(registry, project)?;
    let manifest_path = entry.path.join("devhub.toml");
    let manifest = manifest::Manifest::load(&manifest_path)?;

    println!("{} Stopping {}...", "→".blue(), name.cyan());

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
    registry: &mut Registry,
    project: Option<String>,
    service: Option<String>,
    all: bool,
    favorites_only: bool,
) -> Result<()> {
    cmd_stop(
        registry,
        project.clone(),
        service.clone(),
        all,
        favorites_only,
    )
    .await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    cmd_start(registry, project, service, all, favorites_only).await?;
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
            println!(
                "{} No log files found. Start the services first.",
                "!".yellow()
            );
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

async fn cmd_discover(path: PathBuf, dry_run: bool) -> Result<()> {
    let path = path.canonicalize()?;

    println!(
        "{} Discovering project at {}...{}",
        "→".blue(),
        path.display().to_string().cyan(),
        if dry_run { " (dry-run)".dimmed().to_string() } else { String::new() }
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
            svc.service_type
        );
        println!("    Command: {}", svc.command.dimmed());
        if let Some(port) = svc.port {
            println!("    Port: {}", port);
        }
        if let Some(ref cwd) = svc.cwd {
            println!("    Directory: {}", cwd.dimmed());
        }
    }

    // In dry-run mode, show what would be generated but don't write
    if dry_run {
        println!();
        let manifest = discovery::to_manifest(&discovered);
        let content = toml::to_string_pretty(&manifest)?;
        println!("{}", "Would generate devhub.toml:".bold());
        println!("{}", "─".repeat(50).dimmed());
        println!("{}", content);
        println!("{}", "─".repeat(50).dimmed());
        println!();
        println!(
            "{} Run without --dry-run to generate the file",
            "→".blue()
        );
        return Ok(());
    }

    // Generate devhub.toml
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

/// Show resolved environment variables for a project or service
async fn cmd_env(project: String, service: Option<String>, format: String) -> Result<()> {
    let registry = Registry::load()?;
    let (name, entry) = resolve_project(&registry, Some(project))?;

    let manifest_path = entry.path.join("devhub.toml");
    let manifest = manifest::Manifest::load(&manifest_path)?;

    // Determine which services to show
    let services: Vec<&manifest::Service> = if let Some(ref svc_name) = service {
        manifest
            .services
            .iter()
            .filter(|s| s.name == *svc_name)
            .collect()
    } else {
        manifest.services.iter().collect()
    };

    if services.is_empty() {
        if let Some(svc_name) = service {
            anyhow::bail!("Service '{}' not found in project '{}'", svc_name, name);
        } else {
            anyhow::bail!("No services found in project '{}'", name);
        }
    }

    for svc in services {
        let env = process::get_resolved_environment(&entry.path, &manifest, Some(svc));

        println!("{}", format!("Service: {}", svc.name).cyan().bold());
        println!();

        // Sort keys for consistent output
        let mut keys: Vec<_> = env.keys().collect();
        keys.sort();

        if format == "export" {
            // Shell export format for sourcing
            for key in keys {
                // Skip system vars for cleaner output
                if !is_system_env_var(key) {
                    let value = &env[key];
                    // Escape single quotes in value
                    let escaped = value.replace("'", "'\\''");
                    println!("export {}='{}'", key, escaped);
                }
            }
        } else {
            // Table format
            let mut custom_count = 0;
            for key in keys {
                if !is_system_env_var(key) {
                    let value = &env[key];
                    println!("  {} = {}", key.yellow(), value.dimmed());
                    custom_count += 1;
                }
            }
            if custom_count == 0 {
                println!("  {} No custom environment variables", "·".dimmed());
            }
        }
        println!();
    }

    // Show env file sources
    println!("{}", "Environment sources (highest priority first):".dimmed());
    println!("  1. Service env_file: {}", service.as_deref().and_then(|s| {
        manifest.services.iter().find(|svc| svc.name == s).and_then(|svc| svc.env_file.as_deref())
    }).unwrap_or("(none)").dimmed());
    println!("  2. Service cwd/.env.local");
    println!("  3. Service cwd/.env");
    println!("  4. Project root .env.local");
    println!("  5. Project root .env");
    println!("  6. Manifest env_files: {:?}", manifest.project.env_files);
    println!("  7. Manifest [environment] section");
    println!("  8. Service env = {{}}");
    println!();

    Ok(())
}

/// Check if a variable is a system environment variable
fn is_system_env_var(key: &str) -> bool {
    key.starts_with("PATH")
        || key.starts_with("HOME")
        || key.starts_with("USER")
        || key.starts_with("SHELL")
        || key.starts_with("TERM")
        || key.starts_with("LANG")
        || key.starts_with("LC_")
        || key.starts_with("SSH_")
        || key.starts_with("TMPDIR")
        || key.starts_with("XPC_")
        || key.starts_with("__CF")
        || key.starts_with("SECURITYSESSIONID")
        || key.starts_with("LOGNAME")
        || key.starts_with("PWD")
        || key.starts_with("OLDPWD")
        || key.starts_with("SHLVL")
        || key.starts_with("DISPLAY")
        || key.starts_with("COLORTERM")
        || key.starts_with("DBUS_")
        || key.starts_with("GNOME_")
        || key.starts_with("GTK_")
        || key.starts_with("QT_")
        || key.starts_with("XDG_")
        || key.starts_with("WINDOWID")
        || key.starts_with("EDITOR")
        || key.starts_with("VISUAL")
        || key.starts_with("PAGER")
        || key.starts_with("MANPATH")
        || key.starts_with("INFOPATH")
        || key.starts_with("PS1")
        || key.starts_with("PROMPT")
        || key == "TERM_PROGRAM"
        || key == "TERM_PROGRAM_VERSION"
        || key == "_"
        || key == "ORIGINAL_XDG_CURRENT_DESKTOP"
        || key == "Apple_PubSub_Socket_Render"
        || key == "COMMAND_MODE"
        || key == "MallocNanoZone"
        || key == "VSCODE_IPC_HOOK"
        || key == "ZDOTDIR"
        || key == "BROWSER"
        || key == "LSCOLORS"
        || key == "CLICOLOR"
        || key == "LESS"
        || key == "LS_COLORS"
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

    println!("  {} API:       http://localhost:{}", "→".blue(), port);
    println!("  {} Dashboard: http://devhub.localhost", "→".blue(),);
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

/// Scan directories for projects
async fn cmd_scan(
    registry: &mut Registry,
    paths: Vec<PathBuf>,
    max_depth: usize,
    auto_register: bool,
    dry_run: bool,
) -> Result<()> {
    // Default paths if none provided
    let scan_paths = if paths.is_empty() {
        let home = directories::BaseDirs::new()
            .map(|d| d.home_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("~"));
        let moinsen = home.join("work/moinsen");
        vec![
            moinsen.join("ideas"),
            moinsen.join("opensource"),
            moinsen.join("apps"),
        ]
    } else {
        paths
    };

    println!("{}", "Scanning for projects...".bold());
    println!();

    let mut discovered_projects = Vec::new();
    let mut already_registered = 0;
    let mut new_projects = 0;

    for base_path in &scan_paths {
        if !base_path.exists() {
            println!(
                "  {} {} (not found)",
                "!".yellow(),
                base_path.display().to_string().dimmed()
            );
            continue;
        }

        println!(
            "  {} Scanning {}...",
            "→".blue(),
            base_path.display().to_string().cyan()
        );

        // Scan subdirectories
        scan_directory(
            base_path,
            0,
            max_depth,
            registry,
            &mut discovered_projects,
            &mut already_registered,
        )?;
    }

    println!();
    println!("{}", "Scan Results:".bold());
    println!();

    if discovered_projects.is_empty() {
        println!("  No new projects found.");
        if already_registered > 0 {
            println!(
                "  {} projects already registered.",
                already_registered.to_string().cyan()
            );
        }
        return Ok(());
    }

    // Group by type
    let mut by_type: std::collections::HashMap<
        String,
        Vec<&(PathBuf, discovery::DiscoveredProject)>,
    > = std::collections::HashMap::new();
    for item in &discovered_projects {
        by_type
            .entry(item.1.project_type.to_string())
            .or_default()
            .push(item);
    }

    for (project_type, projects) in &by_type {
        println!(
            "  {} {}:",
            project_type.yellow(),
            format!("({})", projects.len()).dimmed()
        );
        for (_path, discovered) in projects {
            let services_str = if discovered.services.is_empty() {
                "no services".dimmed().to_string()
            } else {
                discovered
                    .services
                    .iter()
                    .map(|s| {
                        if let Some(port) = s.port {
                            format!("{}:{}", s.name, port)
                        } else {
                            s.name.clone()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            println!(
                "    {} {} - {}",
                "•".dimmed(),
                discovered.name.cyan(),
                services_str.dimmed()
            );
        }
        println!();
    }

    println!(
        "Found {} new projects ({} already registered)",
        discovered_projects.len().to_string().green(),
        already_registered.to_string().cyan()
    );

    if dry_run {
        println!();
        println!("{} Dry run - no changes made", "!".yellow());
        return Ok(());
    }

    if auto_register {
        println!();
        println!("{} Auto-registering projects...", "→".blue());
        println!();

        for (path, discovered) in &discovered_projects {
            // Generate devhub.toml if it doesn't exist
            let manifest_path = path.join("devhub.toml");
            if !manifest_path.exists() {
                let manifest = discovery::to_manifest(discovered);
                let content = toml::to_string_pretty(&manifest)?;
                std::fs::write(&manifest_path, &content)?;
            }

            // Register the project
            registry.register(&discovered.name, path)?;
            new_projects += 1;
            println!("  {} Registered {}", "✓".green(), discovered.name.cyan());
        }

        registry.save()?;
        println!();
        println!(
            "{} Registered {} new projects",
            "✓".green(),
            new_projects.to_string().cyan()
        );
    } else {
        println!();
        println!(
            "Run with {} to register these projects.",
            "--auto-register".cyan()
        );
    }

    Ok(())
}

/// Recursively scan a directory for projects
fn scan_directory(
    path: &PathBuf,
    current_depth: usize,
    max_depth: usize,
    registry: &Registry,
    discovered: &mut Vec<(PathBuf, discovery::DiscoveredProject)>,
    already_registered: &mut usize,
) -> Result<()> {
    if current_depth > max_depth {
        return Ok(());
    }

    // Check if this is a project directory
    let is_project = path.join("Cargo.toml").exists()
        || path.join("package.json").exists()
        || path.join("pubspec.yaml").exists()
        || path.join("pyproject.toml").exists()
        || path.join("go.mod").exists()
        || path.join("docker-compose.yml").exists()
        || path.join("docker-compose.yaml").exists();

    if is_project {
        // Check if already registered
        if registry.find_by_path(path).is_some() {
            *already_registered += 1;
            return Ok(());
        }

        // Discover project details
        if let Ok(project) = discovery::discover_project(path) {
            discovered.push((path.clone(), project));
        }
        return Ok(());
    }

    // Recurse into subdirectories
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                // Skip hidden directories and common non-project dirs
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with('.')
                    || name_str == "node_modules"
                    || name_str == "target"
                    || name_str == "build"
                    || name_str == "dist"
                    || name_str == "__pycache__"
                    || name_str == ".git"
                {
                    continue;
                }
                scan_directory(
                    &entry_path,
                    current_depth + 1,
                    max_depth,
                    registry,
                    discovered,
                    already_registered,
                )?;
            }
        }
    }

    Ok(())
}

/// Show port allocations
async fn cmd_ports(
    registry: &Registry,
    check_conflicts: bool,
    show_resolutions: bool,
) -> Result<()> {
    let projects = registry.list();

    if projects.is_empty() {
        println!("No projects registered.");
        return Ok(());
    }

    // Use the ports module to collect allocations
    let port_map = ports::collect_allocated_ports(registry);

    // Sort by port
    let mut ports_sorted: Vec<_> = port_map.iter().collect();
    ports_sorted.sort_by_key(|(port, _)| *port);

    println!("{}", "Port Allocations:".bold());
    println!();

    // Show port ranges info
    println!("{}", "Port Ranges:".dimmed());
    for range in ports::PORT_RANGES {
        println!(
            "  {} {}-{}: {}",
            "•".dimmed(),
            range.range.start(),
            range.range.end(),
            range.description.dimmed()
        );
    }
    println!();

    // Group by range
    println!("  {} (3000-3099)", "Web Frontends".cyan());
    for (port, services) in ports_sorted
        .iter()
        .filter(|(p, _)| **p >= 3000 && **p < 3100)
    {
        print_port_line(**port, services, check_conflicts);
    }
    println!();

    println!("  {} (8000-8099)", "APIs".cyan());
    for (port, services) in ports_sorted
        .iter()
        .filter(|(p, _)| **p >= 8000 && **p < 8100)
    {
        print_port_line(**port, services, check_conflicts);
    }
    println!();

    println!("  {} (other)", "Other".cyan());
    for (port, services) in ports_sorted
        .iter()
        .filter(|(p, _)| !(**p >= 3000 && **p < 3100 || **p >= 8000 && **p < 8100))
    {
        print_port_line(**port, services, check_conflicts);
    }

    // Find and display conflicts
    let conflicts = ports::find_conflicts(&port_map);

    if check_conflicts || show_resolutions {
        println!();
        if conflicts.is_empty() {
            println!("{} No port conflicts detected", "✓".green());
        } else {
            println!("{} {} port conflicts detected:", "!".red(), conflicts.len());
            for (port, services) in &conflicts {
                println!(
                    "    Port {}: {}",
                    port.to_string().red(),
                    services
                        .iter()
                        .map(|(p, s)| format!("{}/{}", p, s))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }
    }

    // Show resolution suggestions if requested
    if show_resolutions && !conflicts.is_empty() {
        println!();
        println!("{}", "Suggested Resolutions:".bold());
        println!();

        let resolutions = ports::suggest_alternatives(&conflicts, &port_map);

        if resolutions.is_empty() {
            println!("  {} No automatic resolutions available", "!".yellow());
        } else {
            for resolution in &resolutions {
                println!(
                    "  {} Move {}/{} from port {} to port {}",
                    "→".blue(),
                    resolution.project.cyan(),
                    resolution.service,
                    resolution.original_port.to_string().red(),
                    resolution.suggested_port.to_string().green()
                );
                println!(
                    "    (keeps {}/{} on port {})",
                    resolution.keeper_project.dimmed(),
                    resolution.keeper_service.dimmed(),
                    resolution.original_port.to_string().dimmed()
                );
                println!();
            }

            println!("{}", "To apply:".dimmed());
            for resolution in &resolutions {
                println!("  {}", resolution.fix_command().dimmed());
            }
        }
    }

    Ok(())
}

fn print_port_line(port: u16, services: &[(String, String)], check_conflicts: bool) {
    let in_use = process::is_port_in_use(port);
    let status_icon = if in_use {
        "●".green()
    } else {
        "○".dimmed()
    };
    let conflict = services.len() > 1;
    let conflict_icon = if conflict && check_conflicts {
        " ⚠".red().to_string()
    } else {
        String::new()
    };

    let services_str = services
        .iter()
        .map(|(p, s)| format!("{}/{}", p, s))
        .collect::<Vec<_>>()
        .join(", ");

    println!(
        "    {} {} → {}{}",
        status_icon,
        format!("{:5}", port).cyan(),
        services_str,
        conflict_icon
    );
}

/// Generate shell completions
fn cmd_completions(shell: Shell) -> Result<()> {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    generate(shell, &mut cmd, name, &mut io::stdout());
    Ok(())
}

/// Open a project in the browser
async fn cmd_open(registry: &Registry, project: &str, service: Option<String>) -> Result<()> {
    let entry = registry
        .get(project)
        .ok_or_else(|| anyhow::anyhow!("Project '{}' not found", project))?;

    let manifest_path = entry.path.join("devhub.toml");
    let manifest = manifest::Manifest::load(&manifest_path)?;

    // Find the service to open
    let svc = if let Some(ref service_name) = service {
        manifest
            .services
            .iter()
            .find(|s| s.name == *service_name)
            .ok_or_else(|| anyhow::anyhow!("Service '{}' not found", service_name))?
    } else {
        manifest
            .services
            .iter()
            .find(|s| s.main)
            .or_else(|| manifest.services.first())
            .ok_or_else(|| anyhow::anyhow!("No services found"))?
    };

    // Build the URL
    let url = if svc.main {
        format!("http://{}.localhost", project)
    } else {
        let subdomain = svc.subdomain.as_ref().unwrap_or(&svc.name);
        format!("http://{}.{}.localhost", subdomain, project)
    };

    // Check if running
    if !process::is_port_in_use(svc.port) {
        println!(
            "{} Service '{}' is not running (port {})",
            "!".yellow(),
            svc.name,
            svc.port
        );
        println!("  Run 'devhub start {}' first", project);
        return Ok(());
    }

    println!("{} Opening {} in browser...", "→".blue(), url.cyan());

    // Open in default browser
    #[cfg(target_os = "macos")]
    {
        tokio::process::Command::new("open")
            .arg(&url)
            .output()
            .await?;
    }

    #[cfg(target_os = "linux")]
    {
        tokio::process::Command::new("xdg-open")
            .arg(&url)
            .output()
            .await?;
    }

    #[cfg(target_os = "windows")]
    {
        tokio::process::Command::new("cmd")
            .args(["/c", "start", &url])
            .output()
            .await?;
    }

    Ok(())
}

/// Open a project in VS Code
async fn cmd_code(registry: &Registry, project: &str) -> Result<()> {
    let entry = registry
        .get(project)
        .ok_or_else(|| anyhow::anyhow!("Project '{}' not found", project))?;

    println!("{} Opening {} in VS Code...", "→".blue(), project.cyan());

    tokio::process::Command::new("code")
        .arg(&entry.path)
        .output()
        .await?;

    Ok(())
}

/// Print a project's path (for shell cd integration)
async fn cmd_path(registry: &Registry, project: &str) -> Result<()> {
    let entry = registry
        .get(project)
        .ok_or_else(|| anyhow::anyhow!("Project '{}' not found", project))?;

    // Just print the path - no formatting, so it can be used with: cd $(devhub path myproject)
    println!("{}", entry.path.display());

    Ok(())
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

/// Search for projects with fuzzy matching
async fn cmd_search(registry: &Registry, query: &str, limit: usize) -> Result<()> {
    let results = registry.fuzzy_search(query);

    if results.is_empty() {
        println!("No projects found matching '{}'", query);
        return Ok(());
    }

    println!("{}", "Search Results:".bold());
    println!();

    for (name, entry, score) in results.into_iter().take(limit) {
        let running = check_any_service_running(&entry.path);
        let status_icon = if running {
            "●".green()
        } else {
            "○".dimmed()
        };
        let fav_icon = if entry.favorite { "★ " } else { "" };

        println!(
            "  {} {}{} {} (score: {})",
            status_icon,
            fav_icon.yellow(),
            name.cyan(),
            entry.path.display().to_string().dimmed(),
            score.to_string().dimmed()
        );
    }

    Ok(())
}

/// Check if any service is running for a project
fn check_any_service_running(path: &std::path::Path) -> bool {
    let manifest_path = path.join("devhub.toml");
    if let Ok(manifest) = manifest::Manifest::load(&manifest_path) {
        for service in &manifest.services {
            if process::is_port_in_use(service.port) {
                return true;
            }
        }
    }
    false
}

/// Manage favorite projects
async fn cmd_fav(registry: &mut Registry, action: FavAction) -> Result<()> {
    match action {
        FavAction::Add { project } => {
            if registry.set_favorite(&project, true) {
                registry.save()?;
                println!("{} Added {} to favorites", "★".yellow(), project.cyan());
            } else {
                println!("{} Project '{}' not found", "!".red(), project);
            }
        }
        FavAction::Remove { project } => {
            if registry.set_favorite(&project, false) {
                registry.save()?;
                println!("{} Removed {} from favorites", "☆".dimmed(), project.cyan());
            } else {
                println!("{} Project '{}' not found", "!".red(), project);
            }
        }
        FavAction::List => {
            let favorites = registry.favorites();
            if favorites.is_empty() {
                println!("No favorite projects. Add one with: devhub fav add <project>");
                return Ok(());
            }

            println!("{}", "Favorite Projects:".bold());
            println!();

            for (name, entry) in favorites {
                let running = check_any_service_running(&entry.path);
                let status_icon = if running {
                    "●".green()
                } else {
                    "○".dimmed()
                };

                println!(
                    "  {} {} {}",
                    status_icon,
                    name.cyan(),
                    entry.path.display().to_string().dimmed()
                );
            }
        }
        FavAction::Toggle { project } => {
            if let Some(is_fav) = registry.toggle_favorite(&project) {
                registry.save()?;
                if is_fav {
                    println!("{} Added {} to favorites", "★".yellow(), project.cyan());
                } else {
                    println!("{} Removed {} from favorites", "☆".dimmed(), project.cyan());
                }
            } else {
                println!("{} Project '{}' not found", "!".red(), project);
            }
        }
    }

    Ok(())
}

/// Show recently used projects
async fn cmd_recent(registry: &Registry, limit: usize) -> Result<()> {
    let recent = registry.recent(limit);

    if recent.is_empty() {
        println!("No recent projects. Start or open a project to track usage.");
        return Ok(());
    }

    println!("{}", "Recent Projects:".bold());
    println!();

    for (name, entry) in recent {
        let running = check_any_service_running(&entry.path);
        let status_icon = if running {
            "●".green()
        } else {
            "○".dimmed()
        };
        let fav_icon = if entry.favorite { "★ " } else { "" };

        let time_ago = if let Some(last_used) = entry.last_used {
            format_time_ago(last_used)
        } else {
            "never".to_string()
        };

        println!(
            "  {} {}{} {} ({})",
            status_icon,
            fav_icon.yellow(),
            name.cyan(),
            entry.path.display().to_string().dimmed(),
            time_ago.dimmed()
        );
    }

    Ok(())
}

/// Format a timestamp as "X ago"
fn format_time_ago(dt: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(dt);

    if duration.num_days() > 0 {
        format!("{}d ago", duration.num_days())
    } else if duration.num_hours() > 0 {
        format!("{}h ago", duration.num_hours())
    } else if duration.num_minutes() > 0 {
        format!("{}m ago", duration.num_minutes())
    } else {
        "just now".to_string()
    }
}
