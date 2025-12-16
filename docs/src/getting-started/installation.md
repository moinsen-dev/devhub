# Installation

There are several ways to install DevHub.

## Quick Install (Recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/moinsen-dev/devhub/main/install/install.sh | bash
```

This will:
- Detect your OS and architecture
- Download the latest release
- Install to `~/.local/bin`
- Provide instructions for shell completions

## Homebrew (macOS)

```bash
brew tap moinsen-dev/tap
brew install devhub
```

## Cargo (From Source)

If you have Rust installed:

```bash
cargo install devhub
```

Or build from source:

```bash
git clone https://github.com/moinsen-dev/devhub.git
cd devhub
cargo build --release
cp target/release/devhub ~/.local/bin/
```

## Pre-built Binaries

Download from [GitHub Releases](https://github.com/moinsen-dev/devhub/releases):

| Platform | Download |
|----------|----------|
| macOS (Apple Silicon) | `devhub-aarch64-apple-darwin.tar.gz` |
| macOS (Intel) | `devhub-x86_64-apple-darwin.tar.gz` |
| Linux (x64) | `devhub-x86_64-unknown-linux-gnu.tar.gz` |
| Linux (ARM64) | `devhub-aarch64-unknown-linux-gnu.tar.gz` |

## Verify Installation

```bash
devhub --version
# devhub 0.1.0

devhub --help
# Shows all available commands
```

## Shell Completions

For the best experience, install shell completions:

### Zsh (macOS default)

```bash
# Create completions directory if needed
mkdir -p ~/.zfunc

# Generate completions
devhub completions zsh > ~/.zfunc/_devhub

# Add to ~/.zshrc if not already there
echo 'fpath=(~/.zfunc $fpath)' >> ~/.zshrc
echo 'autoload -Uz compinit && compinit' >> ~/.zshrc

# Reload shell
source ~/.zshrc
```

### Bash

```bash
mkdir -p ~/.local/share/bash-completion/completions
devhub completions bash > ~/.local/share/bash-completion/completions/devhub
```

### Fish

```bash
devhub completions fish > ~/.config/fish/completions/devhub.fish
```

## Next Steps

- [Quick Start Guide](./quick-start.md) - Get up and running in 5 minutes
- [Your First Project](./first-project.md) - Register and manage your first project
