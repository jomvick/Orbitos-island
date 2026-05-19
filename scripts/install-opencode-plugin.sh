#!/usr/bin/env bash
set -euo pipefail

PLUGIN_SRC="${1:-$(dirname "$0")/../plugins/opencode/js/agentos-cli-plugin.js}"
OPENCODE_CONFIG="${OPENCODE_CONFIG_DIR:-$HOME/.config/opencode}"
PLUGIN_DIR="$OPENCODE_CONFIG/plugins"
PLUGIN_DEST="$PLUGIN_DIR/agentos.js"
CONFIG_FILE="$OPENCODE_CONFIG/opencode.json"

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
info()  { echo -e "${GREEN}[install]${NC} $1"; }
error() { echo -e "${RED}[error]${NC} $1"; }

# Resolve plugin source
if [ ! -f "$PLUGIN_SRC" ]; then
    PLUGIN_SRC="$(dirname "$0")/../plugins/opencode/js/agentos-cli-plugin.js"
    if [ ! -f "$PLUGIN_SRC" ]; then
        error "plugin source not found at $PLUGIN_SRC"
        exit 1
    fi
fi

mkdir -p "$PLUGIN_DIR"
cp "$PLUGIN_SRC" "$PLUGIN_DEST"
info "plugin installed: $PLUGIN_DEST"

mkdir -p "$OPENCODE_CONFIG"

if [ -f "$CONFIG_FILE" ]; then
    CONFIG=$(cat "$CONFIG_FILE")
else
    CONFIG='{}'
fi

PLUGIN_URI="file://$PLUGIN_DEST"
python3 -c "
import json, sys
c = json.load(sys.stdin)
# Remove deprecated hooks key (rejected by opencode)
c.pop('hooks', None)
plugins = c.get('plugin', [])
if isinstance(plugins, str):
    plugins = [plugins]
if '$PLUGIN_URI' not in plugins:
    plugins.append('$PLUGIN_URI')
c['plugin'] = plugins
json.dump(c, sys.stdout)
" <<<"$CONFIG" > "$CONFIG_FILE.tmp" && mv "$CONFIG_FILE.tmp" "$CONFIG_FILE"

info "plugin registered in $CONFIG_FILE"
info "Restart opencode for the plugin to take effect."
