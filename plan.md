# 🧠 Vision du Projet

Créer une application Linux-native qui agit comme :

> un cockpit système pour agents IA de développement.

L’application centralise :

* monitoring temps réel
* notifications intelligentes
* gestion multi-agents
* navigation rapide entre sessions terminal
* analytics d’usage
* orchestration de workflows IA

pour :

* OpenCode
* Antigravity
* Codex
* Claude Code
* Aider
* Gemini CLI
* futurs agents

---

# 🎯 Objectif du MVP

Résoudre immédiatement ces problèmes :

✅ agents silencieux
✅ notifications ratées
✅ perte de contexte terminal
✅ multi-agents chaotiques
✅ aucune vue globale des sessions
✅ difficulté à retrouver le bon agent

---

# 🏗️ Architecture Générale

```txt
AI Agents
    ↓
Hooks / Plugins
    ↓
Local Event Bus
    ↓
Core Daemon
    ↓
Desktop UI
    ↓
Notifications / Overlay / Analytics
```

---

# ⚙️ Stack Technique Finale

## 🔥 Backend Core

| Technologie  | Rôle                         |
| ------------ | ---------------------------- |
| Rust         | Performance + daemon système |
| Tokio        | Async runtime                |
| Serde        | JSON serialization           |
| SQLite       | Persistence locale           |
| Axum         | API locale                   |
| Unix Sockets | IPC                          |
| Notify-rust  | Notifications Linux          |

---

## 🎨 Frontend Desktop

| Technologie    | Rôle             |
| -------------- | ---------------- |
| Tauri v2       | Desktop shell    |
| React          | UI               |
| TypeScript     | Frontend logic   |
| TailwindCSS    | Styling          |
| Framer Motion  | Animations       |
| Zustand        | State management |
| TanStack Query | Server state     |

---

## 🧩 Intégrations Terminal

| Terminal | Priorité |
| -------- | -------- |
| tmux     | ⭐⭐⭐⭐⭐    |
| zellij   | ⭐⭐⭐⭐⭐    |
| Ghostty  | ⭐⭐⭐⭐     |
| WezTerm  | ⭐⭐⭐⭐     |
| Kitty    | ⭐⭐⭐      |
| Warp     | futur    |

---

# 🧬 Architecture des Modules

---

# 🔵 1. Core Daemon

Nom exemple :

```txt
agentosd
```

---

## Responsabilités

* recevoir événements
* maintenir état sessions
* stocker historique
* envoyer notifications
* exposer API locale
* gérer plugins agents
* analytics
* reconnect sessions

---

## Structure

```txt
core/
├── agents/
├── events/
├── sessions/
├── ipc/
├── notifications/
├── terminals/
├── storage/
├── analytics/
├── plugins/
└── api/
```

---

# 🟣 2. Plugin System

LE composant le plus important.

Chaque agent devient un plugin.

---

## Structure

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

## Rôle des plugins

Chaque plugin :

* lit hooks spécifiques
* parse événements
* normalise données
* map vers schema universel

---

# 🧩 Universal Event Schema

Très important.

---

## Exemple

```json
{
  "agent": "opencode",
  "event": "task_complete",
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

---

# 🟢 3. Hook CLI

Nom exemple :

```bash
agentos-hook
```

---

## Fonction

Les agents exécutent :

```bash
agentos-hook --event payload.json
```

Puis :

* parse
* valide
* envoie via socket Unix

---

## Objectif

Ultra léger :

* démarrage instantané
* zéro UI
* dépendances minimales

---

# 🌌 4. Desktop UI

Nom exemple :

```txt
AgentOS
```

---

# 🖥️ Layout Principal

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

# 🎨 Direction Design

Style :

* minimal
* cyberpunk soft
* terminal-native
* premium devtool

Inspirations :

* Raycast
* Linear
* Warp
* Arc
* Apple Dynamic Island
* Activity Monitor

---

# 🧠 Système de Notifications

Très important.

---

## Types

| Type          | Comportement          |
| ------------- | --------------------- |
| Permission    | popup urgent          |
| Task complete | toast discret         |
| Error         | notification rouge    |
| Long task     | progress notification |
| Idle          | silencieux            |

---

## Features

✅ sons custom
✅ priorité intelligente
✅ actions rapides
✅ ouvrir terminal directement
✅ focus session

---

# 🔥 Fonctionnalités MVP

---

# Phase 1 — Core MVP

## Backend

✅ daemon Rust
✅ Unix socket IPC
✅ SQLite
✅ schema événements
✅ système plugins

---

## Frontend

✅ tray icon
✅ liste sessions
✅ notifications
✅ dashboard minimal

---

## Intégrations

✅ OpenCode
✅ Antigravity
✅ tmux

---

# ⚡ Phase 2 — Productivité

## Sessions

✅ jump terminal
✅ focus pane
✅ session restore

---

## Analytics

✅ tokens
✅ durée tâches
✅ historique sessions

---

## UI

✅ timeline
✅ activity feed
✅ recherche sessions

---

# 🌌 Phase 3 — Premium UX

## Overlay

✅ floating HUD
✅ orb animation
✅ live pulse

---

## Intelligence

✅ task classification
✅ smart notifications
✅ session grouping

---

## Visualisation

✅ graphs agents
✅ repo map
✅ activity heatmap

---

# 🚀 Phase 4 — AI Operating Layer

Vision long terme.

---

## Multi-Agent Orchestration

Exemple :

```txt
Claude → architecture
Codex → tests
OpenCode → refactor
```

---

## Workflow Automation

```txt
When Claude finishes:
→ open diff
→ run tests
→ notify user
```

---

## AI Memory Layer

Historique global :

* projets
* agents
* tâches
* coûts
* décisions

---

# 📂 Structure Monorepo Recommandée

```txt
agentos/
├── apps/
│   ├── desktop/
│   └── settings/
│
├── core/
│   ├── daemon/
│   ├── ipc/
│   └── storage/
│
├── plugins/
│   ├── opencode/
│   ├── antigravity/
│   └── codex/
│
├── packages/
│   ├── shared-schema/
│   ├── ui/
│   └── utils/
│
└── docs/
```

---

# 🔐 Sécurité

Important dès le début.

---

## Règles

✅ localhost only
✅ sockets Unix privés
✅ aucun cloud obligatoire
✅ données locales uniquement
✅ permissions minimales

---

# 📈 Ce qui fera la différence

Le vrai avantage compétitif ne sera PAS :

* la UI
* les notifications
* le dashboard

mais :

> le système universel d’événements agents + navigation terminal intelligente.

C’est le cœur du produit.

---

# 🧠 Roadmap Réaliste

---

# Semaine 1

✅ architecture
✅ schema événements
✅ daemon minimal
✅ IPC socket

---

# Semaine 2

✅ plugin OpenCode
✅ plugin Antigravity
✅ hook CLI

---

# Semaine 3

✅ tray app Tauri
✅ notifications
✅ sessions live

---

# Semaine 4

✅ tmux integration
✅ jump session
✅ timeline

---

# Mois 2

✅ analytics
✅ overlays
✅ zellij support
✅ graphs

---

# 🎯 Vision Finale

Le projet doit devenir :

> l’interface système universelle des agents IA développeurs sur Linux.

Pas juste :

* une app tray
* une app notifications
* un clone de Vibe Island

mais :

> une couche OS moderne pour le développement assisté par agents IA.
