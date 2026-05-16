# Contributing to Orbitos Island

We welcome contributions! This project is open source (MIT) and built by and for the developer community.

## Quick Links

- [Architecture](docs/architecture.md) — Understand the system design
- [Plugin System](docs/plugins.md) — How agents plugins work
- [Terminal Integration](docs/terminals.md) — Adding terminal support
- [Development Guide](docs/development.md) — Build & run locally

## Development Setup

```bash
# Clone
git clone https://github.com/jomvick/Orbitos-island.git
cd Orbitos-island

# Build all crates
cargo build --workspace

# Run all tests
cargo test --workspace

# Start daemon
cargo run --bin agentosd -- --verbose
```

## What You Can Contribute

### Agent Plugins

We currently support 9 agents. Adding a new one is straightforward:

1. Create `plugins/<name>/` with a crate implementing `AgentPlugin` trait
2. Register in `core/daemon/src/plugin_loader.rs`
3. Add to workspace and daemon `Cargo.toml`
4. Write tests (see existing plugins for patterns)

See [Plugin System docs](docs/plugins.md) for details.

### Terminal Integrations

We support 5 terminals for "jump to session". Adding more:

1. Create `daemon-core/src/terminals/<name>.rs`
2. Implement detection + focus functions
3. Add to `terminals/mod.rs` and `detector.rs`

See [Terminal docs](docs/terminals.md) for details.

### Desktop UI

The frontend is React + TypeScript in `apps/desktop/`. Key areas:

- **FloatingBar** — Compact agent activity display
- **Overlay** — Permission/question HUD
- **CommandPalette** — Session search and quick actions
- **Timeline** — Event history
- **Tray** — System tray integration

### Core Backend

Rust crates in `core/` and `daemon-core/`:

- Event system, state machine, IPC protocol
- SQLite persistence and analytics
- Notification dispatcher

## Code Style

- Rust: follow existing patterns, `cargo clippy` must pass
- TypeScript/React: use existing component conventions
- Tests: every plugin needs at least 4 tests (start, complete, permission/activity, invalid)
- Keep it simple. No speculative abstractions.

## Pull Request Process

1. Ensure `cargo test --workspace` passes
2. Ensure `cargo clippy --workspace --all-targets` passes
3. Update docs if you add a plugin, terminal, or feature
4. PR description should explain what and why

## Questions?

Open an issue on GitHub.
