# Octopod

[![crate](https://img.shields.io/crates/v/octopod.svg)](https://crates.io/crates/octopod)
[![build](https://img.shields.io/github/actions/workflow/status/ajsongpilot/octopod/ci.yml?branch=master)](https://github.com/ajsongpilot/octopod/actions)
[![license](https://img.shields.io/crates/l/octopod)](LICENSE)

**Many-Armed Company Orchestration** — Run your own AI-powered software company where you call the shots.

Octopod lets you act as CEO of a simulated company where AI agents in different departments collaborate to build software. Think Civilization meets software development — you set the strategy, your AI team executes.

## Command Reference

| Command | Description |
|---------|-------------|
| `octopod onboard` | First-time setup with AI-powered project discovery |
| `octopod init` | Initialize a new project with database |
| `octopod` | Open CEO Dashboard (default) |
| `octopod spawn all` | Open all department TUIs + start agent daemons |
| `octopod spawn <dept>` | Open specific department TUI + agent daemon |
| `octopod stop all` | Stop all department processes |

### Agent Commands

| Command | Description |
|---------|-------------|
| `octopod agent loop <dept>` | Run agent daemon continuously (always-on) |
| `octopod agent run <dept>` | Run agent once |
| `octopod agent schedule <dept> <secs>` | Schedule periodic agent runs |
| `octopod agent list` | List agent schedules |
| `octopod agent enable <dept>` | Enable schedule |
| `octopod agent disable <dept>` | Disable schedule |

### Decision & Initiative Commands

| Command | Description |
|---------|-------------|
| `octopod decide propose` | Create initiative with severity |
| `octopod decide submit` | Submit initiative for review |
| `octopod decide start` | Start an initiative |
| `octopod decide complete` | Complete an initiative |
| `octopod decide decide` | Log a decision |

### Task Commands

| Command | Description |
|---------|-------------|
| `octopod task new <dept> <title>` | Create task |
| `octopod task list` | List tasks |

### Database Commands

| Command | Description |
|---------|-------------|
| `octopod db init` | Initialize database |
| `octopod db backup` | Create backup |
| `octopod db list` | List backups |

## Quick Start

```bash
# Install
cargo install octopod

# First time setup
mkdir my-company && cd my-company
octopod onboard
octopod init

# Start everything (TUIs + agent daemons)
octopod spawn all

# Open CEO Dashboard (separate terminal)
octopod
```

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                      CEO Dashboard                       │
│   (Decision Queue, Initiative Planning, Activity Feed)    │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│                      Message Bus                         │
│   (DecisionProposal, InitiativeUpdate, TaskAssignment)    │
└─────────────────────────────────────────────────────────┘
                           │
      ┌──────────┬──────────┼──────────┬──────────┐
      ▼          ▼          ▼          ▼          ▼
┌─────────┐┌─────────┐┌─────────┐┌─────────┐┌─────────┐
│ Product ││  Eng    ││   QA    ││  DevOps ││Marketing│
│  Agent  ││  Agent  ││  Agent  ││  Agent  ││  Agent  │
└─────────┘└─────────┘└─────────┘└─────────┘└─────────┘
      │          │          │          │          │
      └──────────┴──────────┴──────────┴──────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│                    SQLite Database                        │
│   (Tasks, Decisions, Initiatives, Meetings, Messages)     │
└─────────────────────────────────────────────────────────┘
```

## Features

### Initiative-Driven Workflow
- **Propose initiatives** with severity levels (LOW/MED/HIGH)
- **Automatic triage** — LOW/MED auto-proceed, HIGH requires CEO approval
- **Status tracking** — Draft -> Proposed -> Approved -> Active -> Completed
- **Markdown files** — Initiatives and decisions are stored as markdown files in `.octopod/initiatives/` and `.octopod/decisions/`

### Ironclaw Integration
Octopod uses [Ironclaw](https://github.com/anomalyco/ironclaw) with [opencode](https://github.com/opencode-ai/opencode) as the underlying AI agent. Press `a` in the Planning tab to open an interactive tmux chat with Ironclaw about an initiative. Ironclaw can suggest improvements and, upon your approval, apply changes to the markdown file.

### Autonomous AI Agents
- Each department runs its own **always-on agent daemon** powered by Ironclaw/opencode
- Agents **pick up unassigned tasks** automatically
- Agents **collaborate via message bus**

### CEO Dashboard
- **Decision Queue** — see all HIGH-severity decisions awaiting your approval
- **Severity indicators** — HIGH / MED / LOW
- **Initiative Planning** — track all company initiatives

### Department TUIs
Each department has its own terminal interface with:
- **Board view** — Kanban-style task board
- **List view** — filterable task list
- **Activity feed** — real-time department activity
- **Chat** — inter-department communication

### Activity Log
- All agent actions logged to database
- CEO can view all activity across departments
- Timestamps, actors, actions tracked

## Severity System

Agents interpret severity themselves — no keyword matching:

| Severity | Behavior |
|----------|----------|
| 🔴 HIGH | CEO approval required via decision queue |
| 🟡 MEDIUM | Auto-proceed, log only |
| ⚪ LOW | Auto-proceed, log only |

## TUI Shortcuts

### CEO Dashboard (`octopod`)
Press `?` for help overlay.

| Key | Action |
|-----|--------|
| `Tab` / `1-4` | Switch tabs |
| `↑` / `↓` | Navigate |
| `s` | Spawn department |
| `k` | Kill department |
| `a` | Approve decision |
| `x` | Reject decision |
| `p` | Create roadmap |
| `i` | Create initiative |
| `d` | Draft with Ironclaw (one-shot improvement) |
| `a` | Ask Ironclaw (interactive chat) |
| `e` | Edit initiative/decision markdown |
| `v` | Cycle view filters (All/Active/Done) |
| `w` | Show work-in-progress only |
| `r` | Refresh |
| `?` | Show help |
| `q` | Quit |

### Department TUIs (`octopod spawn <dept>`)
Press `?` for help overlay.

| Key | Action |
|-----|--------|
| `Tab` | Cycle views (Board/List/Activity/Chat) |
| `←` / `→` | Navigate columns (Board view) |
| `↑` / `↓` | Navigate items |
| `Enter` | View task detail |
| `n` | Create new task |
| `x` | Delete task |
| `/` | Search (List view) |
| `f` | Cycle filters |
| `?` | Show help |
| `q` | Quit |

## Configuration

### Global Config (`~/.config/octopod/config.toml`)
```toml
[openrouter]
api_key = "sk-..."
model = "moonshotai/kimi-k2.5"

[platform]
type = "tmux"  # or "omarchy", "headless"
```

### Project Config (`.octopod/config.toml`)
```toml
[company]
name = "Acme Corp"
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

```bash
cargo test
cargo fmt --check
cargo clippy -- -D warnings
```

## License

Dual-licensed under **MIT** or **Apache-2.0** at your option. See [LICENSE](LICENSE) for details.
