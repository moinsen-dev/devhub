# DevHub - Product Initiation Document (PID)

> **Status**: Draft v1.0
> **Created**: 2025-12-16
> **Author**: Udi + Claude (Product Shaper)

---

## 1. Vision

**DevHub**: A desktop command center for developers managing 100+ projects. One registry, one dashboard, zero friction. Ship-ready CLI + web dashboard for macOS power users.

**One-Liner**: *"Stop remembering. Start building."*

**Go/No-Go**: ✅ YES - This solves a daily, painful, productivity-killing problem.

---

## 2. Personas

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| **Udi (Primary)** | Power developer with 100+ projects across ideas/opensource/apps, works on 8+ daily | Instant context switching without cognitive overhead |
| **Solo Developer** | Anyone managing multiple projects who's tired of terminal chaos | Single source of truth for all projects |
| **Team Lead** | Wants visibility into which services are running locally | Dashboard overview |
| **Open Source Contributor** | Juggles many repos, needs quick project bootstrapping | Fast onboarding to any project |

---

## 3. The Exciting Challenge (What We Dream Of)

- **Eliminate "which terminal was that?"** → One place to see and control everything
- **Kill port conflicts forever** → DevHub detects, warns, and resolves automatically
- **Zero startup friction** → `devhub start projectname` just works
- **No more README hunting** → DevHub knows how to start each project
- **Instant context awareness** → Dashboard shows what's running, what's stopped, what's unhealthy

---

## 4. How We Solve It (Core)

| Feature | Description |
|---------|-------------|
| **Central Registry** | `~/.devhub/registry.json` - Single source of truth for all 100+ projects |
| **Auto-Discovery** | Scan `~/work/moinsen/{ideas,opensource,apps}/` and detect project types automatically |
| **Project Manifests** | `devhub.toml` - Per-project config for services, ports, commands |
| **PM2 Integration** | Leverage PM2 for Node.js process management |
| **Docker Compose Support** | Manage containerized services alongside native processes |
| **Port Management** | Track allocated ports, detect conflicts before they happen |
| **Caddy Integration** | Auto-generate `*.localhost` subdomains for each project |
| **Health Monitoring** | Port checks + HTTP health endpoints |
| **Unified Logs** | Stream logs from all services in one place |
| **Web Dashboard** | Visual command center at `http://devhub.localhost` |
| **Recent/Favorites** | Quick access to most-used projects (not scrolling through 100!) |
| **Rust CLI + Daemon** | Fast, single binary, no runtime dependencies |

---

## 5. State Flow (Loop)

```
Discover → Register → Configure → Start → Monitor → Stop → (repeat)
```

**Detailed Flow:**

1. **Discover** - Scan directories, detect project types (Cargo.toml, package.json, docker-compose.yml)
2. **Register** - Add to central registry with metadata (path, type, services)
3. **Configure** - Generate/validate `devhub.toml` manifest
4. **Start** - Launch services via PM2/Docker/native, configure Caddy routes
5. **Monitor** - Health checks, log aggregation, port tracking
6. **Stop** - Graceful shutdown, cleanup Caddy routes

---

## 6. Deliverables

### Core CLI & Daemon

- [ ] **Rust CLI binary** (`devhub`) - Core commands: init, register, start, stop, status, logs, discover
- [ ] **Background Daemon** - Manages processes, health checks, Caddy config
- [ ] **PM2 Integration Layer** - Wraps PM2 for Node projects
- [ ] **Docker Compose Integration** - Manages container services
- [ ] **Native Process Manager** - Direct spawning for Rust/Go/Python

### Discovery & Configuration

- [ ] **Auto-Discovery Engine** - Detects project types from file signatures
- [ ] **devhub.toml Schema** - Project manifest specification
- [ ] **Registry Format** - JSON/SQLite for project metadata
- [ ] **Port Conflict Resolver** - Tracks ports, suggests alternatives

### Dashboard & UX

