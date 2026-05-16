#!/bin/bash

# AgentOS System Sync Diagnostic Battery
# This script tests the entire communication chain

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== AgentOS Diagnostic Battery ===${NC}"

# 1. Check Daemon Process
echo -n "1. Checking agentosd process... "
PID=$(pgrep agentosd)
if [ -z "$PID" ]; then
    echo -e "${RED}FAILED (Not running)${NC}"
else
    echo -e "${GREEN}OK (PID: $PID)${NC}"
fi

# 2. Check Socket File
SOCKET="$HOME/.agentosd.sock"
echo -n "2. Checking IPC socket ($SOCKET)... "
if [ -S "$SOCKET" ]; then
    echo -e "${GREEN}OK (Exists and is a socket)${NC}"
    ls -l "$SOCKET"
else
    echo -e "${RED}FAILED (Not found or not a socket)${NC}"
fi

# 3. Test IPC Communication (Ping)
echo -n "3. Testing IPC Ping-Pong... "
PYTHON_CMD="import socket, json, os; s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM); s.connect(os.path.expanduser('~/.agentosd.sock')); s.sendall(json.dumps({'type':'command','id':'00000000-0000-0000-0000-000000000000','command':{'action':'ping'},'timestamp':''}).encode()+b'\\n'); print(s.recv(1024).decode())"
RESPONSE=$(python3 -c "$PYTHON_CMD" 2>/dev/null)
if [[ $RESPONSE == *"pong"* ]]; then
    echo -e "${GREEN}OK (Received Pong)${NC}"
else
    echo -e "${RED}FAILED (No response)${NC}"
fi

# 4. Test Hook Injection
echo -n "4. Testing Hook -> Daemon relay... "
./agentos/target/debug/agentos-hook -s test-diag -e '{"id":"diag-session","agent":"claude","event":"session_started","session_id":"diag-test","cwd":"'$(pwd)'","timestamp":"2026-05-15T21:00:00Z"}'
if [ $? -eq 0 ]; then
    echo -e "${GREEN}OK (Event sent)${NC}"
else
    echo -e "${RED}FAILED (Hook error)${NC}"
fi

# 5. Check Daemon Logs for Tauri connection
echo -n "5. Checking for active Tauri connections... "
CONN_COUNT=$(grep "client subscribed" agentos/agentosd.log | wc -l)
DISC_COUNT=$(grep "client disconnected" agentos/agentosd.log | wc -l)
ACTIVE=$((CONN_COUNT - DISC_COUNT))
if [ $ACTIVE -gt 0 ]; then
    echo -e "${GREEN}OK ($ACTIVE client(s) connected)${NC}"
else
    echo -e "${RED}WARNING (No active clients detected)${NC}"
fi

echo -e "${BLUE}=== Diagnostic Complete ===${NC}"
