# DevHub

**Multi-project development environment manager** - Start, stop, and monitor all your local development services from one place.

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

## Key Features

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

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        DevHub Dashboard (SvelteKit)                  │
│                    http://devhub.localhost:5173                      │
│  Features: Project grid, Start/Stop buttons, Search, Status polling │
└─────────────────────────────────────────────────────────────────────┘
                              │ HTTP (proxied)
┌─────────────────────────────┴───────────────────────────────────────┐
│                      DevHub Daemon (Rust/Axum)                       │
│                    http://localhost:9876                             │
│   - REST API (/api/projects, /api/projects/:name/start, etc.)       │
│   - Registry management (~/.devhub/registry.json)                   │
│   - Process spawning (native + PM2 for Node)                        │
│   - Docker Compose integration                                       │
│   - Health check polling                                             │
│   - Caddy config generation                                          │
└─────────────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────┴───────────────────────────────────────┐
│                      Caddy Reverse Proxy (Docker)                    │
│   http://project.localhost → localhost:PORT                         │
└─────────────────────────────────────────────────────────────────────┘
```

## Quick Links

- [Installation](./getting-started/installation.md)
- [Quick Start Guide](./getting-started/quick-start.md)
- [CLI Reference](./reference/cli.md)
- [GitHub Repository](https://github.com/moinsen-dev/devhub)

---

*"Stop remembering. Start building."*
