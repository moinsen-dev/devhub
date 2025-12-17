#!/bin/bash
# Install git hooks for DevHub development
#
# Usage: ./scripts/install-hooks.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
HOOKS_DIR="$REPO_ROOT/.git/hooks"

echo "Installing git hooks..."

# Install pre-commit hook
cp "$SCRIPT_DIR/pre-commit" "$HOOKS_DIR/pre-commit"
chmod +x "$HOOKS_DIR/pre-commit"
echo "✓ Installed pre-commit hook"

echo ""
echo "Done! Git hooks installed successfully."
echo ""
echo "The pre-commit hook will run:"
echo "  • cargo fmt --check (formatting)"
echo "  • cargo clippy -- -D warnings (linting)"
echo ""
echo "To bypass the hook temporarily: git commit --no-verify"
