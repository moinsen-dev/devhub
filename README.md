# DevHub

**Multi-project development environment manager** - Start, stop, and monitor all your local development services from one place.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)

## The Problem

Working on multiple projects means juggling multiple terminals:

```bash
# Project A
cd ~/work/project-a && npm run dev        # Terminal 1
cd ~/work/project-a && cargo run          # Terminal 2

# Project B
cd ~/work/project-b && docker compose up  # Terminal 3
cd ~/work/project-b && npm start          # Terminal 4

# Which ports? Which commands? Where's that README again?
```

## The Solution

```bash
devhub start project-a    # Starts all services
devhub start project-b    # Starts all services
devhub status             # See everything at a glance

# Or open http://devhub.localhost for the dashboard
```

## Features

- **One command to rule them all** - `devhub start` launches your entire stack
- **Web dashboard** - Visual status, start/stop buttons, service URLs
- **Auto-discovery** - Detects Rust, Node, Python, Go, Docker projects
- **Reverse proxy integration** - Pretty URLs like `http://myproject.localhost`
- **Unified logs** - All service logs in one place
- **Zero lock-in** - Simple TOML config, works with any stack

## Quick Start

### 1. Install DevHub

```bash
# Clone and build
git clone https://github.com/moinsen-dev/devhub.git
cd devhub
cargo build --release

# Add to PATH
cp target/release/devhub ~/.local/bin/
# or
sudo cp target/release/devhub /usr/local/bin/
```

### 2. Set Up Reverse Proxy (Optional but Recommended)

DevHub works best with a local reverse proxy for pretty URLs:

```bash
# Start the included Caddy proxy
cd proxy
docker compose up -d
```

This gives you:
- `http://projectname.localhost` → Your main service
- `http://api.projectname.localhost` → Your API service
- `http://devhub.localhost` → DevHub dashboard

### 3. Register Your First Project

```bash
cd ~/work/my-project

# Auto-detect project type and create config
devhub discover

# Or manually create devhub.toml
devhub init

# Register with DevHub
devhub register
```

### 4. Start Developing

```bash
devhub start my-project   # Start all services
devhub status             # Check what's running
devhub logs my-project    # View unified logs
devhub stop my-project    # Stop when done
```

## Configuration

Each project gets a `devhub.toml` file:

```toml
[project]
name = "my-app"
description = "My awesome application"
tags = ["rust", "web"]

[[services]]
name = "api"
type = "rust-binary"
command = "cargo run --release"
port = 8080
health_check = "/health"
subdomain = "api"           # → http://api.my-app.localhost

[[services]]
name = "frontend"
type = "node"
command = "npm run dev"
cwd = "frontend"            # Relative working directory
port = 3000
main = true                 # → http://my-app.localhost (no subdomain)
depends_on = ["api"]        # Start order

[environment]
RUST_LOG = "info"
DATABASE_URL = "postgres://localhost/myapp"
```

### Service Types

| Type | Detection | Default Command |
|------|-----------|-----------------|
| `rust-binary` | `Cargo.toml` with `[[bin]]` | `cargo run` |
| `node` | `package.json` | `npm run dev` or `npm start` |
| `python` | `pyproject.toml` or `requirements.txt` | `python -m` or `uvicorn` |
| `go` | `go.mod` | `go run .` |
| `docker-compose` | `docker-compose.yml` | `docker compose up` |
| `shell` | Any | Custom command |

## CLI Reference

```bash
devhub init                    # Create devhub.toml in current directory
devhub discover                # Auto-detect and generate devhub.toml
devhub register [path]         # Register project with DevHub
devhub unregister <name>       # Remove project from registry
devhub list                    # List all registered projects

devhub start [project]         # Start project services
devhub stop [project]          # Stop project services
devhub restart [project]       # Restart project services
devhub status                  # Show status of all projects

devhub logs <project> [svc]    # Stream logs (optionally filter by service)
devhub daemon                  # Run API server for dashboard
```

## Dashboard

The web dashboard provides:

- Project cards with running status
- One-click start/stop/restart
- Service URLs (clickable)
- Real-time status updates

**Access:** http://devhub.localhost (requires proxy) or http://localhost:5173 (dev mode)

### Running the Dashboard

```bash
# Option 1: Development mode
devhub daemon &                           # Start API on :9876
cd devhub-ui && npm run dev               # Start UI on :5173

# Option 2: Docker
docker compose up -d
```

## Reverse Proxy Setup

DevHub auto-generates Caddy configurations for each project. The included proxy setup uses:

- **Caddy** - Modern reverse proxy with automatic HTTPS
- **Docker** - Containerized for easy management
- **`.localhost` domains** - No `/etc/hosts` editing needed (Chrome/Firefox)

### Directory Structure

```
proxy/
├── docker-compose.yml    # Caddy container
├── Caddyfile             # Main config (imports sites.d/*)
└── sites.d/              # Auto-generated per-project configs
    ├── devhub.caddy      # Dashboard routes
    ├── my-app.caddy      # Your project routes
    └── ...
```

### Custom Proxy Location

By default, DevHub looks for the proxy at `~/docker/reverse-proxy/`. To change:

```bash
# Create config directory
mkdir -p ~/.devhub

# Edit config
cat > ~/.devhub/config.toml << EOF
caddy_sites_dir = "/path/to/your/proxy/sites.d"
caddy_reload_command = "docker exec caddy-proxy caddy reload --config /etc/caddy/Caddyfile"
EOF
```

## Project Structure

```
devhub/
├── src/                  # Rust CLI + daemon
│   ├── main.rs           # CLI entry (clap)
│   ├── api.rs            # REST API (axum)
│   ├── process.rs        # Service management
│   ├── discovery.rs      # Auto-detection
│   ├── manifest.rs       # TOML parsing
│   ├── registry.rs       # Project registry
│   ├── caddy.rs          # Proxy config generation
│   └── config.rs         # Global settings
├── devhub-ui/            # SvelteKit dashboard
├── proxy/                # Caddy reverse proxy setup
├── Dockerfile            # Multi-stage build
└── docker-compose.yml    # Container orchestration
```

## Troubleshooting

### Port shows as "not running" but service is up

Modern servers (Next.js, Vite) often listen on IPv6. DevHub checks both IPv4 and IPv6, but if you still have issues:

```bash
# Check what's actually listening
lsof -i :3000
```

### Caddy not reloading

```bash
# Manual reload
docker exec caddy-proxy caddy reload --config /etc/caddy/Caddyfile

# Check Caddy logs
docker logs caddy-proxy
```

### Service won't start

```bash
# Check logs
devhub logs my-project my-service

# Or check log files directly
cat ~/Library/Application\ Support/com.moinsen.devhub/logs/my-project/my-service.err.log
```

## Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- [Caddy](https://caddyserver.com/) - The reverse proxy that makes this possible
- [axum](https://github.com/tokio-rs/axum) - Excellent Rust web framework
- [SvelteKit](https://kit.svelte.dev/) - Clean, fast UI framework
