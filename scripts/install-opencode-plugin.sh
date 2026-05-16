#!/usr/bin/env bash
set -euo pipefail

PLUGIN_SRC="${1:-$(dirname "$0")/../plugins/opencode/js/agentos-opencode.js}"
OPENCODE_CONFIG="${OPENCODE_CONFIG_DIR:-$HOME/.config/opencode}"
PLUGIN_DIR="$OPENCODE_CONFIG/plugins"
PLUGIN_DEST="$PLUGIN_DIR/agentos.js"
CONFIG_FILE="$OPENCODE_CONFIG/config.json"

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
info()  { echo -e "${GREEN}[install]${NC} $1"; }
error() { echo -e "${RED}[error]${NC} $1"; }

# Resolve plugin source
if [ ! -f "$PLUGIN_SRC" ]; then
    # Try relative from project root
    PLUGIN_SRC="$(dirname "$0")/../plugins/opencode/js/agentos-opencode.js"
    if [ ! -f "$PLUGIN_SRC" ]; then
        error "plugin source not found at $PLUGIN_SRC"
        exit 1
    fi
fi

# Create plugin directory
mkdir -p "$PLUGIN_DIR"

# Copy plugin
cp "$PLUGIN_SRC" "$PLUGIN_DEST"
info "plugin installed: $PLUGIN_DEST"

# Update config.json
mkdir -p "$OPENCODE_CONFIG"

if [ -f "$CONFIG_FILE" ]; then
    CONFIG=$(cat "$CONFIG_FILE")
else
    CONFIG='{}'
fi

# Add plugin entry if not already present
PLUGIN_URI="file://$PLUGIN_DEST"
if echo "$CONFIG" | python3 -c "
import json, sys
c = json.load(sys.stdin)
plugins = c.get('plugin', [])
if isinstance(plugins, str):
    plugins = [plugins]
if '$PLUGIN_URI' in plugins:
    sys.exit(0)  # already registered
plugins.append('$PLUGIN_URI')
c['plugin'] = plugins
json.dump(c, sys.stdout)
" 2>/dev/null > "$CONFIG_FILE.tmp"; then
    mv "$CONFIG_FILE.tmp" "$CONFIG_FILE"
    info "plugin registered in config.json"
else
    warn "could not update config.json, manual step required:"
    warn "  add \"$PLUGIN_URI\" to the \"plugin\" array in $CONFIG_FILE"
fi

# Verify
if [ -f "$PLUGIN_DEST" ]; then
    info "OpenCode plugin installed successfully"
    echo ""
    echo "  Plugin: $PLUGIN_DEST"
    echo "  Config: $CONFIG_FILE"
    echo ""
    echo "  Restart OpenCode for the plugin to take effect."
fi
