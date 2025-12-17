# DevHub Configuration Examples

## Simple Node.js Project

```toml
[project]
name = "my-react-app"
description = "React frontend application"
tags = ["react", "typescript"]

[[services]]
name = "web"
type = "node"
command = "npm run dev"
port = 3000
main = true

[environment]
NODE_ENV = "development"
```

## Rust API Server

```toml
[project]
name = "rust-api"
description = "REST API built with Axum"
tags = ["rust", "api"]

[[services]]
name = "api"
type = "rust-binary"
command = "cargo run --release"
port = 8080
health_check = "/health"
main = true

[environment]
RUST_LOG = "info"
DATABASE_URL = "postgres://localhost/mydb"
```

## Python FastAPI Application

```toml
[project]
name = "fastapi-app"
description = "FastAPI backend service"
tags = ["python", "fastapi"]

[[services]]
name = "api"
type = "python"
command = "uv run uvicorn app:app --host 0.0.0.0 --port 8000 --reload"
port = 8000
health_check = "/health"
main = true

[environment]
PYTHONPATH = "src"
DATABASE_URL = "postgresql://localhost/mydb"
```

## Flutter Web Application

```toml
[project]
name = "flutter-web"
description = "Flutter web application"
tags = ["flutter", "dart"]

[[services]]
name = "web"
type = "shell"
command = "flutter run -d chrome --web-port 3000"
port = 3000
main = true
```

## Go Microservice

```toml
[project]
name = "go-service"
description = "Go microservice"
tags = ["go", "api"]

[[services]]
name = "api"
type = "go"
command = "go run ."
port = 8080
health_check = "/healthz"
main = true

[environment]
GO_ENV = "development"
```

## Docker Compose Stack

```toml
[project]
name = "docker-stack"
description = "Multi-container Docker application"
tags = ["docker"]

[[services]]
name = "stack"
type = "docker-compose"
command = "docker compose up"
port = 80
main = true
```

## Monorepo: Frontend + Backend

```toml
[project]
name = "fullstack-app"
description = "Full-stack application with API and frontend"
tags = ["typescript", "react", "node"]
env_files = [".env"]

[[services]]
name = "api"
type = "node"
command = "npm run dev"
cwd = "backend"
port = 8080
health_check = "/health"
subdomain = "api"

[[services]]
name = "web"
type = "node"
command = "npm run dev"
cwd = "frontend"
port = 3000
main = true
depends_on = ["api"]

[environment]
NODE_ENV = "development"
API_URL = "http://api.fullstack-app.localhost"
```

## Complex Monorepo with Database

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

## DevHub Itself (Real Example)

```toml
[project]
name = "devhub"
description = "Multi-project development environment manager"
tags = ["rust", "svelte", "developer-tools"]

[[services]]
name = "daemon"
type = "rust-binary"
command = "cargo run --release -- daemon"
port = 9876
health_check = "/api/projects"
subdomain = "api"

[[services]]
name = "dashboard"
type = "node"
command = "npm run dev"
cwd = "devhub-ui"
port = 5173
health_check = "/"
main = true
depends_on = ["daemon"]

[environment]
RUST_LOG = "info"
```

## Service with Custom Environment

```toml
[project]
name = "multi-env-project"
description = "Project with complex environment setup"

[[services]]
name = "api"
type = "node"
command = "npm run start"
port = 8080
env_file = "api/.env.local"
env = {
    DEBUG = "true",
    LOG_LEVEL = "debug",
    FEATURE_FLAG_NEW_UI = "enabled"
}

[environment]
SHARED_SECRET = "dev-secret-123"
```

## Tips for Writing Configurations

1. **Use `main = true`** for the primary service (usually frontend)
2. **Use `subdomain`** for secondary services (api, admin, etc.)
3. **Use `depends_on`** to ensure services start in the right order
4. **Use `cwd`** for monorepo subdirectories
5. **Use `env_files`** at project level for shared environment
6. **Use service-level `env_file`** for service-specific secrets
7. **Use `health_check`** to verify services are actually ready
