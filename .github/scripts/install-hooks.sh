#!/bin/bash
# LUMOS Git Hooks Installation Script
# Installs git hooks for automatic .lumos schema validation
# Usage: bash .github/scripts/install-hooks.sh

set -e

# Color codes
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}🪝 Installing LUMOS Git Hooks...${NC}"
echo ""

# Check if we're in a git repository
if [ ! -d ".git" ]; then
    echo -e "${RED}❌ Error: Not in a git repository${NC}"
    echo "Please run this script from the root of the LUMOS repository."
    exit 1
fi

# Check if lumos CLI is installed
if ! command -v lumos &> /dev/null; then
    echo -e "${YELLOW}⚠️  Warning: lumos CLI not found${NC}"
    echo ""
    echo "The git hooks require the LUMOS CLI to be installed."
    echo "Install it with:"
    echo "  cargo install lumos-cli"
    echo ""
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Installation cancelled."
        exit 1
    fi
fi

# Install pre-commit hook
if [ -f ".github/hooks/pre-commit" ]; then
    echo -e "${BLUE}▸${NC} Installing pre-commit hook..."
    cp .github/hooks/pre-commit .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit
    echo -e "  ${GREEN}✓${NC} pre-commit hook installed"
else
    echo -e "${RED}❌ Error: .github/hooks/pre-commit not found${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ Git hooks installed successfully!${NC}"
echo ""
echo "What happens now:"
echo "  • Every commit will run Clippy on staged Rust files"
echo "  • Every commit will check rustfmt formatting"
echo "  • Every commit will validate staged .lumos files"
echo "  • Failed checks will block the commit"
echo "  • You'll see clear error messages if validation fails"
echo ""
echo "To bypass validation (not recommended):"
echo "  git commit --no-verify"
echo ""
echo "To uninstall:"
echo "  rm .git/hooks/pre-commit"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
