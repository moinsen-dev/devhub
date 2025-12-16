use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;

use crate::manifest::Service;

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

/// Start a service with improved startup detection
pub async fn start_service(
    project_name: &str,
    project_path: &Path,
    service: &Service,
    global_env: &HashMap<String, String>,
) -> Result<()> {
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

    // Build environment
    let mut env: HashMap<String, String> = std::env::vars().collect();
    env.extend(global_env.clone());
    env.extend(service.env.clone());

    // Set up log files
    let log_dir = get_log_dir(project_name)?;
    let stdout_path = log_dir.join(format!("{}.out.log", service.name));
    let stderr_path = log_dir.join(format!("{}.err.log", service.name));

    let stdout_file = File::create(&stdout_path)?;
    let stderr_file = File::create(&stderr_path)?;

    // Parse command - handle shell commands properly
    let (program, args) = if service.command.contains("&&") || service.command.contains("|") {
        // Complex command, use shell
        ("sh".to_string(), vec!["-c".to_string(), service.command.clone()])
    } else {
        let parts: Vec<String> = service.command.split_whitespace().map(String::from).collect();
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
    tracing::info!(
        "Started service {} with PID {:?}",
        service.name,
        pid
    );

    // Write PID file
    let pid_path = log_dir.join(format!("{}.pid", service.name));
    if let Some(p) = pid {
        fs::write(&pid_path, p.to_string())?;
    }

    // Wait for startup with progress indication
    let timeout_secs = match service.service_type {
        crate::manifest::ServiceType::RustBinary => 120, // Rust needs compile time
        crate::manifest::ServiceType::Node => 30,        // Node is usually fast
        crate::manifest::ServiceType::Python => 15,
        crate::manifest::ServiceType::Go => 30,
        crate::manifest::ServiceType::DockerCompose => 60,
        crate::manifest::ServiceType::Shell => 10,
    };

    let started = wait_for_port_with_progress(service.port, &service.name, timeout_secs).await;

    if started {
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

/// Wait for port with progress dots
async fn wait_for_port_with_progress(port: u16, service_name: &str, timeout_secs: u64) -> bool {
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
    let lines: Vec<String> = reader.lines().filter_map(|l| l.ok()).collect();
    let last_lines: Vec<&String> = lines.iter().rev().take(10).collect();

    for line in last_lines {
        let lower = line.to_lowercase();
        if lower.contains("error") && !lower.contains("error[e") { // Skip Rust compilation progress
            return Some(line.clone());
        }
        if lower.contains("fatal") || lower.contains("panic") {
            return Some(line.clone());
        }
        if lower.contains("eaddrinuse") || lower.contains("address already in use") {
            return Some(format!("Port already in use"));
        }
        if lower.contains("enoent") || lower.contains("not found") {
            return Some(line.clone());
        }
    }

    None
}

/// Stop a service by killing the process on its port
pub async fn stop_service(project_name: &str, service: &Service) -> Result<()> {
    if !is_port_in_use(service.port) {
        println!(
            "  {} {} not running",
            "•".dimmed(),
            service.name
        );
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
            let _ = Command::new("kill")
                .args([&pid.to_string()])
                .output()
                .await;
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
        println!(
            "  {} {} stopped",
            "✓".green(),
            service.name
        );
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

/// Wait for a service to be healthy
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
pub fn read_service_logs(project_name: &str, service_name: &str, lines: usize, stderr: bool) -> Result<Vec<String>> {
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
    let all_lines: Vec<String> = reader.lines().filter_map(|l| l.ok()).collect();

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
