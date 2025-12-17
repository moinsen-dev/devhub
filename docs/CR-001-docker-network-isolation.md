# CR-001: Docker Network Isolation for Zero Port Conflicts

> **Status**: Proposed
> **Created**: 2025-12-17
> **Priority**: High
> **Affects**: caddy.rs, process.rs, manifest.rs, discovery.rs

---

## Problem Statement

DevHub currently requires **unique host ports** for every service across all projects:

```
moinsen-pub API  → host:8081
egb API          → host:8080
another-project  → host:8082  ← Manual assignment, prone to conflicts
```

This approach:
1. **Doesn't scale** - With 100+ projects, port management becomes chaos
2. **Creates conflicts** - Port 8080 free now ≠ free in an hour
3. **Requires manual intervention** - Editing devhub.toml to avoid clashes
4. **Defeats the purpose** - DevHub should eliminate this friction, not manage it

---

## Desired State

Every project uses **standard internal ports** with Docker network isolation:

```
moinsen-pub:
  api      → container moinsen-pub-api:8080    (internal)
  dashboard → container moinsen-pub-web:3000   (internal)

egb:
  api      → container egb-api:8080            (internal)
  dashboard → container egb-web:3000           (internal)

Caddy routes:
  api.moinsen-pub.localhost → moinsen-pub-api:8080
  moinsen-pub.localhost     → moinsen-pub-web:3000
  api.egb.localhost         → egb-api:8080
  egb.localhost             → egb-web:3000
```

**Zero port conflicts. Ever.**

---

## Solution Architecture

### Network Topology

```
┌─────────────────────────────────────────────────────────────────────┐
│                     shared_proxy (Docker network)                    │
│                                                                      │
│  ┌──────────────┐                                                   │
│  │ caddy-proxy  │ ← Routes all *.localhost traffic                  │
│  │   :80/:443   │                                                   │
│  └──────┬───────┘                                                   │
│         │                                                           │
│    ┌────┴────┬─────────────┬─────────────┐                         │
│    │         │             │             │                          │
│    ▼         ▼             ▼             ▼                          │
│ ┌──────┐ ┌──────┐     ┌──────┐     ┌──────┐                        │
│ │ pg   │ │minio │     │ redis│     │ ...  │  ← Shared infra        │
│ │:5432 │ │:9000 │     │:6379 │     │      │    (one instance)      │
│ └──────┘ └──────┘     └──────┘     └──────┘                        │
└─────────────────────────────────────────────────────────────────────┘
         │                    │
         │ DNS/container name │
         ▼                    ▼
┌─────────────────┐  ┌─────────────────┐
│ moinsen-pub-net │  │    egb-net      │   ← Per-project networks
│                 │  │                 │
│ ┌─────────────┐ │  │ ┌─────────────┐ │
│ │moinsen-pub  │ │  │ │  egb-api    │ │
│ │   -api:8080 │ │  │ │    :8080    │ │
│ └─────────────┘ │  │ └─────────────┘ │
│ ┌─────────────┐ │  │ ┌─────────────┐ │
│ │moinsen-pub  │ │  │ │  egb-web    │ │
│ │   -web:3000 │ │  │ │    :3000    │ │
│ └─────────────┘ │  │ └─────────────┘ │
└─────────────────┘  └─────────────────┘
```

### Key Design Decisions

1. **Shared infrastructure services** - PostgreSQL, MinIO, Redis run once in `shared_proxy` network
2. **Per-project application networks** - Each project's services in isolated network
3. **Dual-network attachment** - Project services attach to both their network AND shared_proxy
4. **Container naming convention** - `{project}-{service}` for predictable routing
5. **Standard internal ports** - API=8080, Web=3000, etc. (configurable defaults)

---

## Implementation Plan

### Phase 1: Shared Infrastructure Layer

**New command**: `devhub infra`

```bash
devhub infra start      # Start shared postgres, minio, redis
devhub infra stop       # Stop shared infrastructure
devhub infra status     # Show shared service status
```

**New file**: `~/.devhub/infrastructure/docker-compose.yml`

```yaml
services:
  postgres:
    image: postgres:16-alpine
    container_name: devhub-postgres
    networks:
      - shared_proxy
    ports: []  # No host ports exposed!

  minio:
    image: minio/minio:latest
    container_name: devhub-minio
    networks:
      - shared_proxy

  redis:
    image: redis:7-alpine
    container_name: devhub-redis
    networks:
      - shared_proxy

networks:
  shared_proxy:
    external: true
```

### Phase 2: Project Containerization

**New manifest field**: `mode`

```toml
[project]
name = "moinsen-pub"
mode = "container"  # NEW: "native" (default) | "container" | "hybrid"

[[services]]
name = "api"
type = "dart"           # NEW: language-specific types
image = "dart:3.5"      # NEW: base image for containerization
command = "dart run bin/server.dart"
port = 8080             # Standard port - no conflicts!
```