- [ ] **SvelteKit Dashboard** - Project grid, status indicators, log viewer, quick actions
- [ ] **Recent Projects List** - Track last-used for quick access
- [ ] **Favorites/Pinned** - Star your daily drivers

### Infrastructure

- [ ] **Caddy Config Generator** - Auto-creates `*.localhost` routes
- [ ] **Installation Script** - `cargo install` or brew tap
- [ ] **Shell Completions** - bash/zsh/fish tab completion

---

## 7. Inputs & Configuration

### Project Directories (Auto-Scan)

```
~/work/moinsen/
├── ideas/        # Experimental projects
├── opensource/   # Open source contributions
└── apps/         # Production applications
```

### Infrastructure

- **Process Manager**: PM2 (Node), native (Rust/Go), Docker Compose (containers)
- **Reverse Proxy**: Caddy at `~/docker/reverse-proxy/`
- **Data Store**: `~/.devhub/` (registry, config, logs)

### Port Allocation Strategy

| Type | Range | Example |
|------|-------|---------|
| Web frontends | 3000-3099 | `egb.localhost:3000` |
| APIs | 8000-8099 | `api.egb.localhost:8080` |
| Databases | 5432, 27017, 6379 | Standard ports |
| Custom | 9000-9999 | Overflow/special |

---

## 8. Success Criteria

| Criteria | Target |
|----------|--------|
| Auto-discovery | `devhub scan` discovers and registers 100+ projects automatically |
| Universal start | `devhub start <project>` works for any registered project (Rust, Node, Docker, etc.) |
| Port safety | Conflicts detected BEFORE starting (not after crash) |
| Dashboard | Shows all projects with real-time status |
| Context switch time | < 5 seconds from "I want to work on X" to "X is running" |
| Offline | Works completely offline (local macOS, no cloud dependency) |

---

## 9. Exciting Future Horizons (Phase 2+)

> *"Maybe not today, but tomorrow is NOT far away!"*

- **Project Templates** - `devhub new rust-api` scaffolds with best practices
- **Environment Sync** - Share `.env` configs across related projects
- **Dependency Graph** - Visualize which projects depend on which services
- **Resource Monitor** - CPU/memory per project
- **Mobile Companion** - Check project status from your phone
- **Team Sync** - Share project configs with teammates (opt-in)
- **AI Assistant** - "What projects use PostgreSQL?" "Which ones haven't been touched in 30 days?"

---

## 10. Success Amplifiers

- **Smart Defaults** - Sensible port allocation, auto-detect common patterns
- **Alias Support** - `devhub start egb` works even if full name is "edgegraphbox"
- **Fuzzy Search** - Don't remember exact name? `devhub start edge` finds it
- **Shell Completions** - Tab completion for project names in bash/zsh/fish
- **Quiet Mode** - `devhub start -q` for scripting, no output noise
- **Batch Operations** - `devhub stop --all` or `devhub start frontend-*`

---

## 11. Technical Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        DevHub Dashboard                              │
│                    http://devhub.localhost                           │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐                 │
│  │  Project A   │ │   Project B  │ │   Project C  │  ... (100+)     │
│  │  ● Running   │ │  ● Running   │ │  ○ Stopped   │                 │
│  │  [Logs] [Stop]│ │  [Logs] [Stop]│ │    [Start]  │                │
│  └──────────────┘ └──────────────┘ └──────────────┘                 │
└─────────────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────┴───────────────────────────────────────┐
│                      DevHub Daemon (Rust)                            │
│   - Project Registry (JSON → SQLite)                                │
│   - Process Manager (PM2 / Docker / Native)                         │
│   - Port Allocator (conflict detection)                             │
│   - Log Aggregator (unified streaming)                              │
│   - Health Monitor (port checks, HTTP health)                       │
│   - Caddy Config Generator (auto-reload)                            │
│   - REST API (for dashboard)                                        │
│   - WebSocket (real-time updates)                                   │
└─────────────────────────────┬───────────────────────────────────────┘
                              │
