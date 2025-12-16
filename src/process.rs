use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;

use crate::manifest::{Manifest, Service, ServiceType};

/// Load environment variables from a .env file
/// Returns a HashMap of key-value pairs
fn load_env_file(path: &Path) -> HashMap<String, String> {
    let mut vars = HashMap::new();

    if !path.exists() {
        return vars;
    }

    // Use dotenvy to parse the file
    if let Ok(iter) = dotenvy::from_path_iter(path) {
        for item in iter.flatten() {
            vars.insert(item.0, item.1);
        }
    }

    vars
}

/// Interpolate variables in environment values
/// Supports $VAR and ${VAR} syntax
fn interpolate_env_vars(env: &mut HashMap<String, String>) {
    // Collect keys first to avoid borrow issues
    let keys: Vec<String> = env.keys().cloned().collect();

    for key in keys {
        if let Some(value) = env.get(&key).cloned() {
            let interpolated = interpolate_value(&value, env);
            env.insert(key, interpolated);
        }
    }
}

/// Interpolate a single value, replacing $VAR and ${VAR} with their values
fn interpolate_value(value: &str, env: &HashMap<String, String>) -> String {
    let mut result = value.to_string();

    // Handle ${VAR} syntax first (more specific)
    let re_braces = regex::Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap();
    result = re_braces
        .replace_all(&result, |caps: &regex::Captures| {
            let var_name = &caps[1];
            env.get(var_name)
                .cloned()
                .or_else(|| std::env::var(var_name).ok())
                .unwrap_or_default()
        })
        .to_string();

    // Handle $VAR syntax (without braces)
    let re_simple = regex::Regex::new(r"\$([A-Za-z_][A-Za-z0-9_]*)").unwrap();
    result = re_simple
        .replace_all(&result, |caps: &regex::Captures| {
            let var_name = &caps[1];
            env.get(var_name)
                .cloned()
                .or_else(|| std::env::var(var_name).ok())
                .unwrap_or_default()
        })
        .to_string();

    result
}

/// Load all environment variables for a service
/// Priority (highest to lowest):
/// 1. Service-specific env_file (if specified)
/// 2. Service cwd/.env.local (if cwd exists)
/// 3. Service cwd/.env (if cwd exists)
/// 4. Project root .env.local
/// 5. Project root .env
/// 6. Project env_files (from manifest)
/// 7. Manifest [environment] section
/// 8. Service env = {} section
pub fn load_service_environment(
    project_path: &Path,
    manifest: &Manifest,
    service: &Service,
) -> HashMap<String, String> {
    let mut env: HashMap<String, String> = HashMap::new();

    // 1. Start with system environment
    env.extend(std::env::vars());

    // 2. Load project-level env_files (from manifest)
    for env_file in &manifest.project.env_files {
        let path = project_path.join(env_file);
        let file_env = load_env_file(&path);
        env.extend(file_env);
    }

    // 3. Load project root .env (if not in env_files)
    if !manifest.project.env_files.iter().any(|f| f == ".env") {
        let path = project_path.join(".env");
        let file_env = load_env_file(&path);
        env.extend(file_env);
    }

    // 4. Load project root .env.local (if not in env_files)
    if !manifest.project.env_files.iter().any(|f| f == ".env.local") {
        let path = project_path.join(".env.local");
        let file_env = load_env_file(&path);
        env.extend(file_env);
    }

    // 5. Add manifest [environment] section
    env.extend(manifest.environment.clone());

    // 6. Load service cwd .env files (if cwd is set)
    if let Some(ref cwd) = service.cwd {
        let service_dir = project_path.join(cwd);

        // .env in service directory
        let path = service_dir.join(".env");
        let file_env = load_env_file(&path);
        env.extend(file_env);

        // .env.local in service directory
        let path = service_dir.join(".env.local");
        let file_env = load_env_file(&path);
        env.extend(file_env);
    }

    // 7. Load service-specific env_file (if specified)
    if let Some(ref env_file) = service.env_file {
        let path = project_path.join(env_file);
        let file_env = load_env_file(&path);
        env.extend(file_env);
    }

    // 8. Add service env = {} section (highest priority)
    env.extend(service.env.clone());

    // 9. Interpolate variables
    interpolate_env_vars(&mut env);

    env
}

