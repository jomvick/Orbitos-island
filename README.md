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

- **Ambient UI** — Compact floating bar shows live agent activity. Hover to expand. No context switch.
- **Permission Overlay** — When an agent needs approval, a HUD pops up instantly. Approve or deny in one click.
- **Terminal Jump** — One click to focus the exact tmux/zellij/kitty/ghostty/wezterm pane where your agent is running.
- **Session Timeline** — Full event history per agent: tokens, duration, model, errors.
- **Command Palette** — Search sessions by agent, project, or command. Focus any session instantly.
- **Analytics** — Token usage, session duration, cost estimates, agent activity graphs.
- **Notifications** — Smart Linux-native notifications with priority (permission > error > complete > activity).
- **Tray Icon** — Quick glance at active sessions, start/stop daemon.

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

## Supported Terminals

| Terminal | Jump to Session |
|---|---|
| tmux | ✅ |
| zellij | ✅ |
| Ghostty | ✅ |
| WezTerm | ✅ |
| Kitty | ✅ |

## Architecture

```
AI Agents (Claude, OpenCode, Codex, …)
    │
    ▼
agentos-hook  (ultra-light CLI, calls daemon)
    │
    ▼
┌──────────────────────────────────────┐
│           agentosd (daemon)          │
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
│     Orbitos Desktop (Tauri + React)  │
│                                      │
│  Floating Bar  │  Overlay HUD       │
│  Timeline      │  Command Palette   │
│  Analytics     │  Tray Icon         │
└──────────────────────────────────────┘
```

## Quick Start

### Prerequisites

- Rust 1.75+
- Node.js 18+
- pnpm or npm

### Build & Run

```bash
# Clone the repo
git clone https://github.com/jomvick/Orbitos-island.git
cd Orbitos-island

# Build all Rust crates (12 crates, 85+ tests)
cargo build --workspace
cargo test --workspace

# Start the daemon
cargo run --bin agentosd -- --verbose

# In another terminal, start the desktop app
cd apps/desktop
npm install
npm run tauri dev
```

### Auto-Detect & Install

```bash
# Auto-discover installed agents and install hooks
cargo run --bin agentosd -- --discover

# Or via IPC command (from desktop app)
# The desktop UI can call discover_agents which returns structured results
```

Example output:
```json
{
  "total_agents": 9,
  "installed_count": 5,
  "hooks_installed_count": 2,
  "agents": [
    {"name": "claude", "installed": true, "hooks_installed": true},
    {"name": "opencode", "installed": true, "hooks_installed": true}
  ]
}
```

Supports auto-config for **Claude Code** and **OpenCode** (native hooks). Other agents report as detected for manual setup.

### Manual Hook Installation

```bash
./scripts/install-hooks.sh
agentos-hook --event '{"type":"session_start","agent":"opencode","session_id":"abc"}'
```

## Project Structure

```
agentos/
├── daemon-core/            # Shared domain logic (Rust)
│   ├── agents/             # Plugin trait + registry
│   ├── events/             # EventBus (broadcast)
│   ├── state/              # Session state machine + event types
│   ├── notifications/      # Priority-based notification dispatcher
│   └── terminals/          # tmux, zellij, ghostty, wezterm, kitty
│
├── core/
│   ├── daemon/             # agentosd binary (IPC server, plugin loader)
│   ├── ipc/                # Unix socket protocol
│   └── storage/            # SQLite persistence + analytics
│
├── plugins/                # Agent-specific event parsers
│   ├── opencode/           # OpenCode
│   ├── claude/             # Claude Code
│   ├── codex/              # Codex CLI
│   ├── antigravity/        # Antigravity
│   ├── aider/              # Aider
│   ├── gemini/             # Gemini CLI
│   ├── cursor/             # Cursor
│   ├── copilot/            # GitHub Copilot CLI
│   └── deepseek/           # DeepSeek
│
├── hooks/                  # agentos-hook CLI (calls daemon)
├── apps/desktop/           # Tauri v2 desktop app (React + TypeScript)
├── packages/               # Shared TypeScript packages (schema, UI)
├── docs/                   # Documentation
└── scripts/                # Install/uninstall helpers
```

## Tech Stack

| Layer | Technology |
|---|---|
| Core | Rust, Tokio, Serde |
| IPC | Unix Sockets (length-prefixed JSON) |
| Storage | SQLite (rusqlite) |
| Desktop Shell | Tauri v2 |
| Frontend | React, TypeScript, TailwindCSS |
| State | Zustand, TanStack Query |
| Notifications | notify-rust |
| Mutli-agent | Plugin system (`AgentPlugin` trait) |

## Why Orbitos Island?

- **Linux native** — No Electron bloat, no macOS gatekeeping. Built for KDE/GNOME/Wayland.
- **100% local** — Zero cloud, zero telemetry, zero accounts. Your agents, your machine.
- **Agent-agnostic** — Any AI coding agent can be supported via a lightweight plugin.
- **Ambient, not annoying** — Floating UI that stays out of your way until you need it.
- **Terminal-first** — Jump to the exact tmux/zellij pane where your agent is running.
- **Open source** — MIT license. Fork it, hack it, share it.

## Status

- **85+ tests** across 12 Rust crates, all passing
- **9 agent plugins** implemented and tested
- **5 terminal integrations** for session jump
- **Daemon**: EventBus, state machine, SQLite persistence, notification dispatcher
- **Desktop**: Tauri v2 shell, floating bar, timeline, overlay, command palette, tray icon
- **Build**: CI pipeline (clippy + build + test)

## Roadmap

- [x] Daemon with IPC, plugins, session state, persistence
- [x] 9 agent plugins + 5 terminal integrations
- [x] Tauri desktop with floating bar, overlay, timeline, palette
- [ ] Plan Review overlay (diff in permission request)
- [ ] Sound alerts for high-priority events
- [ ] Auto-detect and configure agent hooks
- [ ] Event history timeline (pagination, filtering)
- [ ] Vibe Island feature parity (remote SSH, quotas, smart actions)
- [ ] Orb mode — animated activity visualization

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT
