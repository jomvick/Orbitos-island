# Terminal Integration

AgentOS can detect, list, and focus terminal panes to enable "jump to session" functionality.

## Supported Terminals

| Terminal | Detection Method          | Focus Command                          |
|----------|---------------------------|----------------------------------------|
| tmux     | `tmux list-sessions`      | `tmux select-pane -t <id>`             |
| zellij   | `zellij list-sessions`    | `zellij action focus-pane -p <id>`     |
| Ghostty  | `ghostty +list-splits`    | `ghostty +focus-split <id>`            |
| WezTerm  | `wezterm cli list`        | `wezterm cli activate-pane --pane-id`  |
| Kitty    | `kitty @ ls`              | `kitty @ focus-window --match id:<id>` |

## How It Works

1. An agent emits an event with a `terminal` field
2. `detector::resolve_jump_target()` identifies the correct pane by PID or CWD
3. `detector::focus_terminal()` executes the terminal-specific focus command
4. The desktop UI calls `jump_to_session` to activate the pane

## Adding a New Terminal

1. Create `daemon-core/src/terminals/<name>.rs`
2. Implement `is_available()`, `is_in_<name>()`, `list_panes()`, `focus_pane()`
3. Add module to `terminals/mod.rs`
4. Add detection + focus handling to `detector.rs`
