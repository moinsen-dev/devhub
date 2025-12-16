# Contributing to DevHub

Thank you for your interest in contributing to DevHub! This document provides guidelines and information for contributors.

## Code of Conduct

Be respectful and inclusive. We're all here to build great software together.

## Getting Started

### Prerequisites

- Rust 1.75+ (`rustup update stable`)
- Node.js 22+ (`nvm install 22`)
- Docker (for proxy and containerized testing)

### Development Setup

```bash
# Clone the repository
git clone https://github.com/moinsen-dev/devhub.git
cd devhub

# Build Rust CLI
cargo build

# Set up dashboard UI
cd devhub-ui
npm install
cd ..

# Run in development mode
cargo run -- daemon &
cd devhub-ui && npm run dev
```

### Running Tests

```bash
# Rust tests
cargo test

# Check formatting
cargo fmt -- --check

# Lint
cargo clippy -- -D warnings
```

## How to Contribute

### Reporting Bugs

1. Check existing issues first
2. Include reproduction steps
3. Include your environment (OS, Rust version, etc.)
4. Include relevant logs

### Suggesting Features

1. Open an issue with the "enhancement" label
2. Describe the use case
3. Explain why existing features don't solve it

### Pull Requests

1. **Fork** the repository
2. **Create a branch** (`git checkout -b feature/my-feature`)
3. **Make changes** following our code style
4. **Test** your changes
5. **Commit** with clear messages
6. **Push** and open a PR

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add Python project discovery
fix: handle IPv6 port detection
docs: update README installation steps
refactor: simplify service startup logic
test: add integration tests for API
```

## Code Style

### Rust

- Follow `rustfmt` defaults
- Use `clippy` with `-D warnings`
- Document public APIs with `///` comments
- Handle errors properly (no `.unwrap()` in library code)

### TypeScript/Svelte

- Follow ESLint/Prettier defaults
- Use TypeScript strict mode
- Prefer `const` over `let`

## Architecture Overview

```
src/
├── main.rs        # CLI entry point (clap)
├── api.rs         # REST API handlers (axum)
├── process.rs     # Service lifecycle management
├── discovery.rs   # Project type detection
├── manifest.rs    # devhub.toml parsing
├── registry.rs    # Project registry (~/.devhub/)
├── caddy.rs       # Reverse proxy config generation
└── config.rs      # Global configuration
```

### Key Design Decisions

1. **Native processes over containers** - DevHub manages processes directly for faster startup and simpler debugging
2. **TOML configuration** - Familiar to Rust developers, simpler than YAML
3. **Caddy for proxying** - Modern, automatic HTTPS, great developer experience
4. **SvelteKit for UI** - Lightweight, fast, excellent DX

## Areas for Contribution

### Good First Issues

- Add support for new project types (Ruby, PHP, etc.)
- Improve error messages
- Add more CLI output colors/formatting
- Documentation improvements

### Intermediate

- WebSocket support for real-time log streaming
- Service health check improvements
- Dashboard features (log viewer, metrics)

### Advanced

- Windows support
- Distributed mode (manage remote services)
- Plugin system for custom service types

## Questions?

Open an issue or reach out to the maintainers. We're happy to help!
