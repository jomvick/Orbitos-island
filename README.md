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
* **Terminal Jump** — One click to focus the exact tmux/zellij/kitty/ghostty/wezterm/Konsole pane where your agent is running, with automatic X11/Wayland fallback.
* **Session Timeline** — Full event history per agent: tokens, duration, model, errors.
* **Command Palette** — Search sessions by agent, project, or command. Focus any session instantly.
* **Analytics** — Token usage, session duration, cost estimates, agent activity graphs.
* **Audio Notifications** — System sound alerts for permission requests, task errors, and completions via PipeWire/PulseAudio/ALSA.
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
| tmux | ✅ (session/window/pane via IPC) |
| zellij | ✅ (session/tab/pane via IPC) |
| Kitty | ✅ (window focus via `kitty @`) |
| Ghostty | ✅ (pane focus via CLI) |
| WezTerm | ✅ (pane focus via `wezterm cli`) |
| Konsole | ✅ (via D-Bus) |

**Fallback chain:** Terminal-specific IPC → X11 (`xdotool` → `wmctrl`) → Wayland (`swaymsg` → `qdbus` → `ydotool`)

Jump detection uses process tree walking from the agent hook's parent PID through `/proc` to identify and store the exact terminal pane at session start — no re-detection on each jump.

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
 │  │Dispatcher│  │Jump+Det  │         │
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
│   └── terminals/          # Terminal detection + jumper (tmux, kitty, Ghostty, WezTerm, zellij, Konsole, X11/Wayland fallback)
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
| Sound / Alerts | notify-rust, pw-play / paplay / aplay |

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

Each item links to its GitHub Issue with full technical specs. Status reflects the **actual codebase state**, not aspirations.

### Completed:

- [x] Daemon IPC protocol — Serde-tagged discriminated unions (`IpcMessage`, `IpcCommand`, `IpcStatus`), Unix socket codec, normalizer plugins, session state machine, SQLite persistence.
- [x] 9 AI agent integrations — Claude, OpenCode, Codex, Aider, Gemini, Antigravity, Cursor, Copilot, DeepSeek. Each has a normalizer plugin in `plugins/`.
- [x] Floating bar + overlay HUD — Tauri v2 translucent window, FloatingBar (priority pill + hover panel + cockpit), Overlay (Permission/Question/Review tabs), Dashboard (sessions + analytics).
- [x] [System Sound Alerts (Issue #7)](https://github.com/jomvick/Orbitos-island/issues/7) — `pw-play` / `paplay` / `aplay` auto-detection, embedded WAV assets, Tauri `play_sound` command invoked on `permission_requested`, `session_failed`, `session_completed` events.
- [x] [Plan Review Overlay (Issue #6)](https://github.com/jomvick/Orbitos-island/issues/6) — `PlanDiffView` renders unified diffs with color-coded add/remove/hunk lines, per-file expand/collapse cards, plan proposal steps. Overlay has "Permission" and "Review Changes" tabs.
- [x] [Agent Auto-Discovery (Issue #8)](https://github.com/jomvick/Orbitos-island/issues/8) — `agentosd --discover` scans PATH for 9 agents, auto-installs hooks for Claude/OpenCode, creates shell wrappers in `~/.local/share/agentos/bin/`, injects PATH into shell configs. `DiscoveryWizard` UI component in Settings > Plugins invokes `discover_agents` IPC, shows live detection status per agent, one-click "Install Hooks" buttons.
- [x] [Native Terminal Focus Jump (Issue #14)](https://github.com/jomvick/Orbitos-island/issues/14) — Process tree walk from hook PPID via `/proc`, pane targeting for tmux/zellij/kitty/Ghostty/WezTerm/Konsole, X11/Wayland fallback dispatcher (`xdotool` → `wmctrl` → `swaymsg` → `ydotool`).
- [x] [Timeline Filters & Pagination (Issue #9)](https://github.com/jomvick/Orbitos-island/issues/9) — Backend SQL supports `OFFSET` + dynamic `WHERE` filters (agent, phase/event_kind). Frontend Timeline rewired to `invoke("get_timeline", { limit, offset, agent, phase })` with agent/phase dropdowns and "Load More" pagination button.
- [x] [Type IPC payload with discriminated union (Issue #19)](https://github.com/jomvick/Orbitos-island/issues/19) — `daemon_client.rs` emits `data_type` tag (`"session"` | `"event"`), TypeScript `DaemonEventData` discriminated union in shared-schema, all `as any` casts removed, `useDaemonConnection.ts` dispatches on `data_type` instead of duck-typing.
- [x] [Replace bare unwrap() with expect() (Issue #17)](https://github.com/jomvick/Orbitos-island/issues/17) — `lib.rs` Tauri entry points use `expect()` with descriptive messages. `clippy.toml` disallows `Option::unwrap` and `Result::unwrap`.
- [x] [Retry in agentos-hook sender (Issue #18)](https://github.com/jomvick/Orbitos-island/issues/18) — `sender.rs` `send_event()` retries once with 50ms delay before failing, preventing dropped events on transient daemon startup.
- [x] [Wayland Layer-Shell Overlay Positioning (Issue #15)](https://github.com/jomvick/Orbitos-island/issues/15) — Compositor detection (Sway/Hyprland/River/KDE/GNOME), per-compositor overlay rules via native CLI tools (`swaymsg`, `hyprctl`, `riverctl`, `kwriteconfig`), `set_ignore_cursor_events` wired to Tauri API, always-on-top without focus steal on Wayland.

### Planned:

- [ ] [Vibe Island Feature Parity (Issue #10)](https://github.com/jomvick/Orbitos-island/issues/10) — SSH tunneling for remote sessions, daily token quotas with hard limits, smart action triggers (auto-approve trusted commands, auto-reject dangerous ones).
- [ ] [Fluid HUD Orb Animations (Issue #11)](https://github.com/jomvick/Orbitos-island/issues/11) — Liquid shape-shifting SVG orb that morphs based on agent cognitive state (idle → thinking → waiting → done). Requires lottie/SVG animation engine and phase-to-shape mapping.
- [ ] [Global System-wide Shortcuts (Issue #12)](https://github.com/jomvick/Orbitos-island/issues/12) — **Needs `tauri-plugin-global-shortcut`** integration. Currently only local DOM `keydown` listeners (Escape, Alt+A, 1-9 for question answers) — these only work when the Tauri window has focus. Global shortcuts must work regardless of focus.
- [ ] [Daemon Auto-Spawning & Watchdog (Issue #13)](https://github.com/jomvick/Orbitos-island/issues/13) — Daemon has internal watchdogs (PID watcher, stale session orphaning). **Missing: Tauri `setup()` must detect if `agentosd` socket exists, spawn it as a child process if not, and restart on crash. Currently `daemon_client.rs` retries 10× then gives up — no spawn logic.**
- [ ] [API Token Pricing & Budget Thresholds (Issue #16)](https://github.com/jomvick/Orbitos-island/issues/16) — Per-model price tables (`input_cost_per_1k`, `output_cost_per_1k`), session cost accumulation, hard notification when daily/monthly budget exceeded, analytics charts.

### Technical Debt:

See remaining open issues at [github.com/jomvick/Orbitos-island/issues](https://github.com/jomvick/Orbitos-island/issues).

---

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## License

MIT
