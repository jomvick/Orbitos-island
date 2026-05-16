# Project Analysis: Orbitos Island

## Overview
Orbitos Island is a Linux-native desktop companion designed to monitor and control AI coding agents. It provides an ambient UI inspired by "Dynamic Island" concepts, allowing developers to keep track of agent activity (tokens, duration, state) without leaving their terminal flow.

## Architecture Breakdown

### 1. Backend Core (`agentosd`)
- **Language**: Rust
- **Key Components**:
    - **EventBus**: Broadcasts events across the system.
    - **Session State Machine**: Manages the lifecycle of agent sessions (Running, Waiting, Completed, Failed).
    - **Persistence**: SQLite (via `rusqlite`) for session history and analytics.
    - **IPC**: Unix Sockets for communication between the daemon, hooks, and desktop app.
    - **Notification Dispatcher**: Linux-native notifications with priority levels.
    - **Terminal Detector**: Handles "Jump to Session" functionality for various terminal emulators.

### 2. Plugin System
- **Location**: `plugins/`
- **Function**: Normalizes agent-specific outputs into a `UniversalEvent` schema.
- **Supported Agents**:
    - Claude Code, OpenCode, Codex CLI, Aider, Gemini CLI, Antigravity, Cursor, GitHub Copilot, DeepSeek.

### 3. Hooks (`agentos-hook`)
- **Function**: A lightweight CLI tool that agents or wrappers call to report events to the daemon.

### 4. Desktop UI (`apps/desktop`)
- **Tech Stack**: Tauri v2, React, TypeScript, TailwindCSS, Zustand, Framer Motion.
- **Features**:
    - **Floating Bar**: A compact, interactive bar showing active agents and status.
    - **Dashboard**: Expanded view with active sessions, timeline, and analytics.
    - **Activity Orb**: A visual indicator of overall agent activity and urgency.
    - **Overlay / HUD**: For permission requests and questions.
    - **Plan/Diff View**: Visualizes proposed plans and code diffs from agents.
    - **Command Palette**: Quick search and actions for sessions.

## Current Project Status

### Tests
- **Backend**: Extensive test suite (85+ tests) covering state transitions, IPC, storage, and plugins. All tests are passing.
- **Frontend**: Standard React/TypeScript setup (tests not explicitly run but build config exists).

### Implementation Progress
- The core infrastructure (Daemon, IPC, Storage) is robust and feature-complete for MVP.
- UI components for the "Island" experience are well-implemented with advanced animations.
- Multi-agent support is already in place via the plugin system.

### Roadmap & Next Steps
Based on the `README.md` and `plan.md`:
- **Plan Review overlay**: Diff view exists but might need more integration into the permission workflow.
- **Sound alerts**: High-priority events should trigger audio cues.
- **Auto-detection**: Automating the discovery and configuration of agent hooks.
- **Orb Mode**: Further enhancements to activity visualization.

## Technical Observations
- The project follows a clean, modular architecture.
- Use of Unix sockets ensures low-latency communication on Linux.
- The UI is highly polished, using Framer Motion for "fluid" transitions that match the "Island" aesthetic.
- SQLite integration provides a solid foundation for long-term analytics and history.