/// Get resolved environment for display (devhub env command)
pub fn get_resolved_environment(
    project_path: &Path,
    manifest: &Manifest,
    service: Option<&Service>,
) -> HashMap<String, String> {
    if let Some(svc) = service {
        load_service_environment(project_path, manifest, svc)
    } else {
        // Return project-level environment only
        let mut env: HashMap<String, String> = HashMap::new();

        // Load project env_files
        for env_file in &manifest.project.env_files {
            let path = project_path.join(env_file);
            env.extend(load_env_file(&path));
        }

        // Load .env and .env.local
        env.extend(load_env_file(&project_path.join(".env")));
        env.extend(load_env_file(&project_path.join(".env.local")));

        // Add manifest environment
        env.extend(manifest.environment.clone());

        interpolate_env_vars(&mut env);
        env
    }
}

/// Get the log directory for a project
fn get_log_dir(project_name: &str) -> Result<PathBuf> {
    let dirs = directories::ProjectDirs::from("com", "moinsen", "devhub")
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
    let log_dir = dirs.data_dir().join("logs").join(project_name);
    fs::create_dir_all(&log_dir)?;
    Ok(log_dir)
}

/// Check if a port is in use
///
/// Checks both IPv4 (127.0.0.1) and IPv6 (::1) since modern servers
/// like Next.js often listen on IPv6 by default.
pub fn is_port_in_use(port: u16) -> bool {
    // Try to connect to the port - this is more reliable than bind-based detection
    // because it works regardless of which IP family the server is using
    let ipv4_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
    let ipv6_addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), port);

    // Use connection-based check with a short timeout
    let timeout = std::time::Duration::from_millis(100);

    // Check IPv4
    if TcpStream::connect_timeout(&ipv4_addr, timeout).is_ok() {
        return true;
    }

    // Check IPv6
    if TcpStream::connect_timeout(&ipv6_addr, timeout).is_ok() {
        return true;
    }

    false
}

