# Architecture

AgentOS is a Linux-native cockpit for AI coding agents. It centralizes monitoring, notifications, multi-agent management, terminal navigation, and usage analytics.

## Overview

```
AI Agents (OpenCode, Claude, Codex, Aider, Gemini, …)
    ↓
Hooks / Plugins (agentos-hook)
    ↓
Local Event Bus (tokio::sync::broadcast)
    ↓
Core Daemon (agentosd)
    ↓
Desktop UI (Tauri + React)
    ↓
Notifications / Overlay / Analytics
```

## Components

### daemon-core
Shared types and core logic: events, sessions, state machine, notifications, terminal integration, plugin traits.

### agentosd
Main daemon binary. Owns `SessionState`, `EventBus`, `AgentRegistry`, SQLite persistence, and Unix socket IPC.

### agentos-hook
Ultra-lightweight CLI hook that agents call to emit events: `agentos-hook --event '{"type":"session_start",…}'`

### agentos-ipc
Unix socket protocol between daemon and clients (desktop, CLI). Uses length-prefixed JSON messages (max 1MB).

### agentos-storage
SQLite persistence for sessions, events, and analytics.

### Plugins
Agent-specific parsing crates that normalize each agent's event format into `UniversalEvent`.

### Desktop App (Tauri + React)
Floating HUD with agent pills, timeline, notifications overlay, command palette, and preferences UI.

## Data Flow

1. Agent calls `agentos-hook --event payload.json`
2. Hook parses and sends to daemon via Unix socket
3. Daemon's `AgentRegistry` maps source to the correct plugin
4. Plugin parses raw payload into `UniversalEvent`
5. Event is published on `EventBus` (broadcast channel)
6. `apply_event()` updates `SessionState`
7. Subscribers (desktop UI, persistence, notifications) receive the event
8. Events are persisted to SQLite asynchronously

## Project Structure

```
agentos/
├── apps/desktop/           # Tauri desktop shell (React + TypeScript)
├── core/
│   ├── daemon/             # agentosd binary
│   ├── ipc/                # Unix socket protocol
│   └── storage/            # SQLite persistence
├── daemon-core/            # Shared domain logic (Rust)
├── plugins/
│   ├── opencode/           # OpenCode event parser
│   ├── claude/             # Claude Code event parser
│   ├── codex/              # Codex CLI event parser
│   ├── antigravity/        # Antigravity event parser
│   ├── aider/              # Aider event parser
│   ├── gemini/             # Gemini CLI event parser
│   ├── cursor/             # Cursor event parser
│   ├── copilot/            # GitHub Copilot CLI event parser
│   └── deepseek/           # DeepSeek event parser
├── hooks/                  # agentos-hook CLI
└── docs/                   # Documentation
```
