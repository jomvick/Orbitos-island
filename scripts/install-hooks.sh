#!/usr/bin/env bash
set -euo pipefail

AGENTOS_HOOK="${AGENTOS_HOOK:-agentos-hook}"
CLAUDE_CONFIG_DIR="${CLAUDE_CONFIG_DIR:-$HOME/.claude}"
OPENCODE_CONFIG_DIR="${OPENCODE_CONFIG_DIR:-$HOME/.config/opencode}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info()  { echo -e "${GREEN}[install]${NC} $1"; }
warn()  { echo -e "${YELLOW}[warn]${NC} $1"; }
error() { echo -e "${RED}[error]${NC} $1"; }

check_hook_binary() {
    if command -v "$AGENTOS_HOOK" &>/dev/null; then
        info "found hook binary: $(command -v "$AGENTOS_HOOK")"
        return 0
    fi
    if [ -f "$AGENTOS_HOOK" ]; then
        info "found hook binary: $AGENTOS_HOOK"
        return 0
    fi
    error "agentos-hook not found. Set AGENTOS_HOOK or add to PATH"
    return 1
}

install_claude_hooks() {
    info "installing Claude Code hooks..."

    mkdir -p "$CLAUDE_CONFIG_DIR"

    local settings_file="$CLAUDE_CONFIG_DIR/settings.json"
    local settings
    if [ -f "$settings_file" ]; then
        settings=$(cat "$settings_file")
    else
        settings="{}"
    fi

    local hook_config
    hook_config=$(cat <<HOOKJSON
{
  "hooks": {
    "onTaskStart": "$AGENTOS_HOOK --source claude",
    "onTaskComplete": "$AGENTOS_HOOK --source claude",
    "onError": "$AGENTOS_HOOK --source claude",
    "onPermissionRequest": "$AGENTOS_HOOK --source claude",
    "onHeartbeat": "$AGENTOS_HOOK --source claude"
  }
}
HOOKJSON
)

    local merged
    merged=$(echo "$settings" | jq -M --argjson hooks "$(echo "$hook_config" | jq '.hooks')" '.hooks = $hooks' 2>/dev/null || echo "$hook_config")

    echo "$merged" > "$settings_file"
    info "Claude Code hooks installed: $settings_file"
}

install_opencode_hooks() {
    info "installing OpenCode hooks..."

    mkdir -p "$OPENCODE_CONFIG_DIR"

    local config_file="$OPENCODE_CONFIG_DIR/opencode.json"
    local config
    if [ -f "$config_file" ]; then
        config=$(cat "$config_file")
    else
        config='{"hooks":{}}'
    fi

    local hook_entry
    hook_entry=$(cat <<HOOKJSON
{
  "hooks": {
    "preTask": "$AGENTOS_HOOK --source opencode --event",
    "postTask": "$AGENTOS_HOOK --source opencode --event",
    "onPermission": "$AGENTOS_HOOK --source opencode --event",
    "onError": "$AGENTOS_HOOK --source opencode --event"
  }
}
HOOKJSON
)

    local merged
    merged=$(echo "$config" | jq -M --argjson hooks "$(echo "$hook_entry" | jq '.hooks')" '.hooks = $hooks' 2>/dev/null || echo "$hook_entry")

    echo "$merged" > "$config_file"
    info "OpenCode hooks installed: $config_file"
}

install_tmux_hook() {
    info "configuring tmux hook..."

    local tmux_conf="${TMUX_CONF:-$HOME/.tmux.conf}"
    local hook_cmd="set -g @agentos-hook '$AGENTOS_HOOK --source tmux'"

    if [ -f "$tmux_conf" ] && grep -q "agentos-hook" "$tmux_conf" 2>/dev/null; then
        warn "tmux hook already configured in $tmux_conf"
        return 0
    fi

    echo "" >> "$tmux_conf"
    echo "# AgentOS hook" >> "$tmux_conf"
    echo "$hook_cmd" >> "$tmux_conf"
    info "tmux hook added to $tmux_conf (reload: tmux source-file $tmux_conf)"
}

verify_daemon_running() {
    local socket="$HOME/.agentos/run/agentosd.sock"
    if [ -S "$socket" ]; then
        info "daemon socket found at $socket"
        return 0
    fi

    warn "daemon not running. Start with: agentosd"
    warn "hooks will work once daemon is started (fail-open)"
    return 0
}

main() {
    echo "================================================"
    echo "  AgentOS Hook Installer"
    echo "================================================"
    echo ""

    check_hook_binary || exit 1

    install_claude_hooks
    echo ""
    install_opencode_hooks
    echo ""
    install_tmux_hook
    echo ""

    verify_daemon_running

    echo ""
    info "installation complete"
    echo ""
    echo "  Next steps:"
    echo "    1. Start the daemon:  agentosd"
    echo "    2. Launch your AI agent"
    echo "    3. Events will flow: agent → agentos-hook → agentosd → desktop UI"
    echo ""
}

main
