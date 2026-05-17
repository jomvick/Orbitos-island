#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"

echo "=== Orbitos Island — starting all services ==="

# 1. Daemon
echo "[1/3] Starting daemon..."
pkill -f "agentosd" 2>/dev/null || true
rm -f "$HOME/.agentos/run/agentosd.sock"
nohup "$ROOT/target/debug/agentosd" --db-in-memory > /tmp/agentos-daemon.log 2>&1 &
sleep 1
echo "  daemon pid: $(pgrep -f agentosd | head -1)"
echo "  socket: $HOME/.agentos/run/agentosd.sock"

# 2. Desktop app (tauri dev will start the Vite dev server via beforeDevCommand)
echo "[2/2] Launching desktop app..."
cd "$ROOT/apps/desktop"
npm run tauri dev &
echo "  desktop: launching..."

echo ""
echo "=== All services running ==="
echo "  Daemon:    $HOME/.agentos/run/agentosd.sock"
echo "  Frontend:  http://localhost:1420/"
echo "  Logs:      /tmp/agentos-daemon.log"
echo "             /tmp/agentos-vite.log"
