#!/bin/bash

# Cross-compile Antigravity CLI for Windows from Linux
# This script builds a Windows executable on Ubuntu/Linux

set -e

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║     Cross-Compiling Antigravity CLI for Windows (x86_64)      ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

# Check if rustup is installed
if ! command -v rustup &> /dev/null; then
    echo -e "${RED}Error: rustup is not installed${NC}"
    echo "Install it from: https://rustup.rs/"
    exit 1
fi

# Step 1: Add Windows target
echo -e "${CYAN}[1/4] Adding Windows target (x86_64-pc-windows-gnu)...${NC}"
rustup target add x86_64-pc-windows-gnu

# Step 2: Install MinGW cross-compiler
echo ""
echo -e "${CYAN}[2/4] Checking for MinGW cross-compiler...${NC}"
if ! command -v x86_64-w64-mingw32-gcc &> /dev/null; then
    echo -e "${YELLOW}MinGW not found. Installing...${NC}"
    
    # Detect package manager
    if command -v apt-get &> /dev/null; then
        sudo apt-get update
        sudo apt-get install -y mingw-w64
    elif command -v dnf &> /dev/null; then
        sudo dnf install -y mingw64-gcc
    elif command -v pacman &> /dev/null; then
        sudo pacman -S --noconfirm mingw-w64-gcc
    else
        echo -e "${RED}Could not detect package manager. Please install mingw-w64 manually.${NC}"
        exit 1
    fi
else
    echo -e "${GREEN}MinGW already installed ✓${NC}"
fi

# Step 3: Configure cargo for cross-compilation
echo ""
echo -e "${CYAN}[3/4] Configuring cargo for Windows cross-compilation...${NC}"
mkdir -p ~/.cargo
cat > ~/.cargo/config.toml << 'EOF'
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
ar = "x86_64-w64-mingw32-ar"
EOF
echo -e "${GREEN}Cargo configured ✓${NC}"

# Step 4: Build for Windows
echo ""
echo -e "${CYAN}[4/4] Building for Windows...${NC}"
echo -e "${YELLOW}This may take a few minutes...${NC}"
echo ""

cargo build --release --target x86_64-pc-windows-gnu

# Check if build succeeded
if [ -f "target/x86_64-pc-windows-gnu/release/antigravity-cli.exe" ]; then
    echo ""
    echo -e "${GREEN}════════════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}✓ Build successful!${NC}"
    echo -e "${GREEN}════════════════════════════════════════════════════════════════${NC}"
    echo ""
    echo -e "${CYAN}Windows executable location:${NC}"
    echo "  target/x86_64-pc-windows-gnu/release/antigravity-cli.exe"
    echo ""
    
    # Get file size
    SIZE=$(du -h "target/x86_64-pc-windows-gnu/release/antigravity-cli.exe" | cut -f1)
    echo -e "${CYAN}File size:${NC} $SIZE"
    echo ""
    
    # Create a distribution folder
    echo -e "${CYAN}Creating distribution package...${NC}"
    DIST_DIR="antigravity-cli-windows"
    rm -rf "$DIST_DIR"
    mkdir -p "$DIST_DIR"
    
    # Copy executable
    cp "target/x86_64-pc-windows-gnu/release/antigravity-cli.exe" "$DIST_DIR/"
    
    # Copy PowerShell script
    cp "switch-account.ps1" "$DIST_DIR/"
    
    # Copy accounts file
    cp "antigravity_accounts.json" "$DIST_DIR/"
    
    # Copy documentation
    cp README.md QUICKSTART.md USAGE_EXAMPLES.md "$DIST_DIR/" 2>/dev/null || true
    
    # Create Windows-specific README
    cat > "$DIST_DIR/README-WINDOWS.txt" << 'WINEOF'
Antigravity CLI for Windows
============================

Quick Start:
1. Open PowerShell in this directory
2. Run: .\switch-account.ps1 -Email "your-email@example.com"

Or use the executable directly:
.\antigravity-cli.exe --accounts-file .\antigravity_accounts.json --email "your-email@example.com"

For more information, see README.md

WINEOF
    
    # Create a zip file
    if command -v zip &> /dev/null; then
        ZIP_FILE="antigravity-cli-windows-x64.zip"
        rm -f "$ZIP_FILE"
        zip -r "$ZIP_FILE" "$DIST_DIR"
        echo ""
        echo -e "${GREEN}Distribution package created:${NC}"
        echo "  $ZIP_FILE"
    fi
    
    echo ""
    echo -e "${CYAN}Transfer to Windows:${NC}"
    echo "  1. Copy the '$DIST_DIR' folder to your Windows machine"
    echo "  2. Or download: $ZIP_FILE"
    echo ""
    echo -e "${CYAN}Run on Windows:${NC}"
    echo "  PowerShell> cd $DIST_DIR"
    echo "  PowerShell> .\switch-account.ps1 -Email \"pphstory@gmail.com\""
    echo ""
else
    echo ""
    echo -e "${RED}✗ Build failed${NC}"
    exit 1
fi
