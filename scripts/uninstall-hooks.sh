#!/usr/bin/env bash
set -euo pipefail

CLAUDE_CONFIG_DIR="${CLAUDE_CONFIG_DIR:-$HOME/.claude}"
OPENCODE_CONFIG_DIR="${OPENCODE_CONFIG_DIR:-$HOME/.config/opencode}"

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

info()  { echo -e "${GREEN}[uninstall]${NC} $1"; }
error() { echo -e "${RED}[error]${NC} $1"; }

remove_claude_hooks() {
    local settings_file="$CLAUDE_CONFIG_DIR/settings.json"
    if [ -f "$settings_file" ]; then
        local cleaned
        cleaned=$(cat "$settings_file" | jq 'del(.hooks)' 2>/dev/null || echo "{}")
        echo "$cleaned" > "$settings_file"
        info "removed Claude Code hooks"
    fi
}

remove_opencode_hooks() {
    local config_file="$OPENCODE_CONFIG_DIR/opencode.json"
    local plugin_file="$OPENCODE_CONFIG_DIR/plugins/agentos.js"
    local plugin_uri="file://$plugin_file"

    if [ -f "$plugin_file" ]; then
        rm "$plugin_file"
        info "removed OpenCode CLI plugin file"
    fi

    if [ -f "$config_file" ]; then
        local cleaned
        cleaned=$(cat "$config_file" | jq \
          --arg uri "$plugin_uri" \
          'del(.hooks) | .plugin = ((.plugin // []) | map(select(. != $uri))) | if .plugin == [] then del(.plugin) else . end' \
          2>/dev/null || echo "{}")
        echo "$cleaned" > "$config_file"
        info "removed OpenCode hooks and plugin reference"
    fi
}

main() {
    echo "================================================"
    echo "  AgentOS Hook Uninstaller"
    echo "================================================"
    echo ""

    remove_claude_hooks
    remove_opencode_hooks

    echo ""
    info "hooks removed"
    echo ""
}

main
