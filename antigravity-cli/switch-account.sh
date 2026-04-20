#!/bin/bash

# Antigravity Account Switcher Helper Script
# Usage: ./switch-account.sh <email> [project_id]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCOUNTS_FILE="$SCRIPT_DIR/antigravity_accounts.json"
CLI_BIN="$SCRIPT_DIR/target/release/antigravity-cli"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if email is provided
if [ -z "$1" ]; then
    echo -e "${RED}Error: Email address required${NC}"
    echo "Usage: $0 <email> [project_id]"
    echo ""
    echo "Available accounts:"
    if [ -f "$ACCOUNTS_FILE" ]; then
        jq -r '.[].email' "$ACCOUNTS_FILE" 2>/dev/null || cat "$ACCOUNTS_FILE"
    else
        echo -e "${RED}Accounts file not found: $ACCOUNTS_FILE${NC}"
    fi
    exit 1
fi

EMAIL="$1"
PROJECT_ID="${2:-}"

# Check if accounts file exists
if [ ! -f "$ACCOUNTS_FILE" ]; then
    echo -e "${RED}Error: Accounts file not found: $ACCOUNTS_FILE${NC}"
    exit 1
fi

# Check if CLI binary exists
if [ ! -f "$CLI_BIN" ]; then
    echo -e "${YELLOW}CLI binary not found. Building...${NC}"
    cd "$SCRIPT_DIR"
    cargo build --release
    if [ $? -ne 0 ]; then
        echo -e "${RED}Build failed${NC}"
        exit 1
    fi
fi

# Build command
CMD="$CLI_BIN --accounts-file $ACCOUNTS_FILE --email $EMAIL"
if [ -n "$PROJECT_ID" ]; then
    CMD="$CMD --project-id $PROJECT_ID"
fi

echo -e "${GREEN}Switching to account: $EMAIL${NC}"
if [ -n "$PROJECT_ID" ]; then
    echo -e "${GREEN}Using project ID: $PROJECT_ID${NC}"
fi
echo ""

# Execute
eval "$CMD"

if [ $? -eq 0 ]; then
    echo ""
    echo -e "${GREEN}✓ Account switch completed successfully!${NC}"
else
    echo ""
    echo -e "${RED}✗ Account switch failed${NC}"
    exit 1
fi