**Auto-generated docker-compose** (in `.devhub/` per project):

```yaml
# .devhub/docker-compose.generated.yml
services:
  api:
    build:
      context: ./backend
      dockerfile: .devhub/Dockerfile.api
    container_name: moinsen-pub-api
    networks:
      - shared_proxy
      - default
    environment:
      - POSTGRES_HOST=devhub-postgres
      - MINIO_ENDPOINT=devhub-minio:9000
```

### Phase 3: Caddy Configuration Updates

**Update caddy.rs**:

```rust
// Current (problematic):
format!("reverse_proxy host.docker.internal:{}", service.port)

// New (network-aware):
fn generate_reverse_proxy(project: &str, service: &Service, mode: &ProjectMode) -> String {
    match mode {
        ProjectMode::Container => {
            let container = format!("{}-{}", project, service.name);
            format!("reverse_proxy {}:{}", container, service.port)
        }
        ProjectMode::Native => {
            // Fallback for native mode - still needs unique ports
            format!("reverse_proxy host.docker.internal:{}", service.port)
        }
    }
}
```

### Phase 4: Service Discovery & Environment

**Automatic service discovery**:

```toml
# Projects can reference shared infra by name
[environment]
POSTGRES_HOST = "devhub-postgres"      # Auto-resolved
POSTGRES_PORT = "5432"
MINIO_ENDPOINT = "devhub-minio:9000"
REDIS_URL = "redis://devhub-redis:6379"
```

**DevHub injects these automatically** when `mode = "container"`.

---

## Migration Path

### For existing projects:

1. **No breaking changes** - `mode = "native"` is default, existing configs work
2. **Opt-in containerization** - Add `mode = "container"` to devhub.toml
3. **Gradual migration** - Projects can switch one at a time

### For new projects:

1. **`devhub init --container`** - Scaffolds container-ready config
2. **Auto-detect Dockerfile** - If present, suggest container mode

---

## CLI Changes

```bash
# Infrastructure management
devhub infra start [service...]    # Start shared services
devhub infra stop [service...]     # Stop shared services
devhub infra status                # Show infrastructure status
devhub infra logs [service] -f     # View infrastructure logs

# Project mode switching
devhub containerize [project]      # Convert project to container mode
devhub native [project]            # Switch back to native mode

# Network inspection
devhub network status              # Show all Docker networks
devhub network inspect <project>   # Show project's network topology
```

---

## Configuration Changes

### Global config (`~/.devhub/config.toml`)

```toml
[infrastructure]
postgres = true         # Enable shared PostgreSQL
minio = true           # Enable shared MinIO
redis = false          # Disable shared Redis

[defaults]
mode = "container"     # Default mode for new projects
api_port = 8080        # Standard API port
web_port = 3000        # Standard web port
```

### Project manifest additions

```toml
[project]
name = "moinsen-pub"
mode = "container"              # NEW

[[services]]
name = "api"
type = "dart"
image = "dart:3.5"              # NEW: for auto-containerization
dockerfile = "backend/Dockerfile"  # NEW: custom Dockerfile
port = 8080
networks = ["shared_proxy"]     # NEW: additional networks
depends_on_infra = ["postgres", "minio"]  # NEW: infra dependencies
```

---

## Benefits

| Before | After |
|--------|-------|
| Manual port assignment | Standard ports everywhere |
| Port conflicts between projects | Zero conflicts possible |
| `host.docker.internal:8081` | `moinsen-pub-api:8080` |
| Edit devhub.toml per project | Just works |
| Each project needs own postgres | Shared infrastructure |
| 100 projects = 100 postgres instances | 1 shared postgres |

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Docker overhead for simple projects | Keep `native` mode as option |
| Learning curve | Clear migration docs, auto-detection |
| Dockerfile maintenance | Auto-generate from language detection |
| Network complexity | `devhub network` commands for debugging |

---

## Success Criteria

1. `devhub start project-a && devhub start project-b` - Both APIs on port 8080, no conflict
2. `curl http://api.project-a.localhost` - Routes correctly
3. `curl http://api.project-b.localhost` - Routes correctly
4. Shared PostgreSQL accessible from both projects
5. Zero manual port configuration required

---

## Implementation Order

1. **Week 1**: Shared infrastructure layer (`devhub infra`)
2. **Week 2**: Container mode for services (auto-generated docker-compose)
3. **Week 3**: Caddy config updates (container name routing)
4. **Week 4**: Migration tooling (`devhub containerize`)
5. **Week 5**: Documentation & testing

---

## Related Files to Modify

- `src/caddy.rs` - Network-aware reverse proxy config
- `src/process.rs` - Docker container spawning
- `src/manifest.rs` - New fields (mode, image, networks)
- `src/discovery.rs` - Dockerfile detection
- `src/main.rs` - New `infra` subcommand
- `proxy/docker-compose.yml` - Shared infrastructure
- `README.md` - Updated architecture docs

---

*"Stop assigning ports. Start shipping code."*