/// Check if PM2 is available
fn is_pm2_available() -> bool {
    std::process::Command::new("pm2")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Start a service with improved startup detection
pub async fn start_service(
    project_name: &str,
    project_path: &Path,
    service: &Service,
    manifest: &Manifest,
) -> Result<()> {
    // Handle Docker Compose services separately
    if service.service_type == ServiceType::DockerCompose {
        return start_docker_compose_service(project_name, project_path, service).await;
    }

    // Use PM2 for Node.js services if available
    if service.service_type == ServiceType::Node && is_pm2_available() {
        return start_pm2_service(project_name, project_path, service, manifest).await;
    }

    // Check if already running
    if is_port_in_use(service.port) {
        println!(
            "  {} {} already running on port {}",
            "•".yellow(),
            service.name,
            service.port
        );
        return Ok(());
    }

    println!(
        "  {} Starting {} on port {}...",
        "→".blue(),
        service.name.yellow(),
        service.port
    );

    // Determine working directory
    let cwd = if let Some(ref rel_cwd) = service.cwd {
        project_path.join(rel_cwd)
    } else {
        project_path.to_path_buf()
    };

    // Build environment from .env files and manifest settings
    let env = load_service_environment(project_path, manifest, service);

    // Set up log files
    let log_dir = get_log_dir(project_name)?;
    let stdout_path = log_dir.join(format!("{}.out.log", service.name));
    let stderr_path = log_dir.join(format!("{}.err.log", service.name));

    let stdout_file = File::create(&stdout_path)?;
    let stderr_file = File::create(&stderr_path)?;

    // Parse command - handle shell commands properly
    let (program, args) = if service.command.contains("&&") || service.command.contains("|") {
        // Complex command, use shell
        (
            "sh".to_string(),
            vec!["-c".to_string(), service.command.clone()],
        )
    } else {
        let parts: Vec<String> = service
            .command
            .split_whitespace()
            .map(String::from)
            .collect();
        if parts.is_empty() {
            anyhow::bail!("Empty command for service {}", service.name);
        }
        (parts[0].clone(), parts[1..].to_vec())
    };

    // Spawn the process
    let mut cmd = Command::new(&program);
    cmd.args(&args)
        .current_dir(&cwd)
        .envs(env)
        .stdout(stdout_file)
        .stderr(stderr_file)
        .stdin(Stdio::null());

    // Spawn and detach
    let child = cmd.spawn()?;

    // Store PID for later
    let pid = child.id();
    tracing::info!("Started service {} with PID {:?}", service.name, pid);

    // Write PID file
    let pid_path = log_dir.join(format!("{}.pid", service.name));
    if let Some(p) = pid {
        fs::write(&pid_path, p.to_string())?;
    }

    // Wait for startup with progress indication
    let timeout_secs = match service.service_type {
        ServiceType::RustBinary => 120, // Rust needs compile time
        ServiceType::Node => 30,        // Node is usually fast
        ServiceType::Python => 15,
        ServiceType::Go => 30,
        ServiceType::DockerCompose => 60,
        ServiceType::Shell => 10,
    };

    let started = wait_for_port_with_progress(service.port, &service.name, timeout_secs).await;

    if started {
        // Run health check if configured
        if let Some(ref health_path) = service.health_check {
            print!("    Health check");
            let healthy = check_health_endpoint(service.port, health_path, 10).await;
            if healthy {
                println!(" {}", "✓".green());
            } else {
                println!(" {} (endpoint not responding)", "!".yellow());
            }
        }

        println!(
            "  {} {} started (port {})",
            "✓".green(),
            service.name,
            service.port
        );
    } else {
        // Check if process is still running by checking stderr log
        let maybe_error = check_for_startup_error(&stderr_path);

        if let Some(error) = maybe_error {
            println!(
                "  {} {} failed to start: {}",
                "✗".red(),
                service.name,
                error
            );
        } else {
            println!(
                "  {} {} still starting (check logs: {})",
                "⋯".yellow(),
                service.name,
                stderr_path.display()
            );
        }
    }

    Ok(())
}

/// Start a Docker Compose service
async fn start_docker_compose_service(
    project_name: &str,
    project_path: &Path,
    service: &Service,
) -> Result<()> {
    // Determine compose file path
    let compose_file = if service.command.contains("-f ") {
        // Extract compose file from command
        service
            .command
            .split("-f ")
            .nth(1)
            .and_then(|s| s.split_whitespace().next())
            .unwrap_or("docker-compose.yml")
            .to_string()
    } else if project_path.join("docker-compose.yml").exists() {
        "docker-compose.yml".to_string()
    } else {
        "docker-compose.yaml".to_string()
    };

    let compose_path = project_path.join(&compose_file);
    if !compose_path.exists() {
        anyhow::bail!("Docker Compose file not found: {}", compose_path.display());
    }

    println!(
        "  {} Starting Docker Compose: {}...",
        "→".blue(),
        service.name.yellow()
    );

    // Check if container is already running
    let check_output = Command::new("docker")
        .args(["compose", "-f", &compose_file, "ps", "-q"])
        .current_dir(project_path)
        .output()
        .await?;

    let running_containers = String::from_utf8_lossy(&check_output.stdout);
    if !running_containers.trim().is_empty() {
        println!("  {} {} already running", "•".yellow(), service.name);
        return Ok(());
    }

    // Start the service with docker compose up -d
    let log_dir = get_log_dir(project_name)?;
    let stdout_path = log_dir.join(format!("{}.out.log", service.name));
    let stderr_path = log_dir.join(format!("{}.err.log", service.name));

    let stdout_file = File::create(&stdout_path)?;
    let stderr_file = File::create(&stderr_path)?;

    let output = Command::new("docker")
        .args(["compose", "-f", &compose_file, "up", "-d"])
        .current_dir(project_path)
        .stdout(stdout_file)
        .stderr(stderr_file)
        .output()
        .await?;

    if output.status.success() {
        // Wait a moment for containers to start
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Check if port is now in use (if port is specified)
        if service.port > 0 && is_port_in_use(service.port) {
            println!(
                "  {} {} started (port {})",
                "✓".green(),
                service.name,
                service.port
            );
        } else if service.port > 0 {
            println!(
                "  {} {} started (port {} may not be exposed yet)",
                "✓".green(),
                service.name,
                service.port
            );
        } else {
            println!("  {} {} started", "✓".green(), service.name);
        }
    } else {
        let stderr = fs::read_to_string(&stderr_path).unwrap_or_default();
        println!(
            "  {} {} failed to start: {}",
            "✗".red(),
            service.name,
            stderr.lines().last().unwrap_or("unknown error")
        );
    }

    Ok(())
}

/// Check a health endpoint
async fn check_health_endpoint(port: u16, path: &str, timeout_secs: u64) -> bool {
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(timeout_secs);
    let url = format!("http://127.0.0.1:{}{}", port, path);

    loop {
        if start.elapsed() > timeout {
            return false;
        }

        match reqwest::get(&url).await {
            Ok(resp) if resp.status().is_success() => return true,
            _ => {
                print!(".");
                use std::io::Write;
                let _ = std::io::stdout().flush();
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        }
    }
}

/// Wait for port with progress dots
async fn wait_for_port_with_progress(port: u16, _service_name: &str, timeout_secs: u64) -> bool {
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(timeout_secs);
    let mut dots = 0;
    let max_dots = 30;

    // Print initial waiting message
    print!("    Waiting");

    loop {
        if start.elapsed() > timeout {
            println!(" timeout after {}s", timeout_secs);
            return false;
        }

        if is_port_in_use(port) {
            // Clear the dots and print success
            println!(" ready!");
            return true;
        }

        // Print progress dot every second
        if dots < max_dots {
            print!(".");
            use std::io::Write;
            let _ = std::io::stdout().flush();
            dots += 1;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

/// Check stderr log for obvious errors
fn check_for_startup_error(stderr_path: &Path) -> Option<String> {
    let file = File::open(stderr_path).ok()?;
    let reader = BufReader::new(file);

    // Look for common error patterns in the last few lines
    let lines: Vec<String> = reader.lines().map_while(Result::ok).collect();
    let last_lines: Vec<&String> = lines.iter().rev().take(10).collect();

    for line in last_lines {
        let lower = line.to_lowercase();
        if lower.contains("error") && !lower.contains("error[e") {
            // Skip Rust compilation progress
            return Some(line.clone());
        }
        if lower.contains("fatal") || lower.contains("panic") {
            return Some(line.clone());
        }
        if lower.contains("eaddrinuse") || lower.contains("address already in use") {
            return Some("Port already in use".to_string());
        }
        if lower.contains("enoent") || lower.contains("not found") {
            return Some(line.clone());
        }
    }

    None
}

/// Stop a service by killing the process on its port
pub async fn stop_service(project_name: &str, service: &Service) -> Result<()> {
    // Handle Docker Compose services separately
    if service.service_type == ServiceType::DockerCompose {
        return stop_docker_compose_service(project_name, service).await;
    }

    // Try PM2 first for Node services
    if service.service_type == ServiceType::Node && is_pm2_available() {
        let pm2_result = stop_pm2_service(project_name, service).await;
        if pm2_result.is_ok() && !is_port_in_use(service.port) {
            return Ok(());
        }
        // Fall through to regular stop if PM2 didn't work
    }

    if !is_port_in_use(service.port) {
        println!("  {} {} not running", "•".dimmed(), service.name);
        return Ok(());
    }

    println!(
        "  {} Stopping {} (port {})...",
        "→".blue(),
        service.name.yellow(),
        service.port
    );

    // Find and kill process using the port
    // On macOS/Linux, we can use lsof
    let output = Command::new("lsof")
        .args(["-t", "-i", &format!(":{}", service.port)])
        .output()
        .await?;

    let pids: Vec<i32> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.trim().parse().ok())
        .collect();

    for pid in pids {
        tracing::info!("Killing PID {} for service {}", pid, service.name);

        // Send SIGTERM first
        #[cfg(unix)]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;
            let _ = kill(Pid::from_raw(pid), Signal::SIGTERM);
        }

        #[cfg(not(unix))]
        {
            let _ = Command::new("kill").args([&pid.to_string()]).output().await;
        }
    }

    // Wait for port to be released
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // Clean up PID file
    if let Ok(log_dir) = get_log_dir(project_name) {
        let pid_path = log_dir.join(format!("{}.pid", service.name));
        let _ = fs::remove_file(pid_path);
    }

    if !is_port_in_use(service.port) {
        println!("  {} {} stopped", "✓".green(), service.name);
    } else {
        println!(
            "  {} {} may still be running (port {} still in use)",
            "!".yellow(),
            service.name,
            service.port
        );
    }

    Ok(())
}

/// Stop a Docker Compose service
async fn stop_docker_compose_service(_project_name: &str, service: &Service) -> Result<()> {
    println!(
        "  {} Stopping Docker Compose: {}...",
        "→".blue(),
        service.name.yellow()
    );

    // Use docker compose down
    let output = Command::new("docker")
        .args(["compose", "down"])
        .output()
        .await?;

    if output.status.success() {
        println!("  {} {} stopped", "✓".green(), service.name);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!(
            "  {} {} may have failed to stop: {}",
            "!".yellow(),
            service.name,
            stderr.lines().last().unwrap_or("unknown error")
        );
    }

    Ok(())
}

/// Start a Node.js service using PM2
async fn start_pm2_service(
    project_name: &str,
    project_path: &Path,
    service: &Service,
    manifest: &Manifest,
) -> Result<()> {
    let pm2_name = format!("{}:{}", project_name, service.name);

    // Check if already running in PM2
    let list_output = Command::new("pm2").args(["jlist"]).output().await?;

    let list_json = String::from_utf8_lossy(&list_output.stdout);
    if list_json.contains(&format!("\"name\":\"{}\"", pm2_name)) {
        // Check if it's actually running
        if list_json.contains(&format!("\"name\":\"{}\",\"pm2_env\":{{", pm2_name)) {
            println!("  {} {} already managed by PM2", "•".yellow(), service.name);

            // Check if port is in use
            if is_port_in_use(service.port) {
                return Ok(());
            }

            // Restart if not responding
            let _ = Command::new("pm2")
                .args(["restart", &pm2_name])
                .output()
                .await;
        }
    }

    println!(
        "  {} Starting {} via PM2 on port {}...",
        "→".blue(),
        service.name.yellow(),
        service.port
    );

    // Build environment from .env files and manifest settings
    let env = load_service_environment(project_path, manifest, service);

    // Convert to PM2 environment string format
    let mut env_args = Vec::new();
    for (key, value) in &env {
        // Skip system env vars to keep PM2 command clean - only pass custom vars
        if !key.starts_with("PATH")
            && !key.starts_with("HOME")
            && !key.starts_with("USER")
            && !key.starts_with("SHELL")
            && !key.starts_with("TERM")
            && !key.starts_with("LANG")
            && !key.starts_with("LC_")
            && !key.starts_with("SSH_")
            && !key.starts_with("TMPDIR")
            && !key.starts_with("XPC_")
            && !key.starts_with("__CF")
            && !key.starts_with("SECURITYSESSIONID")
        {
            env_args.push(format!("{}={}", key, value));
        }
    }

    // Parse the npm/pnpm/yarn command
    let script = if service.command.starts_with("npm run ") {
        service.command.trim_start_matches("npm run ").to_string()
    } else if service.command.starts_with("pnpm ") {
        service.command.trim_start_matches("pnpm ").to_string()
    } else if service.command.starts_with("yarn ") {
        service.command.trim_start_matches("yarn ").to_string()
    } else {
        service.command.clone()
    };

    // Determine interpreter based on command
    let (interpreter, interpreter_args) = if service.command.starts_with("npm") {
        ("npm", vec!["run".to_string(), script])
    } else if service.command.starts_with("pnpm") {
        ("pnpm", vec![script])
    } else if service.command.starts_with("yarn") {
        ("yarn", vec![script])
    } else if service.command.starts_with("bun") {
        ("bun", vec!["run".to_string(), script])
    } else {
        ("node", vec![script])
    };

    // Start with PM2
    let mut cmd = Command::new("pm2");
    cmd.args([
        "start",
        interpreter,
        "--name",
        &pm2_name,
        "--cwd",
        project_path.to_str().unwrap_or("."),
        "--",
    ])
    .args(&interpreter_args)
    .current_dir(project_path);

    let output = cmd.output().await?;

    if output.status.success() {
        // Wait for port to be in use
        let started = wait_for_port_with_progress(service.port, &service.name, 30).await;

        if started {
            println!(
                "  {} {} started via PM2 (port {})",
                "✓".green(),
                service.name,
                service.port
            );
        } else {
            println!(
                "  {} {} started via PM2 (port {} may not be ready)",
                "✓".green(),
                service.name,
                service.port
            );
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!(
            "  {} {} failed to start via PM2: {}",
            "✗".red(),
            service.name,
            stderr.lines().last().unwrap_or("unknown error")
        );
    }

    Ok(())
}

/// Stop a PM2-managed service
async fn stop_pm2_service(project_name: &str, service: &Service) -> Result<()> {
    let pm2_name = format!("{}:{}", project_name, service.name);

    println!(
        "  {} Stopping {} via PM2...",
        "→".blue(),
        service.name.yellow()
    );

    let output = Command::new("pm2")
        .args(["delete", &pm2_name])
        .output()
        .await?;

    if output.status.success() {
        println!("  {} {} stopped (PM2)", "✓".green(), service.name);
    } else {
        // May not have been PM2 managed, try regular stop
        return Ok(());
    }

    Ok(())
}

/// Wait for a service to be healthy
#[allow(dead_code)]
pub async fn wait_for_healthy(service: &Service, timeout_secs: u64) -> Result<bool> {
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(timeout_secs);

    loop {
        if start.elapsed() > timeout {
            return Ok(false);
        }

        // First check if port is bound
        if !is_port_in_use(service.port) {
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            continue;
        }

        // If there's a health check endpoint, try it
        if let Some(ref health_path) = service.health_check {
            let url = format!("http://127.0.0.1:{}{}", service.port, health_path);
            match reqwest::get(&url).await {
                Ok(resp) if resp.status().is_success() => return Ok(true),
                _ => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                    continue;
                }
            }
        } else {
            // No health check, port being bound is enough
            return Ok(true);
        }
    }
}

/// Read recent log lines for a service
pub fn read_service_logs(
    project_name: &str,
    service_name: &str,
    lines: usize,
    stderr: bool,
) -> Result<Vec<String>> {
    let log_dir = get_log_dir(project_name)?;
    let log_file = if stderr {
        log_dir.join(format!("{}.err.log", service_name))
    } else {
        log_dir.join(format!("{}.out.log", service_name))
    };

    if !log_file.exists() {
        return Ok(vec![]);
    }

    let file = File::open(log_file)?;
    let reader = BufReader::new(file);
    let all_lines: Vec<String> = reader.lines().map_while(Result::ok).collect();

    // Return last N lines
    let start = if all_lines.len() > lines {
        all_lines.len() - lines
    } else {
        0
    };

    Ok(all_lines[start..].to_vec())
}

/// Tail logs in real-time (returns a stream)
pub async fn tail_service_logs(
    project_name: &str,
    service_name: &str,
    stderr: bool,
) -> Result<PathBuf> {
    let log_dir = get_log_dir(project_name)?;
    let log_file = if stderr {
        log_dir.join(format!("{}.err.log", service_name))
    } else {
        log_dir.join(format!("{}.out.log", service_name))
    };

    Ok(log_file)
}
