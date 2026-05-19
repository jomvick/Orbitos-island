#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"

echo "=== Orbitos Island — starting all services ==="

# 1. Clean up any stale processes from previous runs
echo "[1/3] Cleaning up stale processes..."
pkill -f "agentosd" 2>/dev/null || true
pkill -f "tauri" 2>/dev/null || true
pkill -f "vite" 2>/dev/null || true
sleep 1

echo "[2/3] Starting daemon..."
rm -f "$HOME/.agentos/run/agentosd.sock"
mkdir -p "$HOME/.local/share/agentos"
nohup "$ROOT/target/debug/agentosd" > /tmp/agentos-daemon.log 2>&1 &
sleep 1
echo "  daemon pid: $(pgrep -f agentosd | head -1)"
echo "  socket: $HOME/.agentos/run/agentosd.sock"

# 3. Desktop app (tauri dev will start the Vite dev server via beforeDevCommand)
echo "[3/3] Launching desktop app..."
cd "$ROOT/apps/desktop"
npm run tauri dev &
echo "  desktop: launching..."

echo ""
echo "=== All services running ==="
echo "  Daemon:    $HOME/.agentos/run/agentosd.sock"
echo "  Frontend:  http://localhost:1420/"
echo "  Logs:      /tmp/agentos-daemon.log"
echo "             /tmp/agentos-vite.log"
