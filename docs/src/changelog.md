# Changelog

## [0.3.0] - 2025-12-16

### Added

- **Environment File Loading**
  - Automatic `.env` and `.env.local` file support
  - Variable interpolation with `$VAR` and `${VAR}` syntax
  - `env_files` field in project config for explicit env file list
  - `env_file` field per service for service-specific env files
  - `devhub env` command to show resolved environment variables

- **Monorepo Auto-Discovery**
  - Automatic subdirectory scanning when no root project type found
  - Detects multiple project types in subdirectories
  - Generates multi-service `devhub.toml` with correct `cwd` settings
  - Smart port allocation for discovered services

- **Discovery Enhancements**
  - `devhub discover --dry-run` - Preview without writing files

## [0.2.0] - 2025-12-16

### Added

- **Fuzzy Search** - `devhub search <query>` for finding projects
- **Favorites System** - `devhub fav` commands to star projects
- **Recent Projects** - `devhub recent` to show recently used projects
- **Batch Operations** - `--all` and `--favorites` flags for start/stop/restart
- **Port Management** - `devhub ports --check` and `--resolve` for conflict detection
- **Dashboard Enhancements** - Log viewer, service-level controls

## [0.1.0] - 2025-12-16

### Added

- Core CLI commands (init, discover, register, list, status, start, stop, restart, logs)
- Bulk operations (scan, ports)
- Developer shortcuts (open, code, path)
- Service support: Rust, Node.js, Python, Go, Docker Compose, Shell
- PM2 integration for Node.js
- Docker Compose support
- Health check endpoint polling
- SvelteKit dashboard with real-time polling
- Caddy reverse proxy integration
- Shell completions (bash, zsh, fish)
