#!/bin/bash
#
# Install DevHub skill for Claude Code
# This copies the skill files to your personal skills directory
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
SOURCE_DIR="$REPO_ROOT/.claude/skills/devhub"
TARGET_DIR="$HOME/.claude/skills/devhub"

echo "DevHub Skill Installer"
echo "======================"
echo ""

# Check if source exists
if [ ! -d "$SOURCE_DIR" ]; then
    echo -e "${RED}Error: Source skill directory not found at $SOURCE_DIR${NC}"
    exit 1
fi

# Check if SKILL.md exists
if [ ! -f "$SOURCE_DIR/SKILL.md" ]; then
    echo -e "${RED}Error: SKILL.md not found in $SOURCE_DIR${NC}"
    exit 1
fi

# Create target directory
echo "Creating directory: $TARGET_DIR"
mkdir -p "$TARGET_DIR"

# Copy files
echo "Copying skill files..."
cp -v "$SOURCE_DIR/SKILL.md" "$TARGET_DIR/"
cp -v "$SOURCE_DIR/COMMANDS.md" "$TARGET_DIR/"
cp -v "$SOURCE_DIR/CONFIG-FORMAT.md" "$TARGET_DIR/"
cp -v "$SOURCE_DIR/EXAMPLES.md" "$TARGET_DIR/"

echo ""
echo -e "${GREEN}DevHub skill installed successfully!${NC}"
echo ""
echo "The skill is now available at: $TARGET_DIR"
echo ""
echo "To verify, start Claude Code and ask:"
echo "  'What skills are available?'"
echo ""
echo "Or try the skill with:"
echo "  'Set up this project for DevHub'"
echo "  'Create a devhub.toml for this project'"
