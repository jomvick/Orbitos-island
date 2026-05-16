# Development

## Prerequisites

- Rust 1.75+
- Node.js 18+
- pnpm or npm

## Building

```bash
# Build all Rust crates
cargo build

# Run all tests
cargo test

# Build desktop app
cd apps/desktop
npm install
npm run tauri build
```

## Running

```bash
# Start the daemon
cargo run --bin agentosd -- --verbose

# Start desktop dev mode
cd apps/desktop
npm run tauri dev
```

## Project Structure

```
agentos/
├── daemon-core/       # Shared domain types and logic
├── core/
│   ├── daemon/        # agentosd binary
│   ├── ipc/           # Unix socket IPC protocol
│   └── storage/       # SQLite persistence
├── plugins/           # Agent-specific event parsers
├── hooks/             # agentos-hook CLI
├── apps/desktop/      # Tauri desktop application
├── plugins/           # 9 agent-specific event parsers
└── packages/          # Shared TypeScript packages
```

## IPC Protocol

Unix socket with length-prefixed JSON messages. Max message size: 1MB.

Messages types:
- `Event { source, payload }` — incoming event from agent
- `Command { id, command }` — RPC-style request
- `Subscribe` — real-time event subscription
- `SubscriptionEvent { channel, event }` — pushed to subscribers
- `Response { id, data, error }` — RPC response