┌─────────────────────────────┴───────────────────────────────────────┐
│                      Caddy Reverse Proxy                             │
│   http://projecta.localhost → localhost:3000                        │
│   http://api.projecta.localhost → localhost:8080                    │
│   http://devhub.localhost → DevHub Dashboard                        │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 12. CLI Command Reference

```bash
# Discovery & Registration
devhub scan [path]              # Auto-discover projects in directory
devhub register [path]          # Register current/specified project
devhub unregister <name>        # Remove project from registry
devhub list                     # List all registered projects
devhub list --recent            # Show recently used projects
devhub list --running           # Show only running projects

# Project Management
devhub init                     # Create devhub.toml in current project
devhub start [project]          # Start project services
devhub stop [project]           # Stop project services
devhub restart [project]        # Restart services
devhub status [project]         # Show status (all or specific)

# Monitoring
devhub logs <project> [service] # Stream logs (unified or per-service)
devhub health [project]         # Health check report
devhub ports                    # Show port allocation table

# Daemon
devhub daemon                   # Start background daemon
devhub daemon stop              # Stop daemon

# Utilities
devhub open <project>           # Open project in browser
devhub code <project>           # Open project in VS Code
devhub cd <project>             # Print cd command (use with: cd $(devhub cd foo))
```

---

## 13. Project Manifest (devhub.toml)

```toml
[project]
name = "my-project"
description = "My awesome project"
tags = ["rust", "api", "database"]

[[services]]
name = "api"
type = "rust-binary"                    # rust-binary | node | docker-compose | python
command = "cargo run --release"
port = 8080
health_check = "/health"
subdomain = "api"                       # → api.my-project.localhost

[[services]]
name = "web"
type = "node"
command = "npm run dev"
cwd = "apps/web"
port = 3000
health_check = "/"
main = true                             # → my-project.localhost (no subdomain)
depends_on = ["api"]

[[services]]
name = "db"
type = "docker-compose"
compose_file = "docker/docker-compose.yml"
service = "postgres"
port = 5432

[environment]
RUST_LOG = "info"
DATABASE_URL = "postgres://localhost:5432/mydb"
```

---

## 14. Implementation Phases

### Phase 1: Foundation (MVP)
- [ ] Rust CLI skeleton with clap
- [ ] Registry (JSON-based)
- [ ] `devhub init`, `register`, `list` commands
- [ ] Basic `start`/`stop` with native process spawning
- [ ] PM2 integration for Node projects

### Phase 2: Auto-Discovery & Ports
- [ ] Directory scanner
- [ ] Project type detection
- [ ] Port allocation & conflict detection
- [ ] Caddy config generation

### Phase 3: Docker & Health
- [ ] Docker Compose integration
- [ ] Health monitoring
- [ ] Log aggregation

### Phase 4: Dashboard
- [ ] SvelteKit UI
- [ ] REST API
- [ ] WebSocket for real-time updates

### Phase 5: Polish
- [ ] Recent/favorites
- [ ] Shell completions
- [ ] Fuzzy search
- [ ] Documentation

---

## 15. Why This Will Succeed

1. **Real Pain** - This isn't theoretical. 100 projects, 8 daily, constant friction.
2. **Clear Solution** - Central registry + unified interface = cognitive load eliminated.
3. **Right Tools** - Rust (fast, reliable), PM2 (battle-tested), Caddy (modern proxy).
4. **Incremental Value** - Even Phase 1 MVP provides immediate relief.
5. **AI-First Era** - With AI agents, implementation is fast. Focus on value, not timelines.

---

## 16. Next Steps

1. ✅ PID approved
2. [ ] Create Rust project structure
3. [ ] Implement Phase 1 MVP
4. [ ] Test with real projects (EdgeGraphBox as first guinea pig)
5. [ ] Iterate based on daily usage

---

*"If you can dream about it, somebody can build it."*

**DevHub**: Stop remembering. Start building. 🚀
