# Changelog

All notable changes to DevHub will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/moinsen-dev/devhub/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/moinsen-dev/devhub/releases/tag/v0.1.0
