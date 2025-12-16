# Changelog

All notable changes to DevHub will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2025-12-16

### Added

- **Fuzzy Search**
  - `devhub search <query>` - Find projects by fuzzy name matching
  - Score-based ranking shows best matches first
  - Integrated SkimMatcherV2 for fast, accurate matching

- **Favorites System**
  - `devhub fav add <project>` - Star a project as favorite
  - `devhub fav remove <project>` - Remove from favorites
  - `devhub fav list` - List all favorite projects
  - `devhub fav toggle <project>` - Toggle favorite status

- **Recent Projects Tracking**
  - `devhub recent` - Show recently used projects
  - Automatic `last_used` timestamp on start operations
  - Configurable limit (default: 10)

- **Batch Operations**
  - `devhub start --all` - Start all registered projects
  - `devhub start --favorites` - Start all favorite projects
  - `devhub stop --all` / `--favorites` - Batch stop
  - `devhub restart --all` / `--favorites` - Batch restart

- **Port Management**
  - `devhub ports --check` - Show port allocation by range
  - `devhub ports --resolve` - Suggest fixes for port conflicts
  - Port ranges: frontend (3000-3099), API (8000-8099), admin (9000-9099), flutter (3100-3199), rust (8100-8199)

- **Dashboard Enhancements**
  - Log viewer modal with auto-refresh (2s interval)
  - Syntax highlighting for errors/warnings in logs
  - Service-level start/stop controls (visible on hover)
  - Logs button in project card footer

- **REST API Additions**
  - `POST /api/projects/:name/services/:service/start` - Start specific service
  - `POST /api/projects/:name/services/:service/stop` - Stop specific service

### Changed

- ProjectEntry now includes `last_used` and `favorite` fields
- Registry schema is backward-compatible with `#[serde(default)]`

## [0.1.0] - 2025-12-16

### Added

- **CLI Commands**
  - `devhub init` - Create devhub.toml in current directory
  - `devhub discover` - Auto-detect project type and generate config
  - `devhub register` - Register project with DevHub
  - `devhub unregister` - Remove project from registry
  - `devhub list` - List all registered projects with details
  - `devhub status` - Show running status of all services
  - `devhub start` - Start project services with progress feedback
  - `devhub stop` - Gracefully stop services
  - `devhub restart` - Stop and start services
  - `devhub logs` - Stream service logs
  - `devhub daemon` - Run REST API server for dashboard

- **Auto-Discovery**
  - Rust projects (Cargo.toml with [[bin]] sections)
  - Node.js projects (package.json with dev/start scripts)
  - Python projects (pyproject.toml, requirements.txt)
  - Go projects (go.mod)
  - Docker Compose projects (docker-compose.yml)
  - Monorepo support (Cargo workspace, npm workspaces)

- **Service Management**
  - Process spawning with proper signal handling
  - IPv4 and IPv6 port detection
  - Configurable startup timeouts per service type
  - Log file management (stdout/stderr separation)
  - PID file tracking

- **Reverse Proxy Integration**
  - Auto-generate Caddy configs per project
  - Support for main service and subdomains
  - Automatic Caddy reload on service start

- **Web Dashboard**
  - SvelteKit 5 with Tailwind CSS
  - Project grid with status indicators
  - Start/Stop/Restart buttons per project
  - Service URLs (clickable)
  - Dark mode

- **REST API**
  - `GET /api/projects` - List all projects with status
  - `GET /api/projects/:name` - Get project details
  - `POST /api/projects/:name/start` - Start project
  - `POST /api/projects/:name/stop` - Stop project
  - `POST /api/projects/:name/restart` - Restart project
  - `GET /api/projects/:name/logs` - Get service logs

- **Docker Support**
  - Multi-stage Dockerfile (Rust + Node build)
  - docker-compose.yml for containerized deployment
  - Volume mounts for persistence

### Fixed

- IPv6 port detection for Next.js and other modern Node servers

[Unreleased]: https://github.com/moinsen-dev/devhub/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/moinsen-dev/devhub/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/moinsen-dev/devhub/releases/tag/v0.1.0
