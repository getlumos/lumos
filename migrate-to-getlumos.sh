#!/bin/bash
# LUMOS Migration Script: RECTOR-LABS → getlumos
# Automates organization creation, repo transfer, and configuration updates

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
ORG_NAME="getlumos"
OLD_OWNER="RECTOR-LABS"
REPO_NAME="lumos"
EMAIL="rz1989s@gmail.com"
WEBSITE="https://getlumos.dev"

echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  LUMOS Migration: RECTOR-LABS → getlumos              ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════╝${NC}"
echo ""

# Step 1: Check prerequisites
echo -e "${YELLOW}[1/9] Checking prerequisites...${NC}"
if ! command -v gh &> /dev/null; then
    echo -e "${RED}✗ GitHub CLI (gh) not found. Install: brew install gh${NC}"
    exit 1
fi

if ! gh auth status &> /dev/null; then
    echo -e "${RED}✗ Not authenticated with GitHub CLI${NC}"
    echo "Run: gh auth login"
    exit 1
fi
echo -e "${GREEN}✓ GitHub CLI authenticated${NC}"

# Step 2: Verify we're in the right directory
echo -e "${YELLOW}[2/9] Verifying repository location...${NC}"
if [ ! -d ".git" ]; then
    echo -e "${RED}✗ Not in a git repository${NC}"
    exit 1
fi

CURRENT_REMOTE=$(git remote get-url origin 2>/dev/null || echo "")
if [[ "$CURRENT_REMOTE" != *"$OLD_OWNER/$REPO_NAME"* ]]; then
    echo -e "${RED}✗ Not in the lumos repository${NC}"
    echo "Current remote: $CURRENT_REMOTE"
    exit 1
fi
echo -e "${GREEN}✓ In correct repository: $OLD_OWNER/$REPO_NAME${NC}"

# Step 3: Check if organization already exists
echo -e "${YELLOW}[3/9] Checking if organization exists...${NC}"
if gh api "/orgs/$ORG_NAME" &> /dev/null; then
    echo -e "${GREEN}✓ Organization '$ORG_NAME' already exists${NC}"
    read -p "Continue with existing organization? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Aborted."
        exit 1
    fi
else
    echo -e "${YELLOW}⚠ Organization '$ORG_NAME' does not exist${NC}"
    echo -e "${BLUE}Note: Organization creation requires manual step via web UI${NC}"
    echo -e "${BLUE}Visit: https://github.com/organizations/new${NC}"
    echo -e "${BLUE}Organization name: ${GREEN}$ORG_NAME${NC}"
    echo ""
    read -p "Have you created the organization? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Please create the organization first, then re-run this script."
        exit 1
    fi
fi

# Step 4: Update Cargo.toml repository URLs
echo -e "${YELLOW}[4/9] Updating Cargo.toml...${NC}"
if [ -f "Cargo.toml" ]; then
    sed -i.bak "s|https://github.com/$OLD_OWNER/$REPO_NAME|https://github.com/$ORG_NAME/$REPO_NAME|g" Cargo.toml
    sed -i.bak "s|homepage = \"https://github.com/$OLD_OWNER/$REPO_NAME\"|homepage = \"$WEBSITE\"|g" Cargo.toml
    rm Cargo.toml.bak 2>/dev/null || true
    echo -e "${GREEN}✓ Cargo.toml updated${NC}"
else
    echo -e "${YELLOW}⚠ Cargo.toml not found${NC}"
fi

# Step 5: Update CLAUDE.md
echo -e "${YELLOW}[5/9] Updating CLAUDE.md...${NC}"
if [ -f "CLAUDE.md" ]; then
    sed -i.bak "s|https://github.com/$OLD_OWNER/$REPO_NAME|https://github.com/$ORG_NAME/$REPO_NAME|g" CLAUDE.md
    sed -i.bak "s|RECTOR-LABS|$ORG_NAME|g" CLAUDE.md
    rm CLAUDE.md.bak 2>/dev/null || true
    echo -e "${GREEN}✓ CLAUDE.md updated${NC}"
