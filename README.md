# Orbitos Island

> **The ambient cockpit for AI coding agents on Linux.**

Orbitos Island is a Linux-native desktop companion that lives in your system tray, watches your AI coding agents, and gives you instant control — without leaving your terminal flow.

Inspired by [Vibe Island](https://vibeisland.ai) and [Open Island](https://open-island.ai), but **open source, Linux-first, and 100% local**.

![CI](https://github.com/jomvick/Orbitos-island/actions/workflows/ci.yml/badge.svg)
![Rust](https://img.shields.io/badge/rust-1.75+-f06b1f)
![License](https://img.shields.io/badge/license-MIT-blue)
![Agents](https://img.shields.io/badge/agents-9-green)

---

## Features

* **Ambient UI** — Compact floating bar shows live agent activity. Hover to expand. No context switch.
* **Permission Overlay** — When an agent needs approval, a HUD pops up instantly. Approve or deny in one click.
* **Terminal Jump** — One click to focus the exact tmux/zellij/kitty/ghostty/wezterm pane where your agent is running.
* **Session Timeline** — Full event history per agent: tokens, duration, model, errors.
* **Command Palette** — Search sessions by agent, project, or command. Focus any session instantly.
* **Analytics** — Token usage, session duration, cost estimates, agent activity graphs.
* **Notifications** — Smart Linux-native notifications with priority (permission > error > complete > activity).
* **Tray Icon** — Quick glance at active sessions, start/stop services.

---

## Supported Agents

| Agent | Permission Support | Status |
|---|---|---|
| [Claude Code](https://docs.anthropic.com/en/docs/claude-code/overview) | ✅ | ✅ |
| [OpenCode](https://opencode.ai) | ✅ | ✅ |
| [Codex CLI](https://github.com/openai/codex-cli) | ❌ | ✅ |
| [Aider](https://aider.chat) | ❌ | ✅ |
| [Gemini CLI](https://github.com/google-gemini/gemini-cli) | ✅ | ✅ |
| [Antigravity](https://github.com/antigravity-ai/antigravity) | ❌ | ✅ |
| [Cursor](https://cursor.sh) | ✅ | ✅ |
| [GitHub Copilot](https://github.com/github-copilot/cli) | ❌ | ✅ |
| [DeepSeek](https://deepseek.com) | ✅ | ✅ |

Adding a new agent? [Write a plugin →](docs/plugins.md)

---

## Supported Terminals

| Terminal | Jump to Session |
|---|---|
| tmux | ✅ |
| zellij | ✅ |
| Ghostty | ✅ |
| WezTerm | ✅ |
| Kitty | ✅ |

---

## Architecture

Orbitos Island utilizes a high-performance modular architecture. The core service runs as a background system daemon (`agentosd`), communication is handled locally via Unix Sockets using a lightweight CLI tool (`agentos-hook`), and telemetry is rendered in a beautiful floating cockpit (`Orbitos Desktop`).

```
AI Agents (Claude, OpenCode, Codex, …)
    │
    ▼
agentos-hook  (ultra-light CLI hook, calls daemon)
    │
    ▼
┌──────────────────────────────────────┐
│           agentosd (Daemon)          │
│                                      │
│  ┌─────────┐  ┌──────────┐          │
│  │ Plugins │→ │ EventBus │          │
│  └─────────┘  └────┬─────┘          │
│                    ▼                │
│  ┌──────────┐  ┌──────────┐         │
│  │ Session  │  │  SQLite  │         │
│  │  State   │  │   Store  │         │
│  └──────────┘  └──────────┘         │
│                    │                │
│  ┌──────────┐  ┌──────────┐         │
│  │Notificat.│  │Terminal  │         │
│  │Dispatcher│  │Detector  │         │
│  └──────────┘  └──────────┘         │
└────────────────┬─────────────────────┘
                 │ Unix Socket IPC
                 ▼
┌──────────────────────────────────────┐
│     Orbitos Island (Tauri + React)   │
│                                      │
│  Floating Bar  │  Overlay HUD       │
│  Timeline      │  Command Palette   │
│  Analytics     │  Tray Icon         │
└──────────────────────────────────────┘
```

---

## Quick Start

### Prerequisites

* Rust 1.75+
* Node.js 18+
* npm

### Build & Run

1. **Clone the repository**
   ```bash
   git clone https://github.com/jomvick/Orbitos-island.git
   cd Orbitos-island
   ```

2. **Compile and test the workspace (Rust daemon)**
   ```bash
   cargo build --workspace
   cargo test --workspace
   ```

3. **Start the core background daemon**
   ```bash
   # Runs agentosd with debug logging
   cargo run --bin agentosd -- --verbose
   ```

4. **Launch the Orbitos Island desktop cockpit**
   ```bash
   cd apps/desktop
   npm install
   npm run tauri dev
   ```

Alternatively, you can start all services simultaneously with a single command:
```bash
./start.sh
```

---

### Auto-Detect & Install

To discover installed agent instances on your system and configure their hooks:
```bash
cargo run --bin agentosd -- --discover
```

This commands will scan for compatible agents (like Claude Code and OpenCode) and print structured details, letting the desktop UI trigger auto-configuration seamlessly.

---

### Manual Hook Installation

If you prefer to configure triggers manually:
```bash
./scripts/install-hooks.sh
agentos-hook --event '{"type":"session_start","agent":"opencode","session_id":"abc"}'
```

---

## Project Structure

```
agentos/
├── daemon-core/            # Shared domain logic (Rust)
│   ├── agents/             # Plugin traits + registries
│   ├── events/             # Local EventBus broadcaster
│   ├── state/              # Session state machines & transitions
│   ├── notifications/      # Priority-based notification manager
│   └── terminals/          # Terminal jumper modules (tmux, zellij)
│
├── core/
│   ├── daemon/             # agentosd main binary crate
│   ├── ipc/                # Unix socket protocols
│   └── storage/            # SQLite analytics & logs stores
│
├── plugins/                # Normalizers for agent output formats
│   ├── opencode/
│   ├── claude/
│   ├── codex/
│   ├── aider/
│   └── ... (and more)
│
├── hooks/                  # agentos-hook CLI source crate
├── apps/desktop/           # Tauri v2 Desktop app (React + TypeScript)
├── packages/               # Shared schema definitions
├── docs/                   # Developer guides & API docs
└── scripts/                # Launch & deployment helper scripts
```

---

## Tech Stack

| Layer | Technology |
|---|---|
| Core Backend | Rust, Tokio, Serde |
| Local IPC | Unix Sockets (length-prefixed JSON) |
| Local Storage | SQLite (rusqlite) |
| Desktop Shell | Tauri v2 |
| Frontend UI | React, TypeScript, TailwindCSS |
| State Engine | Zustand, TanStack Query |
| Sound / Alerts | notify-rust |

---

## Why Orbitos Island?

* **Linux native** — Built specifically to run on desktop environments like KDE, GNOME, and Wayland. No Electron bloat.
* **100% local** — Zero tracking, zero telemetry, zero accounts. Your credentials and code never leave your machine.
* **Agent-agnostic** — Easily add support for any existing or custom AI agent via lightweight plugins.
* **Ambient presence** — Stays completely out of your way until you need it, integrating as a floating desktop HUD.
* **Terminal-first** — Focuses the exact tmux/zellij terminal pane where the agent is waiting.
* **Open Source** — MIT licensed. Hack it, improve it, share it.

---

## Active Roadmap & Issues

We have organized our pending roadmap and recent core analysis findings into **active, link-tracked GitHub Issues**. Contributors are highly encouraged to click and inspect their full technical requirements below:

### Vague Roadmap Refinement:
- [x] Daemon IPC protocol, normalizer plugins, state machine, and persistence.
- [x] 9 AI agent integrations and tmux session jumping.
- [x] Translucent Tauri floating desktop bar, command palette, and overlay HUD.
- [ ] [Plan Review Overlay (Issue #6)](https://github.com/jomvick/Orbitos-island/issues/6) — Inline file diff visualization inside permission prompt overlays.
- [ ] [System Sound Alerts (Issue #7)](https://github.com/jomvick/Orbitos-island/issues/7) — Custom PipeWire sound playbacks on high-priority agent transitions.
- [ ] [Agent Auto-Discovery (Issue #8)](https://github.com/jomvick/Orbitos-island/issues/8) — Setup assistant detecting local agent installs and placing hooks automatically.
- [ ] [Timeline Filters & Pagination (Issue #9)](https://github.com/jomvick/Orbitos-island/issues/9) — Database-level query limits and dropdown filters for timeline feeds.
- [ ] [Vibe Island Feature Parity (Issue #10)](https://github.com/jomvick/Orbitos-island/issues/10) — SSH tunneling port forwards, daily token quotas, and smart action triggers.
- [ ] [Fluid HUD Orb Animations (Issue #11)](https://github.com/jomvick/Orbitos-island/issues/11) — Liquid shape-shifting SVG indicator representing agent cognitive state.

### Core Enhancements & Stability Issues:
- [ ] [Global System-wide Shortcuts (Issue #12)](https://github.com/jomvick/Orbitos-island/issues/12) — Tauri global keyboard shortcuts to toggle the cockpit view instantly.
- [ ] [Daemon Auto-Spawning & Watchdog (Issue #13)](https://github.com/jomvick/Orbitos-island/issues/13) — Automatically launch `agentosd` on app start and manage logging.
- [ ] [Native kitty & Ghostty Focus Jump (Issue #14)](https://github.com/jomvick/Orbitos-island/issues/14) — Focus non-multiplexed terminal windows utilizing native remote interfaces.
- [ ] [Wayland Layer-Shell Overlay Positioning (Issue #15)](https://github.com/jomvick/Orbitos-island/issues/15) — GTK layer-shell window bounds configurations for solid float on Wayland.
- [ ] [API Token Pricing and Budget Thresholds (Issue #16)](https://github.com/jomvick/Orbitos-island/issues/16) — Multi-model price trackers and hard notifications when budgets are reached.

---

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## License

MIT
