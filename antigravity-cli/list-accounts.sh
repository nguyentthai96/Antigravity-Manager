#!/bin/bash

# List all available accounts from antigravity_accounts.json

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCOUNTS_FILE="$SCRIPT_DIR/antigravity_accounts.json"

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

if [ ! -f "$ACCOUNTS_FILE" ]; then
    echo -e "${YELLOW}Accounts file not found: $ACCOUNTS_FILE${NC}"
    exit 1
fi

echo -e "${GREEN}Available Accounts:${NC}"
echo ""

# Try with jq first (prettier output)
if command -v jq &> /dev/null; then
    jq -r '.[] | "\(.email)"' "$ACCOUNTS_FILE" | nl -w2 -s'. '
else
    # Fallback to grep
    grep -o '"email"[[:space:]]*:[[:space:]]*"[^"]*"' "$ACCOUNTS_FILE" | \
        sed 's/"email"[[:space:]]*:[[:space:]]*"\([^"]*\)"/\1/' | \
        nl -w2 -s'. '
fi

echo ""
echo -e "${CYAN}Usage:${NC}"
echo "  ./switch-account.sh <email>"
echo ""
echo -e "${CYAN}Example:${NC}"
echo "  ./switch-account.sh pphstory@gmail.com"
