# CLI Reference

Complete reference for all DevHub commands.

## Global Options

```bash
devhub [OPTIONS] <COMMAND>

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Commands

### Project Management

#### `devhub init`

Create a new `devhub.toml` in the current directory.

```bash
devhub init [--force]

Options:
  --force, -f  Overwrite existing devhub.toml
```

#### `devhub discover`

Auto-detect project type and generate `devhub.toml`.

```bash
devhub discover [PATH]

Arguments:
  PATH  Project path (default: current directory)
```

#### `devhub register`

Register a project with DevHub.

```bash
devhub register [PATH]

Arguments:
  PATH  Project path (default: current directory)
```

#### `devhub unregister`

Remove a project from DevHub registry.

```bash
devhub unregister <NAME>

Arguments:
  NAME  Project name to unregister
```

#### `devhub list`

List all registered projects.

```bash
devhub list
```

### Service Control

#### `devhub start`

Start project services.

```bash
devhub start [PROJECT] [--service <SERVICE>]

Arguments:
  PROJECT  Project name (optional if in project directory)

Options:
  --service, -s  Start only this service
```

#### `devhub stop`

Stop project services.

```bash
devhub stop [PROJECT] [--service <SERVICE>]

Arguments:
  PROJECT  Project name (optional if in project directory)

Options:
  --service, -s  Stop only this service
```

#### `devhub restart`

Restart project services.

```bash
devhub restart [PROJECT] [--service <SERVICE>]

Arguments:
  PROJECT  Project name (optional if in project directory)

Options:
  --service, -s  Restart only this service
```

#### `devhub status`

Show status of all projects and services.

```bash
devhub status
```

#### `devhub logs`

View project logs.

```bash
devhub logs <PROJECT> [SERVICE] [--follow]

Arguments:
  PROJECT  Project name
  SERVICE  Service name (optional)

Options:
  --follow, -f  Follow log output (like tail -f)
```

### Bulk Operations

#### `devhub scan`

Scan directories for projects.

```bash
devhub scan [PATHS...] [OPTIONS]

Arguments:
  PATHS  Directories to scan (default: ~/work/moinsen/{ideas,opensource,apps})

Options:
  --depth, -d <N>     Maximum scan depth (default: 2)
  --auto-register, -a  Register discovered projects automatically
  --dry-run           Show what would be discovered without registering
```

### Port Management

#### `devhub ports`

Show port allocations across all projects.

```bash
devhub ports [--check]

Options:
  --check, -c  Detect and report port conflicts
```

### Developer Shortcuts

#### `devhub open`

Open project in browser.

```bash
devhub open <PROJECT> [--service <SERVICE>]

Arguments:
  PROJECT  Project name

Options:
  --service, -s  Open specific service (default: main service)
```

#### `devhub code`

Open project in VS Code.

```bash
devhub code <PROJECT>

Arguments:
  PROJECT  Project name
```

#### `devhub path`

Print project path (for shell integration).

```bash
devhub path <PROJECT>

Arguments:
  PROJECT  Project name

# Usage with cd:
cd $(devhub path my-project)
```

### Utilities

#### `devhub daemon`

Run the API server for the dashboard.

```bash
devhub daemon [--port <PORT>]

Options:
  --port, -p  Port to listen on (default: 9876)
```

#### `devhub completions`

Generate shell completions.

```bash
devhub completions <SHELL>

Arguments:
  SHELL  Shell type: bash, zsh, fish, elvish, powershell
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |

## Environment Variables

| Variable | Description |
|----------|-------------|
| `RUST_LOG` | Log level (error, warn, info, debug, trace) |
| `DEVHUB_CONFIG` | Custom config directory |

## Examples

```bash
# Discover and register all projects
devhub scan --auto-register ~/work

# Start a project and follow logs
devhub start my-project
devhub logs my-project -f

# Quick project switch
cd $(devhub path other-project)
devhub code other-project
devhub start other-project

# Check for port conflicts before starting
devhub ports --check
devhub start my-project

# Generate completions
devhub completions zsh > ~/.zfunc/_devhub
```
