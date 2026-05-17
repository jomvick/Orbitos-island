# 🧠 Project Vision

Create a Linux-native application that acts as:

> a system cockpit for AI development agents.

The application centralizes:

* Real-time monitoring
* Smart notifications
* Multi-agent management
* Fast navigation between terminal sessions
* Usage analytics
* AI workflow orchestration

For:

* OpenCode
* Antigravity
* Codex
* Claude Code
* Aider
* Gemini CLI
* Future agents

---

# 🎯 MVP Objective

Immediately solve these common pain points:

✅ Silent agents (processing silently in background)
✅ Missed notifications/approval prompts
✅ Lost terminal context (forgetting which terminal runs what)
✅ Chaotic multi-agent coordination
✅ No global view of active agent sessions
✅ Difficulty jumping back to the active agent's pane

---

# 🏗️ General Architecture

```txt
AI Agents
    ↓
Hooks / Plugins
    ↓
Local Event Bus
    ↓
Core Daemon (agentosd)
    ↓
Desktop UI (Orbitos Island)
    ↓
Notifications / Overlay HUD / Analytics
```

---

# ⚙️ Final Technical Stack

## 🔥 Backend Core

| Technology | Role |
| --- | --- |
| Rust | Performance + system daemon (`agentosd`) |
| Tokio | Async runtime |
| Serde | JSON serialization |
| SQLite | Local persistence |
| Unix Sockets | IPC communication |
| Notify-rust | Native Linux notifications |

---

## 🎨 Frontend Desktop

| Technology | Role |
| --- | --- |
| Tauri v2 | Desktop shell |
| React | UI framework |
| TypeScript | Frontend logic |
| TailwindCSS | Styling |
| Framer Motion | Fluid micro-animations |
| Zustand | State management |
| TanStack Query | Local server state caching |

---

## 🧩 Terminal Integrations

| Terminal | Priority |
| --- | --- |
| tmux | ⭐⭐⭐⭐⭐ |
| zellij | ⭐⭐⭐⭐⭐ |
| Ghostty | ⭐⭐⭐⭐ |
| WezTerm | ⭐⭐⭐⭐ |
| Kitty | ⭐⭐⭐ |
| Warp | Future |

---

# 🧬 Module Architecture

---

# 🔵 1. Core Daemon

Technical binary name:

```txt
agentosd
```

---

## Responsibilities

* Receive agent hooks events
* Maintain active session states
* Save history and token telemetry
* Dispatch prioritized notifications
* Expose a local Unix socket API
* Load agent plugins
* Track usage analytics
* Support session reconnection

---

## Directory Structure

```txt
core/
├── agents/          # Agent abstractions
├── events/          # EventBus (broadcast channel)
├── sessions/        # Session state machines
├── ipc/             # Unix socket IPC protocol
├── notifications/   # System tray & DBus notifier
├── terminals/       # tmux/zellij focused terminal detectors
├── storage/         # SQLite DB migrations and stores
├── analytics/       # Telemetry parsing & cost models
├── plugins/         # Dynamic agent parser registries
└── api/             # Local API handlers
```

---

# 🟣 2. Plugin System

The core framework component. Each AI agent is treated as an isolated plugin.

---

## Plugin Directory Structure

```txt
plugins/
├── codex/
├── claude/
├── opencode/
├── antigravity/
├── aider/
└── gemini/
```

---

## Role of Plugins

Each agent-specific plugin:
* Hooks into agent CLI/API lifecycles
* Parses output streams/event logs
* Normalizes telemetry datasets
* Maps agent-specific events into the **Universal Event Schema**

---

# 🧩 Universal Event Schema

A standardized model enabling plug-and-play agent integrations.

---

## Schema Payload Example

```json
{
  "agent": "opencode",
  "event": "task_complete",
  "session_id": "abc123",
  "cwd": "/projects/app",
  "branch": "feature/auth",
  "model": "claude-3-5-sonnet",
  "tokens_input": 12000,
  "tokens_output": 8000,
  "duration_ms": 120000,
  "terminal": "ghostty",
  "pane": "2",
  "timestamp": "2026-05-14T12:00:00Z"
}
```

---

# 🟢 3. Hook CLI

Technical binary name:

```bash
agentos-hook
```

---

## Core Function

AI coding agents execute hooks:

```bash
agentos-hook --event payload.json
```

The hook:
1. Parses and validates the event
2. Sends the structured JSON via the local Unix socket to `agentosd`

---

## Design Objective

Ultra-lightweight:
* Zero overhead (instant startup)
* Headless (no GUI dependencies)
* Native compilation with minimal external dependencies

---

# 🌌 4. Desktop UI

Product Name:

```txt
Orbitos Island
```

---

# 🖥️ Main Layout Mockup

```txt
┌──────────────────────────┐
│ Active Agents            │
├──────────────────────────┤
│ Claude  ████ 32k tokens  │
│ Codex   ██   Running     │
│ OpenCode Waiting         │
└──────────────────────────┘

┌──────────────────────────┐
│ Timeline                 │
├──────────────────────────┤
│ Claude completed task    │
│ Permission requested     │
│ Codex generated tests    │
└──────────────────────────┘
```

