# Configuration

DevHub uses two configuration files:

1. **`devhub.toml`** - Per-project configuration (in each project directory)
2. **`~/.devhub/config.toml`** - Global configuration

## Configuration Hierarchy

```
~/.devhub/config.toml     (global defaults)
    ↓
project/devhub.toml       (project-specific)
    ↓
.env files                (environment variables)
    ↓
runtime environment       (system env vars)
```

## Environment Variable Loading

DevHub automatically loads `.env` files with the following priority (later sources override earlier):

1. System environment variables
2. Project root `.env` file
3. Project root `.env.local` file
4. Files listed in `env_files` in devhub.toml
5. `[environment]` section in devhub.toml
6. Service-specific `env_file`
7. Service `cwd/.env` file
8. Service `cwd/.env.local` file
9. Service `env = {}` section

### Variable Interpolation

Variables can reference other variables:

```bash
# .env file
DB_HOST=localhost
DB_PORT=5432
DATABASE_URL=postgres://${DB_HOST}:${DB_PORT}/mydb
```

### View Resolved Environment

Use `devhub env` to see the final resolved environment:

```bash
# Show all services
devhub env my-project

# Show specific service
devhub env my-project -s api

# Export format for sourcing
devhub env my-project -s api -f export
```

## Quick Links

- [devhub.toml Reference](./devhub-toml.md) - Project configuration
- [Global Config](./global-config.md) - System-wide settings
