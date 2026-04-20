#!/bin/bash

# Interactive Account Switcher for Antigravity CLI

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCOUNTS_FILE="$SCRIPT_DIR/antigravity_accounts.json"
CLI_BIN="$SCRIPT_DIR/target/release/antigravity-cli"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

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

# Read accounts into array
mapfile -t EMAILS < <(grep -o '"email"[[:space:]]*:[[:space:]]*"[^"]*"' "$ACCOUNTS_FILE" | \
    sed 's/"email"[[:space:]]*:[[:space:]]*"\([^"]*\)"/\1/')

if [ ${#EMAILS[@]} -eq 0 ]; then
    echo -e "${RED}No accounts found in $ACCOUNTS_FILE${NC}"
    exit 1
fi

# Display menu
clear
echo -e "${CYAN}╔════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║   Antigravity Account Switcher (CLI)      ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}Available Accounts:${NC}"
echo ""

for i in "${!EMAILS[@]}"; do
    printf "  ${BLUE}%2d${NC}. %s\n" $((i+1)) "${EMAILS[$i]}"
done

echo ""
echo -e "${YELLOW}Enter account number (or 'q' to quit):${NC} "
read -r choice

# Handle quit
if [ "$choice" = "q" ] || [ "$choice" = "Q" ]; then
    echo "Cancelled."
    exit 0
fi

# Validate input
if ! [[ "$choice" =~ ^[0-9]+$ ]] || [ "$choice" -lt 1 ] || [ "$choice" -gt ${#EMAILS[@]} ]; then
    echo -e "${RED}Invalid selection${NC}"
    exit 1
fi

# Get selected email
SELECTED_EMAIL="${EMAILS[$((choice-1))]}"

# Ask for project ID (optional)
echo ""
echo -e "${YELLOW}Enter GCP Project ID (optional, press Enter to skip):${NC} "
read -r PROJECT_ID

# Confirm
echo ""
echo -e "${GREEN}Switching to: ${SELECTED_EMAIL}${NC}"
if [ -n "$PROJECT_ID" ]; then
    echo -e "${GREEN}Project ID: ${PROJECT_ID}${NC}"
fi
echo ""
echo -e "${YELLOW}Continue? (y/N):${NC} "
read -r confirm

if [ "$confirm" != "y" ] && [ "$confirm" != "Y" ]; then
    echo "Cancelled."
    exit 0
fi

# Build command
CMD="$CLI_BIN --accounts-file $ACCOUNTS_FILE --email $SELECTED_EMAIL"
if [ -n "$PROJECT_ID" ]; then
    CMD="$CMD --project-id $PROJECT_ID"
fi

# Execute
echo ""
echo -e "${CYAN}════════════════════════════════════════════${NC}"
eval "$CMD"

if [ $? -eq 0 ]; then
    echo -e "${CYAN}════════════════════════════════════════════${NC}"
    echo ""
    echo -e "${GREEN}✓ Account switch completed successfully!${NC}"
    echo ""
else
    echo ""
    echo -e "${RED}✗ Account switch failed${NC}"
    exit 1
fi
