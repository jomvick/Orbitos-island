# Plugin System

Each AI agent becomes a plugin that normalizes its event format into the universal schema.

## How It Works

Plugins implement the `AgentPlugin` trait:

```rust
pub trait AgentPlugin: Send + Sync {
    fn name(&self) -> &'static str;
    fn agent_kind(&self) -> AgentKind;
    fn parse(&self, payload: &str) -> PluginResult;
}
```

- `name()` — unique plugin identifier (matches agent name)
- `agent_kind()` — maps to the `AgentKind` enum
- `parse()` — receives raw JSON, returns `Option<UniversalEvent>` or `PluginError`

## Registering a Plugin

1. Create a crate in `plugins/<name>/`
2. Add it to workspace `Cargo.toml`
3. Add it to `core/daemon/Cargo.toml`
4. Register in `core/daemon/src/plugin_loader.rs`

## AgentKind Enum

```rust
pub enum AgentKind {
    Opencode,
    Claude,
    Codex,
    Antigravity,
    Aider,
    Gemini,
    Cursor,
    Kimi,
    Qoder,
    Qwen,
    Factory,
    Codebuddy,
    Copilot,
    DeepSeek,
    Custom(String),
}
```

## UniversalEvent Schema

```json
{
  "id": "uuid",
  "agent": "opencode",
  "event": "session_started",
  "session_id": "abc123",
  "cwd": "/projects/app",
  "branch": "feature/auth",
  "model": "claude-sonnet-4",
  "tokens_input": 12000,
  "tokens_output": 8000,
  "duration_ms": 120000,
  "terminal": "ghostty",
  "pane": "2",
  "timestamp": "2026-05-14T12:00:00Z"
}
```

## Event Types

| EventKind              | Description              |
|------------------------|--------------------------|
| SessionStarted         | Agent begins a task      |
| ActivityUpdated        | Progress update          |
| PermissionRequested    | Agent needs permission   |
| QuestionAsked          | Agent has a question     |
| SessionCompleted       | Task completed           |
| SessionFailed          | Task errored             |
| SessionPaused          | Task paused              |
| Heartbeat              | Keepalive signal         |
| TokenUsage             | Token consumption update |
| JumpTargetUpdated      | Terminal pane targeting  |
| ActionableStateResolved| Permission/question resolved |
