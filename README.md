# DevHub

**Multi-project development environment manager** - Start, stop, and monitor all your local development services from one place.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![Release](https://img.shields.io/github/v/release/moinsen-dev/devhub)](https://github.com/moinsen-dev/devhub/releases)
[![Docs](https://img.shields.io/badge/docs-mdbook-blue)](https://moinsen-dev.github.io/devhub/)

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
- **Bulk discovery** - Scan directories and register 100+ projects at once
- **Web dashboard** - Visual status, start/stop buttons, real-time updates
- **Auto-discovery** - Detects Rust, Node, Python, Go, Docker, Flutter projects
- **Port conflict detection** - Know before you crash
- **PM2 integration** - Node.js services managed properly
- **Docker Compose support** - First-class container orchestration
- **Health checks** - Wait for services to be truly ready
- **Reverse proxy integration** - Pretty URLs like `http://myproject.localhost`
- **Shell completions** - Tab completion for bash, zsh, and fish
- **Developer shortcuts** - `devhub code`, `devhub open`, `devhub path`
- **Zero lock-in** - Simple TOML config, works with any stack

---

## Quick Start

### 1. Install DevHub

```bash
# Homebrew (macOS)
brew tap moinsen-dev/tap
brew install devhub

# Or download pre-built binary
curl -sL https://github.com/moinsen-dev/devhub/releases/latest/download/devhub-aarch64-apple-darwin.tar.gz | tar xz
sudo mv devhub /usr/local/bin/

# Or from source
git clone https://github.com/moinsen-dev/devhub.git && cd devhub
cargo build --release && cp target/release/devhub ~/.local/bin/
```

### 2. Discover Your Projects

```bash
# Scan your project directories
devhub scan --dry-run ~/work

# Found 50 projects? Register them all!
devhub scan --auto-register ~/work
```

### 3. Install Shell Completions

```bash
# Zsh (most common on macOS)
devhub completions zsh > ~/.zfunc/_devhub

# Bash
devhub completions bash > ~/.local/share/bash-completion/completions/devhub

# Fish
devhub completions fish > ~/.config/fish/completions/devhub.fish
```

### 4. Start Working

```bash
# Start a project
devhub start my-project

# Check status
devhub status

# Quick navigation
cd $(devhub path my-project)  # Jump to project directory
devhub code my-project        # Open in VS Code
devhub open my-project        # Open in browser

# Check for port conflicts
devhub ports --check
```

### 5. Launch the Dashboard (Optional)

```bash
# Start the API daemon
devhub daemon &

# Start the UI
cd devhub-ui && npm install && npm run dev

# Open http://localhost:5173
```

---

## CLI Reference

### Project Management

```bash
devhub init                    # Create devhub.toml in current directory
devhub discover                # Auto-detect and generate devhub.toml
devhub register [path]         # Register project with DevHub
devhub unregister <name>       # Remove project from registry
devhub list                    # List all registered projects
```

### Bulk Operations

```bash
devhub scan [paths...]         # Scan directories for projects
devhub scan --dry-run          # Preview what would be discovered
devhub scan --auto-register    # Discover and register in one step
devhub scan --depth 3          # Scan deeper into subdirectories
```

### Service Control

```bash
devhub start [project]         # Start project services
devhub stop [project]          # Stop project services
devhub restart [project]       # Restart project services
devhub status                  # Show status of all projects
devhub logs <project> [-f]     # Stream logs (with follow mode)
```

### Port Management

```bash
devhub ports                   # Show all port allocations
devhub ports --check           # Detect port conflicts
```

### Developer Shortcuts

```bash
devhub open <project>          # Open project in browser
devhub open <project> -s api   # Open specific service
devhub code <project>          # Open project in VS Code
devhub path <project>          # Print project path (for cd integration)
```

### Utilities

```bash
devhub daemon [--port 9876]    # Run API server for dashboard
devhub completions <shell>     # Generate shell completions (bash/zsh/fish)
```

---

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
subdomain = "api"           # http://api.my-app.localhost

[[services]]
name = "frontend"
type = "node"
command = "npm run dev"
cwd = "frontend"            # Relative working directory
port = 3000
main = true                 # http://my-app.localhost (no subdomain)
depends_on = ["api"]        # Start order

[environment]
RUST_LOG = "info"
DATABASE_URL = "postgres://localhost/myapp"
```

### Service Types

| Type | Detection | Default Command |
|------|-----------|-----------------|
| `rust-binary` | `Cargo.toml` with `[[bin]]` | `cargo run` |
| `node` | `package.json` | `npm run dev` (uses PM2 if available) |
| `python` | `pyproject.toml` | `uvicorn` or `python -m` |
| `go` | `go.mod` | `go run .` |
| `docker-compose` | `docker-compose.yml` | `docker compose up -d` |
| `shell` | Any | Custom command |

---

## Dashboard

The web dashboard provides:

- Project cards with running status indicators
- One-click start/stop/restart buttons
- Real-time auto-refresh (5-second polling)
- Search and filter by name or status
- Service metrics (X/Y services running)

**Access:**
- `http://devhub.localhost` (with Caddy proxy)
- `http://localhost:5173` (development mode)

---

## Reverse Proxy Setup

DevHub auto-generates Caddy configurations for pretty `.localhost` URLs.

### Quick Setup

```bash
cd proxy
docker compose up -d
```

This gives you:
- `http://projectname.localhost` - Your main service
- `http://api.projectname.localhost` - API services
- `http://devhub.localhost` - Dashboard

### Custom Proxy Location

```bash
mkdir -p ~/.devhub
cat > ~/.devhub/config.toml << EOF
caddy_sites_dir = "/path/to/your/proxy/sites.d"
caddy_reload_command = "docker exec caddy-proxy caddy reload --config /etc/caddy/Caddyfile"
EOF
```

---

## Project Structure

```
devhub/
├── src/                  # Rust CLI + daemon
│   ├── main.rs           # CLI entry (clap) - 1100+ lines
│   ├── api.rs            # REST API (axum)
│   ├── process.rs        # Service management (PM2, Docker, native)
│   ├── discovery.rs      # Auto-detection (6 project types)
│   ├── manifest.rs       # TOML parsing
│   ├── registry.rs       # Project registry
│   ├── caddy.rs          # Proxy config generation
│   └── config.rs         # Global settings
├── devhub-ui/            # SvelteKit dashboard (Svelte 5)
├── proxy/                # Caddy reverse proxy setup
└── Cargo.toml            # Rust dependencies
```

---

## Troubleshooting

### Port shows as "not running" but service is up

DevHub checks both IPv4 and IPv6. If issues persist:

```bash
lsof -i :3000
```

### PM2 not being used

PM2 must be installed globally:

```bash
npm install -g pm2
```

### Service won't start

```bash
# Check logs
devhub logs my-project my-service

# Or directly
cat ~/Library/Application\ Support/com.moinsen.devhub/logs/my-project/my-service.err.log
```

### Caddy not reloading

```bash
docker exec caddy-proxy caddy reload --config /etc/caddy/Caddyfile
docker logs caddy-proxy
```

---

## Installation

### Homebrew (macOS)

```bash
brew tap moinsen-dev/tap
brew install devhub

# Start as background service (optional)
brew services start devhub
```

### Pre-built Binaries

Download from [GitHub Releases](https://github.com/moinsen-dev/devhub/releases):

```bash
# macOS Apple Silicon
curl -sL https://github.com/moinsen-dev/devhub/releases/latest/download/devhub-aarch64-apple-darwin.tar.gz | tar xz
sudo mv devhub /usr/local/bin/

# macOS Intel
curl -sL https://github.com/moinsen-dev/devhub/releases/latest/download/devhub-x86_64-apple-darwin.tar.gz | tar xz
sudo mv devhub /usr/local/bin/

# Linux x64
curl -sL https://github.com/moinsen-dev/devhub/releases/latest/download/devhub-x86_64-unknown-linux-gnu.tar.gz | tar xz
sudo mv devhub /usr/local/bin/

# Linux ARM64
curl -sL https://github.com/moinsen-dev/devhub/releases/latest/download/devhub-aarch64-unknown-linux-gnu.tar.gz | tar xz
sudo mv devhub /usr/local/bin/
```

### From Source

```bash
git clone https://github.com/moinsen-dev/devhub.git
cd devhub
cargo build --release
cp target/release/devhub ~/.local/bin/
```

### Cargo (crates.io) - Coming Soon

```bash
cargo install devhub
```

---

## Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

---

## Acknowledgments

- [Caddy](https://caddyserver.com/) - The reverse proxy that makes this possible
- [axum](https://github.com/tokio-rs/axum) - Excellent Rust web framework
- [clap](https://github.com/clap-rs/clap) - CLI argument parsing with completions
- [PM2](https://pm2.keymetrics.io/) - Node.js process management
- [SvelteKit](https://kit.svelte.dev/) - Clean, fast UI framework

---

*"Stop remembering. Start building."*