---

# 🎨 Design Direction

Visual aesthetic:
* Minimalist
* Soft cyberpunk accents
* Terminal-native design language
* Premium tool feel

Design inspirations:
* Raycast
* Linear
* Warp
* Arc Browser
* Apple Dynamic Island
* macOS Activity Monitor

---

# 🧠 Notification System

Prioritized alert system to prevent communication fatigue.

---

## Event Classification

| Event Type | Behavior |
| --- | --- |
| Permission | Urgent HUD popup overlay |
| Task complete | Subtle tray toast |
| Error | Immediate red critical banner |
| Long task | Ambient progress indicator |
| Idle | Silent background update |

---

## Key Features

✅ Custom alert sounds per agent status
✅ Intelligent priority throttling (permission > error > complete)
✅ Quick actionable buttons inside desktop notifications
✅ Direct terminal jumping on click
✅ Dynamic session focus tracking

---

# 🔥 MVP Features

---

# Phase 1 — Core MVP

## Backend
✅ Stable Rust core daemon
✅ Unix Socket IPC pipeline
✅ Local SQLite schema storage
✅ Standardized event serialization
✅ Extensible agent plugin framework

## Frontend
✅ Ambient tray icon status widget
✅ Dynamic session list panel
✅ Desktop notification alerts
✅ Sleek, minimal cockpit dashboard

## Integrations
✅ OpenCode support
✅ Antigravity agent integration
✅ Native tmux terminal jump detector

---

# ⚡ Phase 2 — Productivity

## Sessions
✅ Instant terminal jumping
✅ Direct pane/window focusing
✅ Interactive session restoration

## Analytics
✅ Live token ingestion tracking
✅ Task execution duration monitoring
✅ Historical session search logs

## UI
✅ Session activity timeline
✅ Live activity feeds
✅ Quick session command palette search

---

# 🌌 Phase 3 — Premium UX

## Overlay
✅ Ambient floating HUD
✅ Custom status orb animation
✅ Live heartbeat pulsing glow

## Smart Features
✅ Automated task classification
✅ Priority notification throttling
✅ Logical workspace session grouping

## Visualizations
✅ Cost & usage analysis graphs
✅ Workspace file repository mapping
✅ Contributor activity heatmaps

---

# 🚀 Phase 4 — AI Operating Layer

Long term ecosystem vision.

## Multi-Agent Orchestration

Example:

```txt
Claude (Architect) → Codex (Write Tests) → OpenCode (Refactor Crate)
```

## Workflow Automation

```txt
When Claude finishes:
→ open local code diff
→ launch automated cargo tests
→ alert user on success
```

## AI Memory Layer

Unified workspace execution registry tracking:
* Multi-repo project footprints
* Cumulative agent usage
* Executed tasks and logs
* Operational token costs
* Technical design decisions

---

# 📂 Recommended Monorepo Structure

```txt
agentos/
├── apps/
│   ├── desktop/            # Tauri app (Orbitos Island Cockpit)
│   └── settings/           # Configuration GUI
│
├── core/
│   ├── daemon/             # agentosd executable
│   ├── ipc/                # Unix socket IPC channel
│   └── storage/            # SQLite store crate
│
├── plugins/
│   ├── opencode/
│   ├── antigravity/
│   └── codex/
│
├── packages/
│   ├── shared-schema/      # Typescript IPC models
│   ├── ui/                 # React component library
│   └── utils/              # Helper utilities
│
└── docs/                   # Documentation & RFCs
```

---

# 🔐 Security

Engineered locally-first to keep credentials and workspace intellectual property completely private.

## Rules
✅ Localhost loopback operations only
✅ Private Unix socket permissions
✅ Zero mandatory cloud integration
✅ Local sqlite storage (fully inspectable)
✅ Minimal runtime OS capabilities required

---

# 📈 Competitive Differentiator

The ultimate value proposition is NOT just notification alerts or dashboard widgets, but:

> a standardized event schema combined with sub-second terminal session navigation.

This forms the true core of Orbitos Island.

---

# 🧠 Realistic Roadmap

---

# Week 1
✅ Monorepo structural setup
✅ Universal Event Schema JSON specifications
✅ Rust Core Daemon scaffolding
✅ Local socket IPC pipeline

---

# Week 2
✅ OpenCode normalizer plugin
✅ Antigravity agent plugin
✅ Lightweight `agentos-hook` CLI tool

---

# Week 3
✅ Tauri tray companion shell
✅ DBus desktop notifications
✅ Real-time session store syncing

---

# Week 4
✅ tmux pane focusing terminal integrations
✅ Sub-second terminal jumping
✅ Live session timeline feeds

---

# Month 2
✅ Telemetry graphs & token cost calculators
✅ Translucent overlay HUD components
✅ Zellij session jumping support
✅ Workspace activity analytics graphs

---

# 🎯 Final Vision

The project aims to become:

> the universal operating-system interface for AI development agents on Linux.

It is designed to be much more than a notification tray, but a modern desktop orchestration layer for agent-assisted software engineering.
