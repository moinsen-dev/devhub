# Your First Project

This guide walks you through manually setting up a project with DevHub.

## Option 1: Auto-Discovery (Easiest)

```bash
cd ~/work/my-project

# Let DevHub detect your project type
devhub discover

# Output:
# Discovered: my-project (node)
# Services:
#   - frontend (port 3000): npm run dev
#   - api (port 8080): npm run api
#
# Generated devhub.toml

# Register with DevHub
devhub register
```

## Option 2: Manual Setup

### Step 1: Initialize Config

```bash
cd ~/work/my-project
devhub init
```

This creates a `devhub.toml` template.

### Step 2: Edit devhub.toml

```toml
[project]
name = "my-project"
description = "My awesome full-stack application"
tags = ["web", "api", "docker"]

# API service
[[services]]
name = "api"
type = "node"
command = "npm run api"
port = 8080
health_check = "/health"
subdomain = "api"              # → http://api.my-project.localhost

# Frontend service
[[services]]
name = "frontend"
type = "node"
command = "npm run dev"
cwd = "frontend"               # Run from subdirectory
port = 3000
main = true                    # → http://my-project.localhost
depends_on = ["api"]           # Start after API

# Database (Docker)
[[services]]
name = "postgres"
type = "docker-compose"
command = "docker compose up -d"
port = 5432

# Environment variables for all services
[environment]
NODE_ENV = "development"
DATABASE_URL = "postgres://localhost:5432/myapp"
```

### Step 3: Register

```bash
devhub register

# Output:
# ✓ Registered my-project at /Users/you/work/my-project
```

### Step 4: Start

```bash
devhub start my-project

# Output:
#   → Starting my-project...
#   → Starting postgres...
#   ✓ postgres started (port 5432)
#   → Starting api on port 8080...
#   ✓ api started (port 8080)
#     Health check ✓
#   → Starting frontend on port 3000...
#   ✓ frontend started (port 3000)
```

## Understanding Service Types

DevHub supports several service types:

| Type | Use Case | Example |
|------|----------|---------|
| `node` | Node.js apps | `npm run dev`, `yarn start` |
| `rust-binary` | Rust projects | `cargo run` |
| `python` | Python apps | `uvicorn main:app` |
| `go` | Go applications | `go run .` |
| `docker-compose` | Docker services | `docker compose up` |
| `shell` | Custom commands | Any shell command |

## Service Options

Each service supports these options:

```toml
[[services]]
name = "my-service"           # Required: unique identifier
type = "node"                 # Required: service type
command = "npm run dev"       # Required: start command
port = 3000                   # Required: port to listen on
cwd = "."                     # Optional: working directory
main = false                  # Optional: is this the main service?
subdomain = "api"             # Optional: subdomain for proxy
health_check = "/health"      # Optional: health check endpoint
depends_on = ["db", "cache"]  # Optional: start after these services

[services.env]                # Optional: per-service environment
API_KEY = "secret"
```

## Verify Your Setup

```bash
# Check registration
devhub list
# my-project    /Users/you/work/my-project

# Check status
devhub status
# my-project
#   ○ api (8080)
#   ○ frontend (3000)
#   ○ postgres (5432)

# View configuration
cat devhub.toml
```

## Next Steps

- [Configuration Reference](../guide/devhub-toml.md) - All devhub.toml options
- [Service Types](../reference/service-types.md) - Detailed type documentation
- [Reverse Proxy](../guide/reverse-proxy.md) - Set up pretty URLs
