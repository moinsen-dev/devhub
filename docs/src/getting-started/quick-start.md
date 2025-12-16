# Quick Start Guide

Get DevHub running in under 5 minutes.

## Step 1: Discover Your Projects

DevHub can automatically scan your directories and find projects:

```bash
# Preview what would be discovered (dry run)
devhub scan --dry-run ~/work

# Example output:
# Scanning for projects...
#   → Scanning /Users/you/work...
#
# Scan Results:
#   Rust (23):
#     • my-api - api:8080
#     • cli-tool - no services
#   Node (45):
#     • frontend - web:3000
#     • dashboard - dev:5173
#   Docker (12):
#     • postgres-stack - postgres:5432
```

Happy with what you see? Register them all:

```bash
devhub scan --auto-register ~/work
```

## Step 2: Check Your Projects

```bash
# List all registered projects
devhub list

# Check what's running
devhub status
```

## Step 3: Start a Project

```bash
# Start all services for a project
devhub start my-project

# Watch the output:
#   → Starting my-project...
#   → Starting api on port 8080...
#   ✓ api started (port 8080)
#   → Starting frontend on port 3000...
#   ✓ frontend started (port 3000)
```

## Step 4: Developer Shortcuts

```bash
# Jump to project directory
cd $(devhub path my-project)

# Open in VS Code
devhub code my-project

# Open in browser (if running)
devhub open my-project
```

## Step 5: Check for Port Conflicts

```bash
devhub ports --check

# Port Allocations:
#   Web Frontends (3000-3099)
#     ● 3000 → project-a/frontend
#     ○ 3001 → project-b/frontend
#   APIs (8000-8099)
#     ● 8080 → my-api/api
#
# ✓ No port conflicts detected
```

## Step 6: Launch the Dashboard (Optional)

```bash
# Start the API daemon in background
devhub daemon &

# Start the UI (in devhub-ui directory)
cd devhub-ui && npm install && npm run dev

# Open http://localhost:5173
```

## Common Workflows

### Morning Startup
```bash
devhub start my-main-project
devhub status
devhub open my-main-project
```

### Switch Projects
```bash
devhub stop current-project
devhub start other-project
devhub code other-project
```

### End of Day
```bash
devhub status        # See what's running
devhub stop --all    # Stop everything (coming soon)
```

## Next Steps

- [Your First Project](./first-project.md) - Manual project setup
- [Configuration](../guide/configuration.md) - Customize devhub.toml
- [Dashboard](../guide/dashboard.md) - Web interface guide
