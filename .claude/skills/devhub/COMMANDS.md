# DevHub CLI Commands Reference

## Project Management

### `devhub init`
Create a basic `devhub.toml` template in the current directory.

```bash
devhub init [--force]

Options:
  --force, -f  Overwrite existing devhub.toml
```

### `devhub discover`
Auto-detect project type and generate `devhub.toml`. Supports monorepos by scanning subdirectories.

```bash
devhub discover [PATH] [--dry-run]

Arguments:
  PATH      Project path (default: current directory)

Options:
  --dry-run  Preview what would be discovered without writing files
```

### `devhub register`
Register a project with DevHub's global registry.

```bash
devhub register [PATH]

Arguments:
  PATH  Project path (default: current directory)
```

### `devhub unregister`
Remove a project from DevHub registry.

```bash
devhub unregister <NAME>

Arguments:
  NAME  Project name to unregister
```

### `devhub list`
List all registered projects with their paths and status.

```bash
devhub list
```

## Service Control

### `devhub start`
Start project services.

```bash
devhub start [PROJECT] [OPTIONS]

Arguments:
  PROJECT  Project name (optional if in project directory)

Options:
  --service, -s <NAME>  Start only this service
  --all                 Start all registered projects
  --favorites           Start all favorite projects
```

### `devhub stop`
Stop project services.

```bash
devhub stop [PROJECT] [OPTIONS]

Arguments:
  PROJECT  Project name (optional if in project directory)

Options:
  --service, -s <NAME>  Stop only this service
  --all                 Stop all registered projects
  --favorites           Stop all favorite projects
```

### `devhub restart`
Restart project services.

```bash
devhub restart [PROJECT] [OPTIONS]

Arguments:
  PROJECT  Project name (optional if in project directory)

Options:
  --service, -s <NAME>  Restart only this service
  --all                 Restart all registered projects
  --favorites           Restart all favorite projects
```

### `devhub status`
Show status of all projects and their services.

```bash
devhub status
```

### `devhub logs`
View or follow service logs.

```bash
devhub logs <PROJECT> [SERVICE] [--follow]

Arguments:
  PROJECT  Project name
  SERVICE  Service name (optional, shows all if omitted)

Options:
  --follow, -f  Follow log output (like tail -f)
```

## Bulk Operations

### `devhub scan`
Scan directories for projects and optionally register them.

```bash
devhub scan [PATHS...] [OPTIONS]

Arguments:
  PATHS  Directories to scan (default: common dev directories)

Options:
  --depth, -d <N>        Maximum scan depth (default: 2)
  --auto-register, -a    Register discovered projects automatically
  --dry-run              Show what would be discovered without registering
```

## Discovery & Search

### `devhub search`
Fuzzy search for projects by name.

```bash
devhub search <QUERY> [--limit <N>]

Arguments:
  QUERY  Search query (fuzzy matched against project names)

Options:
  --limit, -l <N>  Maximum results to show (default: 10)
```

### `devhub recent`
Show recently used projects.

```bash
devhub recent [--limit <N>]

Options:
  --limit, -l <N>  Maximum results to show (default: 10)
```

## Favorites

### `devhub fav`
Manage favorite projects.

```bash
devhub fav <SUBCOMMAND>

Subcommands:
  add <PROJECT>     Add project to favorites
  remove <PROJECT>  Remove project from favorites
  list              List all favorite projects
  toggle <PROJECT>  Toggle favorite status
```

## Port Management

### `devhub ports`
Show port allocations across all projects.

```bash
devhub ports [OPTIONS]

Options:
  --check, -c    Detect and report port conflicts
  --resolve, -r  Suggest fixes for port conflicts
```

## Environment

### `devhub env`
Show resolved environment variables for a project or service.

```bash
devhub env <PROJECT> [OPTIONS]

Arguments:
  PROJECT  Project name

Options:
  --service, -s <NAME>  Show environment for specific service
  --format, -f <FMT>    Output format: table (default) or export
```

Example usage:
```bash
# Show all services' environment
devhub env my-project

# Show specific service environment
devhub env my-project -s api

# Export format (for sourcing in shell)
devhub env my-project -s api -f export
eval "$(devhub env my-project -s api -f export)"
```

## Developer Shortcuts

### `devhub open`
Open project in browser (uses configured port and health_check endpoint).

```bash
devhub open <PROJECT> [--service <SERVICE>]

Arguments:
  PROJECT  Project name

Options:
  --service, -s <NAME>  Open specific service (default: main service)
```

### `devhub code`
Open project in VS Code.

```bash
devhub code <PROJECT>

Arguments:
  PROJECT  Project name
```

### `devhub path`
Print project path (useful for shell integration).

```bash
devhub path <PROJECT>

Arguments:
  PROJECT  Project name

# Usage with cd:
cd $(devhub path my-project)
```

## Utilities

### `devhub daemon`
Run the REST API server for the web dashboard.

```bash
devhub daemon [--port <PORT>]

Options:
  --port, -p <PORT>  Port to listen on (default: 9876)
```

### `devhub completions`
Generate shell completions.

```bash
devhub completions <SHELL>

Arguments:
  SHELL  Shell type: bash, zsh, fish, elvish, powershell
```

## Common Workflows

### Set up a new project
```bash
cd /path/to/project
devhub discover --dry-run    # Preview
devhub discover              # Generate config
devhub register              # Add to registry
devhub start                 # Start services
```

### Bulk register projects
```bash
devhub scan ~/work --dry-run        # Preview all projects
devhub scan ~/work --auto-register  # Register everything
```

### Quick project switch
```bash
cd $(devhub path other-project)
devhub start
devhub logs -f
```

### Start all favorites
```bash
devhub fav add project-a
devhub fav add project-b
devhub start --favorites
```
