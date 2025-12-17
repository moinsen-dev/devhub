# devhub.toml Configuration Reference

The `devhub.toml` file is the project manifest that tells DevHub how to manage your project's services.

## Complete Structure

```toml
[project]
name = "my-project"
description = "Project description"
tags = ["rust", "web"]
env_files = [".env", ".env.local"]

[[services]]
name = "api"
type = "rust-binary"
command = "cargo run --release"
port = 8080
cwd = "backend"
health_check = "/health"
subdomain = "api"
main = false
depends_on = ["database"]
env_file = "backend/.env"
env = { KEY = "value" }

[environment]
NODE_ENV = "development"
DATABASE_URL = "postgres://localhost/mydb"
```

## [project] Section

Project-level metadata.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | **Yes** | Unique project identifier |
| `description` | string | No | Human-readable description |
| `tags` | array | No | Tags for organization/filtering |
| `env_files` | array | No | Environment files to load for all services |

## [[services]] Section

Define one or more services. Each service represents a runnable process.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | string | **Yes** | - | Service identifier |
| `type` | string | **Yes** | - | Service type (see below) |
| `command` | string | **Yes** | - | Command to run |
| `port` | integer | **Yes** | - | Port the service listens on |
| `cwd` | string | No | project root | Working directory (relative to project) |
| `health_check` | string | No | - | HTTP endpoint for health checks |
| `subdomain` | string | No | - | Subdomain for reverse proxy |
| `main` | boolean | No | false | Primary service (no subdomain needed) |
| `depends_on` | array | No | [] | Services to start first |
| `env` | table | No | {} | Service-specific environment variables |
| `env_file` | string | No | - | Service-specific env file path |

## Service Types

| Type | Detection | Default Behavior |
|------|-----------|------------------|
| `rust-binary` | `Cargo.toml` | Native process |
| `node` | `package.json` | PM2 if available, otherwise native |
| `python` | `pyproject.toml` | Native process |
| `go` | `go.mod` | Native process |
| `docker-compose` | `docker-compose.yml` | Docker Compose commands |
| `shell` | Any | Native process |

## [environment] Section

Global environment variables applied to all services.

```toml
[environment]
NODE_ENV = "development"
RUST_LOG = "info"
DATABASE_URL = "postgres://localhost/mydb"
API_KEY = "dev-key-123"
```

## Environment Loading Priority

Environment variables are loaded in this order (later sources override earlier):

1. System environment variables
2. Project root `.env` file
3. Project root `.env.local` file
4. Files listed in `env_files`
5. `[environment]` section in manifest
6. Service-specific `env_file`
7. Service `cwd/.env` file
8. Service `cwd/.env.local` file
9. Service `env = {}` section

## Variable Interpolation

Variables can reference other variables using `$VAR` or `${VAR}` syntax:

```toml
[environment]
DB_HOST = "localhost"
DB_PORT = "5432"
DB_NAME = "myapp"
DATABASE_URL = "postgres://${DB_HOST}:${DB_PORT}/${DB_NAME}"
```

## Service Dependencies

Use `depends_on` to control startup order:

```toml
[[services]]
name = "database"
type = "docker-compose"
command = "docker compose up -d postgres"
port = 5432

[[services]]
name = "api"
type = "rust-binary"
command = "cargo run"
port = 8080
depends_on = ["database"]

[[services]]
name = "frontend"
type = "node"
command = "npm run dev"
port = 3000
depends_on = ["api"]
```

Services start in dependency order: database → api → frontend

## Reverse Proxy Integration

DevHub auto-generates Caddy configurations for `.localhost` URLs:

- `main = true` → `http://project-name.localhost`
- `subdomain = "api"` → `http://api.project-name.localhost`

## Port Ranges (Convention)

| Type | Range | Example |
|------|-------|---------|
| Frontend | 3000-3099 | React, Vue, Svelte |
| Flutter | 3100-3199 | Flutter web |
| API | 8000-8099 | REST APIs |
| Rust | 8100-8199 | Rust services |
| Admin | 9000-9099 | Admin panels |
