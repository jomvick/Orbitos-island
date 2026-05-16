#!/usr/bin/env bash
set -euo pipefail

HOOK_BINARY="${AGENTOS_HOOK:-agentos-hook}"
CLAUDE_CONFIG_DIR="${CLAUDE_CONFIG_DIR:-$HOME/.claude}"
SETTINGS_FILE="$CLAUDE_CONFIG_DIR/settings.json"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'
info()  { echo -e "${GREEN}[install]${NC} $1"; }
warn()  { echo -e "${YELLOW}[warn]${NC} $1"; }
error() { echo -e "${RED}[error]${NC} $1"; }

# Resolve hook binary path
if ! command -v "$HOOK_BINARY" &>/dev/null; then
    if [ -f "$HOOK_BINARY" ]; then
        HOOK_BINARY=$(realpath "$HOOK_BINARY")
    else
        error "agentos-hook not found. Set AGENTOS_HOOK env or add to PATH"
        exit 1
    fi
else
    HOOK_BINARY=$(command -v "$HOOK_BINARY")
fi

info "using hook binary: $HOOK_BINARY"

# Ensure config directory
mkdir -p "$CLAUDE_CONFIG_DIR"

# Read current settings or create empty
if [ -f "$SETTINGS_FILE" ]; then
    SETTINGS=$(cat "$SETTINGS_FILE")
else
    SETTINGS='{}'
fi

# Define hook events matching Claude Code's hook system
# Based on official Claude Code hook events
read -r -d '' HOOKS_JSON <<'HOOKS' || true
{
  "UserPromptSubmit": [
    { "matcher": "*", "hooks": [{ "type": "command", "command": "BINARY", "timeout": 5 }] }
  ],
  "SessionStart": [
    { "matcher": "*", "hooks": [{ "type": "command", "command": "BINARY", "timeout": 5 }] }
  ],
  "SessionEnd": [
    { "matcher": "*", "hooks": [{ "type": "command", "command": "BINARY", "timeout": 5 }] }
  ],
  "Stop": [
    { "matcher": "*", "hooks": [{ "type": "command", "command": "BINARY", "timeout": 5 }] }
  ],
  "StopFailure": [
    { "matcher": "*", "hooks": [{ "type": "command", "command": "BINARY", "timeout": 5 }] }
  ],
  "PreToolUse": [
    { "matcher": "*", "hooks": [{ "type": "command", "command": "BINARY", "timeout": 5 }] }
  ],
  "PostToolUse": [
    { "matcher": "*", "hooks": [{ "type": "command", "command": "BINARY", "timeout": 5 }] }
  ],
  "PostToolUseFailure": [
    { "matcher": "*", "hooks": [{ "type": "command", "command": "BINARY", "timeout": 5 }] }
  ],
  "PermissionRequest": [
    { "matcher": "*", "hooks": [{ "type": "command", "command": "BINARY", "timeout": 86400 }] }
  ],
  "PermissionDenied": [
    { "matcher": "*", "hooks": [{ "type": "command", "command": "BINARY", "timeout": 5 }] }
  ],
  "Notification": [
    { "matcher": "*", "hooks": [{ "type": "command", "command": "BINARY", "timeout": 5 }] }
  ],
  "SubagentStart": [
    { "matcher": "*", "hooks": [{ "type": "command", "command": "BINARY", "timeout": 5 }] }
  ],
  "SubagentStop": [
    { "matcher": "*", "hooks": [{ "type": "command", "command": "BINARY", "timeout": 5 }] }
  ],
  "PreCompact": [
    { "matcher": "*", "hooks": [{ "type": "command", "command": "BINARY", "timeout": 5 }] }
  ]
}
HOOKS_JSON

# Replace BINARY placeholder with actual path
HOOKS_JSON="${HOOKS_JSON//BINARY/$HOOK_BINARY --source claude}"

# Merge hooks into settings using python3
python3 -c "
import json, sys

settings = json.loads('''$SETTINGS''')
hooks = json.loads('''$HOOKS_JSON''')

# Preserve existing non-hook settings, merge hooks
settings['hooks'] = hooks

json.dump(settings, sys.stdout, indent=2)
" > "$SETTINGS_FILE.tmp" 2>/dev/null

mv "$SETTINGS_FILE.tmp" "$SETTINGS_FILE"

info "Claude Code hooks installed: $SETTINGS_FILE"
echo ""
echo "  Events registered: 14"
echo "  Hook binary: $HOOK_BINARY --source claude"
echo ""
echo "  Restart Claude Code for hooks to take effect."
echo "  To uninstall: remove the 'hooks' key from $SETTINGS_FILE"
