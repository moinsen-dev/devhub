# devhub.toml Reference

The `devhub.toml` file is the project manifest that tells DevHub how to manage your project's services.

## Basic Structure

```toml
[project]
name = "my-project"
description = "My awesome project"
tags = ["rust", "web", "api"]
env_files = [".env", ".env.local"]

[[services]]
name = "api"
type = "rust-binary"
command = "cargo run --release"
port = 8080
health_check = "/health"
subdomain = "api"
env_file = "api/.env"

[[services]]
name = "frontend"
type = "node"
command = "npm run dev"
cwd = "frontend"
port = 3000
main = true
depends_on = ["api"]

[environment]
RUST_LOG = "info"
DATABASE_URL = "postgres://localhost/mydb"
```

## [project] Section

Project-level metadata.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Unique project identifier |
| `description` | string | No | Human-readable description |
| `tags` | array | No | Tags for organization/filtering |
| `env_files` | array | No | Environment files to load for all services |

### Example

```toml
[project]
name = "my-fullstack-app"
description = "Full-stack application with API and frontend"
tags = ["typescript", "react", "node", "postgres"]
env_files = [".env", ".env.local"]
```

## [[services]] Section

Define one or more services. Each service represents a runnable process.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | string | Yes | - | Service identifier |
| `type` | string | Yes | - | Service type (see below) |
| `command` | string | Yes | - | Command to run |
| `port` | integer | Yes | - | Port the service listens on |
| `cwd` | string | No | project root | Working directory (relative) |
| `health_check` | string | No | - | HTTP endpoint for health checks |
| `subdomain` | string | No | - | Subdomain for reverse proxy |
| `main` | boolean | No | false | Primary service (no subdomain) |
| `depends_on` | array | No | [] | Services to start first |
| `env` | table | No | {} | Service-specific env vars |
| `env_file` | string | No | - | Service-specific env file path |

### Service Types

| Type | Detection | Default Behavior |
|------|-----------|------------------|
| `rust-binary` | Cargo.toml | Native process |
| `node` | package.json | PM2 if available, otherwise native |
| `python` | pyproject.toml | Native process |
| `go` | go.mod | Native process |
| `docker-compose` | docker-compose.yml | Docker Compose commands |
| `shell` | Any | Native process |

### Example Service

```toml
[[services]]
name = "api"
type = "rust-binary"
command = "cargo run --bin api --release"
port = 8080
health_check = "/api/health"
subdomain = "api"
env = { "RUST_LOG" = "debug", "DATABASE_URL" = "postgres://localhost/mydb" }
env_file = "api/.env"
```

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

### Variable Interpolation

Variables can reference other variables using `$VAR` or `${VAR}` syntax:

```toml
[environment]
DB_HOST = "localhost"
DB_PORT = "5432"
DB_NAME = "myapp"
DATABASE_URL = "postgres://${DB_HOST}:${DB_PORT}/${DB_NAME}"
```

## Monorepo Configuration

For projects with multiple services in subdirectories:

```toml
[project]
name = "my-monorepo"
description = "Fullstack monorepo"
env_files = [".env"]

[[services]]
name = "api"
type = "python"
command = "uvicorn app:app --host 0.0.0.0 --port 8000"
cwd = "backend"
port = 8000
subdomain = "api"

[[services]]
name = "web"
type = "node"
command = "npm run dev"
cwd = "frontend"
port = 3000
main = true
depends_on = ["api"]

[[services]]
name = "admin"
type = "node"
command = "npm run dev"
cwd = "admin-panel"
port = 3001
subdomain = "admin"
depends_on = ["api"]
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

## Complete Example

```toml
[project]
name = "ecommerce-platform"
description = "Full-stack e-commerce application"
tags = ["typescript", "rust", "postgres", "redis"]
env_files = [".env", ".env.local"]

[[services]]
name = "postgres"
type = "docker-compose"
command = "docker compose up -d postgres"
port = 5432

[[services]]
name = "redis"
type = "docker-compose"
command = "docker compose up -d redis"
port = 6379

[[services]]
name = "api"
type = "rust-binary"
command = "cargo run --bin api --release"
cwd = "services/api"
port = 8080
health_check = "/health"
subdomain = "api"
depends_on = ["postgres", "redis"]
env_file = "services/api/.env"

[[services]]
name = "worker"
type = "rust-binary"
command = "cargo run --bin worker --release"
cwd = "services/worker"
port = 8081
depends_on = ["postgres", "redis"]

[[services]]
name = "storefront"
type = "node"
command = "npm run dev"
cwd = "apps/storefront"
port = 3000
main = true
depends_on = ["api"]

[[services]]
name = "admin"
type = "node"
command = "npm run dev"
cwd = "apps/admin"
port = 3001
subdomain = "admin"
depends_on = ["api"]

[environment]
NODE_ENV = "development"
RUST_LOG = "info"
DATABASE_URL = "postgres://localhost:5432/ecommerce"
REDIS_URL = "redis://localhost:6379"
```
