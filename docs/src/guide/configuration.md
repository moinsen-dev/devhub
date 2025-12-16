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
environment variables     (runtime overrides)
```

## Quick Links

- [devhub.toml Reference](./devhub-toml.md) - Project configuration
- [Global Config](./global-config.md) - System-wide settings
