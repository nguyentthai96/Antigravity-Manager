#!/bin/bash
# Wrapper script to launch SQLcl MCP Server with SDKMAN Java
# This ensures Java is available in PATH when Antigravity launches the MCP server

# Load SDKMAN to get Java in PATH
export SDKMAN_DIR="/home/nguyentthai96/.sdkman"
[ -s "$SDKMAN_DIR/bin/sdkman-init.sh" ] && source "$SDKMAN_DIR/bin/sdkman-init.sh"

# Launch SQLcl in MCP mode
exec /sqlcl/bin/sql -mcp