else
    echo -e "${YELLOW}⚠ CLAUDE.md not found${NC}"
fi

# Step 6: Update README.md badges
echo -e "${YELLOW}[6/9] Updating README.md...${NC}"
if [ -f "README.md" ]; then
    sed -i.bak "s|github.com/$OLD_OWNER/$REPO_NAME|github.com/$ORG_NAME/$REPO_NAME|g" README.md
    rm README.md.bak 2>/dev/null || true
    echo -e "${GREEN}✓ README.md updated${NC}"
else
    echo -e "${YELLOW}⚠ README.md not found${NC}"
fi

# Step 7: Update documentation files
echo -e "${YELLOW}[7/9] Updating documentation files...${NC}"
if [ -d "docs" ]; then
    find docs -type f -name "*.md" -exec sed -i.bak "s|$OLD_OWNER/$REPO_NAME|$ORG_NAME/$REPO_NAME|g" {} \;
    find docs -type f -name "*.bak" -delete
    echo -e "${GREEN}✓ Documentation updated${NC}"
fi

# Step 8: Commit changes
echo -e "${YELLOW}[8/9] Committing changes...${NC}"
git add Cargo.toml CLAUDE.md README.md docs/ 2>/dev/null || true
if git diff --staged --quiet; then
    echo -e "${BLUE}ℹ No changes to commit${NC}"
else
    git commit -m "chore: Migrate to $ORG_NAME organization

- Update repository URLs
- Update documentation references
- Prepare for crates.io publishing"
    echo -e "${GREEN}✓ Changes committed${NC}"
fi

# Step 9: Transfer repository
echo -e "${YELLOW}[9/9] Transferring repository to $ORG_NAME...${NC}"
echo -e "${BLUE}ℹ This will transfer $OLD_OWNER/$REPO_NAME → $ORG_NAME/$REPO_NAME${NC}"
read -p "Proceed with repository transfer? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    # Transfer repository using GitHub API
    if gh api --method POST "/repos/$OLD_OWNER/$REPO_NAME/transfer" \
        -f new_owner="$ORG_NAME" \
        -f new_name="$REPO_NAME" 2>&1; then
        echo -e "${GREEN}✓ Repository transferred successfully!${NC}"

        # Update local git remote
        echo -e "${YELLOW}Updating local git remote...${NC}"
        git remote set-url origin "https://github.com/$ORG_NAME/$REPO_NAME.git"
        echo -e "${GREEN}✓ Local remote updated${NC}"

        # Push changes
        echo -e "${YELLOW}Pushing changes to new repository...${NC}"
        git push origin dev
        echo -e "${GREEN}✓ Changes pushed${NC}"
    else
        echo -e "${RED}✗ Repository transfer failed${NC}"
        echo -e "${YELLOW}You may need to transfer manually via GitHub web UI${NC}"
        echo -e "${BLUE}Visit: https://github.com/$OLD_OWNER/$REPO_NAME/settings${NC}"
        exit 1
    fi
else
    echo -e "${YELLOW}⚠ Repository transfer skipped${NC}"
    echo "You can transfer manually later via:"
    echo "  https://github.com/$OLD_OWNER/$REPO_NAME/settings"
fi

# Summary
echo ""
echo -e "${GREEN}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║  Migration Complete! 🎉                                ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}Next steps:${NC}"
echo -e "  1. Verify repository: ${GREEN}https://github.com/$ORG_NAME/$REPO_NAME${NC}"
echo -e "  2. Check GitHub Actions: ${GREEN}https://github.com/$ORG_NAME/$REPO_NAME/actions${NC}"
echo -e "  3. Update any external links pointing to old repo"
echo -e "  4. Ready to publish to crates.io! 🚀"
echo ""
echo -e "${BLUE}Publishing commands:${NC}"
echo -e "  ${YELLOW}cd packages/core && cargo publish${NC}"
echo -e "  ${YELLOW}cd ../cli && cargo publish${NC}"
echo ""
