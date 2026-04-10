# Octopod Architecture & Implementation Plan

**Many-Armed Company Orchestration** - AI-Powered Software Development Teams

## Executive Summary

Octopod is an opinionated multi-workspace TUI application that creates AI-powered software company simulations. The user acts as CEO, spawning and managing AI agents across classic departments (Product, Engineering, QA, Design, DevOps, Marketing, Sales) that collaborate to build and operate software products. Like an octopus with many arms working in coordination, Octopod enables multiple AI agents to work simultaneously across different workstreams.

### Core Philosophy

- **Don't build what exists**: Use ironclaw, opencode, and tdl as foundational tools
- **CEO as orchestrator**: User monitors, guides, and unblocks agents across departments
- **Git-native**: Company knowledge lives in Git (cortex), issues tracked via GitHub/GitLab
- **Linux-native**: Deep integration with Hyprland workspaces, tmux, waybar
- **Stateful persistence**: Agents remember context across sessions
- **Secure by default**: Ironclaw is the default backend, but swappable
- **Opinionated but swappable**: All tool choices behind abstractions - we can change our minds later!
- **Database is not optional**: SQLite for structured data (decisions, conversations, tasks) - NOT overkill, essential for queryability
- **Streamlined but comprehensive**: Build what you need, not everything imaginable

### The "Swappable Opinions" Approach

Octopod makes strong default choices but **every opinion is behind an abstraction**:

| Opinion | Default | Swappable To |
|---------|---------|--------------|
| **Coordinator Backend** | Ironclaw | OpenClaw (Phase 6) |
| **Coding Agent** | Opencode | Aider, Claude Code, Continue.dev |
| **Platform** | Linux (Hyprland) | Generic TUI (macOS, other Linux) |
| **LLM Provider** | OpenRouter | Anthropic, Ollama, custom |
| **Git Forge** | GitHub | GitLab, Forgejo, Gitea |
| **Terminal** | Alacritty | Ghostty, Kitty |

This means:
- We ship with strong defaults (Ironclaw + Opencode + Linux/Hyprland)
- Users can swap components without rebuilding Octopod
- We can evolve our recommendations as tools improve
- macOS support becomes possible later (just swap the platform abstraction)

### Keeping Scope Focused (Anti-"Go Big Or Go Home")

**YES, we want comprehensive. NO, we don't want bloated.**

This tool manages AI agents doing software development. That's it. Here's what we WON'T build:

| We WON'T Build | Why | What We DO Instead |
|----------------|-----|-------------------|
| Custom LLM | Use existing APIs | OpenRouter, Anthropic, etc. |
| Our own chat interface | Ironclaw already has this | Extend Ironclaw, don't replace |
| Email client | Not core to software dev | Maybe integrate later, not v1 |
| Calendar/scheduling | Too complex, not essential | Simple task queue in SQLite |
| Video conferencing | Way out of scope | Chat is sufficient |
| Billing/invoicing | Not a SaaS tool (yet?) | Track time in activity log |
| Multi-tenancy | Single-user tool first | Each dev runs their own octopod |
| Kubernetes orchestration | Docker is enough | Simple tmux + opencode |
| Our own git forge | GitHub/GitLab exist | Integrate via APIs |

**Comprehensive means:**
- ✅ Decision tracking with full history
- ✅ Agent chat/conversations
- ✅ Task assignment and status
- ✅ Department workflows
- ✅ Cortex (company knowledge) updates
- ✅ Multi-agent departments (5 engineers working together)

**NOT comprehensive means:**
- ❌ Building things that already exist well (git, chat, LLM APIs)
- ❌ Features no one asked for (email, calendar)
- ❌ Premature scale (enterprise multi-tenant)

**Golden Rule:** If it doesn't help AI agents build software better, we don't build it in v1.

---

## Quick Start: What We're Building

### The Three Abstraction Layers

1. **Coordinator Backend** (`src/backends/`)
   - Manages all AI agents (Ironclaw by default)
   - Can swap to OpenClaw later without changing Octopod code

2. **Coding Agent** (`src/coding/`)
   - Writes code for Engineering/QA/DevOps (Opencode by default)
   - Can swap to Aider or Claude Code based on preference

3. **Platform** (`src/platform/`)
   - Manages windows and workspaces (Linux/Hyprland by default)
   - Can swap to generic mode for macOS or other Linux later

### Implementation Roadmap

**Phase 0 (Week 1): Foundation**
- Build all three abstraction layers
- Implement default implementations (Ironclaw, Opencode, Linux/Hyprland)
- Create onboarding wizard and `octopod init`

**Phase 1 (Week 2): CEO Dashboard**
- Build the Super+1 CEO view
- Department status monitoring
- Spawn system with tmux persistence

**Phase 2 (Week 3): Cortex**
- Knowledge management system
- Git-based company memory
- Cortex Agent in Super+9

**Phase 3-4 (Weeks 4-5): Core Departments**
- Product, Engineering, QA with full workflows
- Inter-department communication
- Work item routing

**Phase 5 (Week 6): Remaining Departments**
- Design, DevOps, Marketing, Sales
- Complete the 7-department setup

**Phase 6 (Week 7+): OpenClaw & Polish**
- Add OpenClaw as alternative backend
- Add Aider as alternative coding agent
- Performance optimization and docs

---

## Backend Abstraction Layer

Octopod is designed with a **pluggable backend architecture** that allows swapping between Ironclaw and OpenClaw without changing application logic.

### Design Rationale

**Why Ironclaw as Default?**
- Linux users value security, privacy, and control
- Company operations involve sensitive data (code, business logic, customer info)
- Ironclaw provides sandboxing, audit trails, and governance
- Aligns with local-first, secure-by-default philosophy

**Why Support Both?**
- OpenClaw has broader community ecosystem and plugin support
- Some users may prefer rapid feature development over security guarantees
- Future-proofing against either project's evolution
- User choice respects different risk tolerances

### Backend Trait Design

```rust
// src/backends/mod.rs

pub trait ClawBackend: Send + Sync {
    /// Unique identifier for this backend
    fn name(&self) -> &str;
    
    /// Check if backend binary is installed and accessible
    async fn is_available(&self) -> Result<bool>;
    
    /// Spawn a department agent
    async fn spawn_agent(&self, config: AgentConfig) -> Result<AgentHandle>;
    
    /// Send message to an agent
    async fn send_message(&self, agent: &AgentHandle, message: &str) -> Result<()>;
    
    /// Get agent status
    async fn agent_status(&self, agent: &AgentHandle) -> Result<AgentStatus>;
    
    /// List running agents
    async fn list_agents(&self) -> Result<Vec<AgentHandle>>;
    
    /// Stop an agent
    async fn stop_agent(&self, agent: &AgentHandle) -> Result<()>;
    
    /// Get backend-specific configuration options
    fn config_options(&self) -> Vec<ConfigOption>;
}

/// Factory for creating backends
pub struct BackendFactory;

impl BackendFactory {
    pub fn create(backend_type: BackendType) -> Box<dyn ClawBackend> {
        match backend_type {
            BackendType::Ironclaw => Box::new(IronclawBackend::new()),
            BackendType::Openclaw => Box::new(OpenclawBackend::new()),
        }
    }
    
    pub fn default() -> Box<dyn ClawBackend> {
        Box::new(IronclawBackend::new())
    }
}

pub enum BackendType {
    Ironclaw,
    Openclaw,
}
```

### Configuration

**~/.config/octopod/config.toml:**
```toml
[backend]
# Options: "ironclaw" (default) or "openclaw"
claw = "ironclaw"

# Backend-specific settings are nested under [backend.ironclaw] or [backend.openclaw]
[backend.ironclaw]
security_level = "strict"  # strict | standard | permissive
audit_logging = true
data_policy = "local_only"

[backend.openclaw]
# OpenClaw-specific settings when supported
plugin_directory = "~/.config/openclaw/plugins"
auto_update = false
```

### Implementation Priority

**Phase 0-2:** Implement IronclawBackend only
- Focus on making Ironclaw integration excellent
- Build all features around Ironclaw capabilities
- Create solid abstraction layer

**Phase 5+:** Add OpenclawBackend
- Implement trait for OpenClaw
- Add configuration migration
- Test feature parity
- Document differences

### Feature Parity Matrix

| Feature | Ironclaw | OpenClaw | Notes |
|---------|----------|----------|-------|
| Agent Spawning | ✅ | ✅ | Core functionality |
| Message Passing | ✅ | ✅ | Via channels/gateway |
| Sandboxing | ✅ | ⚠️ | Ironclaw native, OpenClaw limited |
| Audit Logging | ✅ | ❌ | Ironclaw only |
| Tool Execution | ✅ | ✅ | Both support tool use |
| Plugin System | ❌ | ✅ | OpenClaw advantage |
| Community Tools | Limited | Extensive | OpenClaw advantage |
| Security Policies | ✅ | ❌ | Ironclaw advantage |

---

## Data Storage Layer

**NO, a database is NOT overkill.** SQLite is essential for Octopod's core functionality.

### Why SQLite?

While company knowledge lives in Git (cortex), **operational state needs structured storage**:

- **Decision history**: Who decided what, when, with what context
- **Conversations**: Chat history across departments, searchable, timestamped
- **Agent state**: Who's working on what, task assignments, status
- **Activity logs**: Audit trail of all actions
- **Task queue**: What needs to be done, priorities, assignments

Git is great for documents, terrible for querying "what did Engineering decide last Tuesday?"

### Design Philosophy: SQLite + Git

| Data Type | Storage | Why |
|-----------|---------|-----|
| **Company Knowledge** | Git (cortex/) | Documents, specs, long-term memory |
| **Decisions** | SQLite | Query by time, status, department, searchable |
| **Conversations** | SQLite | Chat history, searchable, real-time |
| **Agent State** | SQLite | Current assignments, who's active |
| **Tasks** | SQLite | Queue, priorities, assignments |
| **Activity Log** | SQLite + export to MD | Audit trail, can export to git |

### Implementation

```rust
// src/state/mod.rs

pub struct StateManager {
    db: SqlitePool,
    project_dir: PathBuf,
}

impl StateManager {
    /// Initialize database (creates tables if not exist)
    pub async fn init(project_dir: &Path) -> Result<Self>;
    
    /// Log a decision
    pub async fn log_decision(&self, decision: Decision) -> Result<DecisionId>;
    
    /// Record a conversation message
    pub async fn log_message(&self, message: Message) -> Result<MessageId>;
    
    /// Query decisions by department/time
    pub async fn get_decisions(&self, filter: DecisionFilter) -> Result<Vec<Decision>>;
    
    /// Get conversation history
    pub async fn get_conversation(&self, department: DepartmentId) -> Result<Vec<Message>>;
    
    /// Update agent state
    pub async fn update_agent_state(&self, agent_id: &str, state: AgentState) -> Result<()>;
}
```

### Database Schema (Essential Tables)

```sql
-- Decisions table
CREATE TABLE decisions (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    department TEXT NOT NULL,
    priority INTEGER, -- 0=P0, 1=P1, 2=P2, 3=P3
    status TEXT, -- pending, approved, rejected, escalated
    requested_by TEXT,
    approved_by TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    resolved_at TIMESTAMP,
    context_json TEXT -- JSON blob for extra context
);

-- Messages table (chat/conversations)
CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    from_agent TEXT NOT NULL,
    to_agent TEXT, -- NULL = broadcast/public
    department TEXT,
    content TEXT NOT NULL,
    message_type TEXT, -- chat, decision_request, decision_response, command
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    metadata_json TEXT
);

-- Agents table
CREATE TABLE agents (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    department TEXT NOT NULL,
    status TEXT, -- idle, working, error, offline
    current_task TEXT,
    spawned_at TIMESTAMP,
    last_seen TIMESTAMP
);

-- Tasks table
CREATE TABLE tasks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    department TEXT NOT NULL,
    assigned_to TEXT,
    status TEXT, -- todo, in_progress, blocked, done
    priority INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP,
    related_decision_id TEXT,
    FOREIGN KEY (assigned_to) REFERENCES agents(id),
    FOREIGN KEY (related_decision_id) REFERENCES decisions(id)
);

-- Activity log
CREATE TABLE activity_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    actor TEXT, -- agent_id or "ceo"
    action TEXT, -- spawn, kill, approve, reject, message
    target TEXT, -- department or agent
    details TEXT
);
```

### Export to Markdown

For git integration, regularly export to markdown:

```rust
// Export decisions to cortex/decisions/
pub async fn export_decisions_to_markdown(&self, cortex_dir: &Path) -> Result<()>;

// Export conversations to cortex/conversations/
pub async fn export_conversations_to_markdown(&self, cortex_dir: &Path) -> Result<()>;
```

---

## Coding Agent Abstraction Layer

Just like the coordinator backend, Octopod abstracts the **coding agent** (the tool that actually writes code) behind a trait. This allows users to choose their preferred coding assistant.

### Why Abstract Coding Agents?

Different coding agents excel at different things:
- **Opencode** - Fast, good for quick edits, works great on Linux
- **Aider** - Excellent multi-file refactoring, architect mode
- **Claude Code** - Deep reasoning, complex architecture (if CLI available)
- **Continue.dev** - IDE-like experience in terminal

Users should be able to choose based on their workflow and preferences.

### Coding Agent Trait Design

```rust
// src/coding/mod.rs

pub trait CodingAgent: Send + Sync {
    /// Agent identifier
    fn name(&self) -> &str;
    
    /// Check if agent binary is installed
    async fn is_available(&self) -> Result<bool>;
    
    /// Spawn coding session for a repo
    async fn spawn_session(
        &self,
        repo_path: &Path,
        task: &str,
        config: CodingConfig,
    ) -> Result<CodingSession>;
    
    /// Get session status
    async fn session_status(&self, session: &CodingSession) -> Result<CodingStatus>;
    
    /// Send message to coding agent
    async fn send_message(&self, session: &CodingSession, message: &str) -> Result<()>;
    
    /// Get visible output (for TDL-like visibility)
    async fn get_output(&self, session: &CodingSession) -> Result<String>;
    
    /// Stop session
    async fn stop_session(&self, session: &CodingSession) -> Result<()>;
    
    /// Check if agent supports visible progress (like tdl c)
    fn supports_visible_progress(&self) -> bool;
}

pub struct CodingConfig {
    pub workspace: Option<u8>,      // Hyprland workspace, None for inline
    pub model: Option<String>,      // Override default model
    pub visible_mode: bool,         // Show tdl-like progress
    pub context_files: Vec<PathBuf>, // Files to include in context
}

pub enum CodingAgentType {
    Opencode,      // Default - fast, lightweight
    Aider,         // Multi-file expert
    ClaudeCode,    // Deep reasoning (future)
    Continue,      // IDE-like (future)
    Custom(String), // User-defined
}
```

### Factory and Configuration

```rust
// src/coding/factory.rs

pub struct CodingAgentFactory;

impl CodingAgentFactory {
    pub fn create(agent_type: CodingAgentType) -> Box<dyn CodingAgent> {
        match agent_type {
            CodingAgentType::Opencode => Box::new(OpencodeAgent::new()),
            CodingAgentType::Aider => Box::new(AiderAgent::new()),
            CodingAgentType::ClaudeCode => Box::new(ClaudeCodeAgent::new()),
            CodingAgentType::Continue => Box::new(ContinueAgent::new()),
            CodingAgentType::Custom(name) => Box::new(CustomAgent::new(name)),
        }
    }
    
    pub fn default() -> Box<dyn CodingAgent> {
        Box::new(OpencodeAgent::new())
    }
}
```

**~/.config/octopod/config.toml:**
```toml
[coding]
# Options: "opencode" (default), "aider", "claude-code", "continue"
agent = "opencode"

[coding.opencode]
model = "claude-3-5-sonnet-20241022"
inline_mode = false  # Use tmux session vs inline

[coding.aider]
model = "claude-3-5-sonnet-20241022"
architect_mode = true
auto_commit = false
```

### Implementation Priority

**Phase 0-3:** Implement OpencodeAgent only
- Focus on perfect Opencode integration
- Build visible progress (tdl-like) integration
- Make it excellent for Engineering/QA/DevOps

**Phase 4+:** Add AiderAgent
- Many users prefer Aider for complex refactoring
- Architect mode is powerful for planning
- Good alternative for users who want it

**Future:** Add ClaudeCodeAgent, ContinueAgent as they mature

### Feature Comparison Matrix

| Feature | Opencode | Aider | Claude Code | Continue |
|---------|----------|-------|-------------|----------|
| **Speed** | ⚡⚡⚡ | ⚡⚡ | ⚡⚡⚡ | ⚡⚡ |
| **Multi-file** | Good | Excellent | Excellent | Good |
| **Reasoning** | Good | Good | Excellent | Good |
| **Terminal-native** | ✅ | ❌ | ❌ | ❌ |
| **TUI Mode** | ✅ | Limited | ❌ | ❌ |
| **Visible Progress** | ✅ | ❌ | ❌ | ❌ |
| **Architect Mode** | ❌ | ✅ | ✅ | ❌ |
| **Auto-commit** | ✅ | ✅ | ✅ | ✅ |
| **Terminal Size** | Small | Medium | Large | Large |

**Recommendation:** Start with Opencode (already installed, terminal-native, TUI mode). Add Aider later for users who need multi-file refactoring.

---

## System Architecture

### High-Level Component Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              USER (CEO)                                      │
│                         Super+1: CEO Dashboard                              │
└─────────────────────┬───────────────────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         OMINKY ORCHESTRATOR                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │   Spawn      │  │  Workspace   │  │   Config     │  │   Status     │     │
│  │   Manager    │  │   Manager    │  │   Manager    │  │   Monitor    │     │
│  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘     │
└─────────────────────┬───────────────────────────────────────────────────────┘
                      │
        ┌─────────────┼─────────────┐
        ▼             ▼             ▼
┌──────────────┐ ┌────────────┐ ┌──────────────┐
│   IRONCLAW   │ │  OPCODE    │ │    TDL       │
│ Coordinator  │ │  (Coding)  │ │  (Dev Loop)  │
│              │ │            │ │              │
│ • Persistent │ │ • Spawned  │ │ • Visible    │
│ • Agent Mgmt │ │   per repo │ │   work       │
│ • Inter-dept │ │ • Stateful │ │ • Progress   │
│   comms      │ │ • Coding   │ │   tracking   │
└──────┬───────┘ └─────┬──────┘ └──────┬───────┘
       │               │               │
       └───────────────┴───────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         DEPARTMENT WORKSPACES                                │
│                                                                              │
│  Super+2      Super+3      Super+4      Super+5      Super+6                │
│  ┌─────┐     ┌─────┐     ┌─────┐     ┌─────┐     ┌─────┐                   │
│  │Product│    │Engineering│  │  QA  │    │Design │    │DevOps │                   │
│  │      │    │ (Opencode)│  │      │    │      │    │      │                   │
│  └─────┘     └─────┘     └─────┘     └─────┘     └─────┘                   │
│                                                                              │
│  Super+7      Super+8      Super+9                                          │
│  ┌─────┐     ┌─────┐     ┌─────┐                                           │
│  │Marketing│   │  Sales │    │Cortex │                                           │
│  │      │    │      │    │Agent │                                           │
│  └─────┘     └─────┘     └─────┘                                           │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Data Flow (Updated Architecture)

```
                    ┌─────────────────────────────────────┐
                    │           IRONCLAW                  │
                    │     (Single Orchestrator)           │
                    │  ┌─────────┐  ┌───────────────┐     │
                    │  │ WebSocket│  │ Message Bus   │     │
                    │  │ Server   │  │ & Router      │     │
                    │  └────┬────┘  └───────┬───────┘     │
                    │       │               │             │
                    │  ┌────┴───────────────┴────┐        │
                    │  │     SQLite State DB     │        │
                    │  │ (decisions, chats, etc) │        │
                    └────┬───────────────────────┬────────┘
                         │                       │
            WebSocket    │                       │  Spawn/Control
            Connections  │                       │
        ┌────────────────┴────────┐   ┌─────────┴────────────┐
        │                         │   │                      │
   ┌────▼────┐            ┌───────▼───▼──────┐  ┌─────────────▼──────┐
   │  CEO    │            │ Department TUIs  │  │  Opencode Agents   │
   │Dashboard│            │ (Product, Eng..) │  │  (Engineering)     │
   │         │            │                  │  │  - Alice (Frontend)│
   │         │            │  Tabs:           │  │  - Bob (Backend)   │
   └─────────┘            │  - Work          │  │  - Carol (DevOps)  │
                          │  - Chat (Slack   │  └────────────────────┘
                          │    style)        │
                          │  - Activity      │
                          │  - Files         │
                          └──────────────────┘
```

**Key Design Principles:**

1. **Ironclaw is the brain** - ONE instance orchestrates everything via WebSocket/API
2. **Opencode agents do the work** - Engineering/QA departments spawn multiple opencode agents
3. **SQLite stores state** - Decisions, conversations, agent state (NOT overkill, essential)
4. **Department TUIs are clients** - Connect to Ironclaw, show chat + work + activity
5. **CEO sees everything** - Dashboard connects to same Ironclaw, shows all departments

**Communication Flow:**

```
1. Engineering agent (opencode) needs CEO decision
   └─> Sends message via CLI/API to Ironclaw
   
2. Ironclaw stores in SQLite, broadcasts to CEO Dashboard
   └─> Shows in Decisions tab with context
   
3. CEO approves in Dashboard
   └─> WebSocket message to Ironclaw
   
4. Ironclaw routes to Engineering department
   └─> Appears in their Chat tab
   
5. Engineering agent sees decision, continues work
   └─> Updates visible in both TUIs
```

**Department Types:**

| Department | Primary Tool | Multi-Agent? | Example Agents |
|------------|--------------|--------------|----------------|
| **Product** | Ironclaw | No | 1 PM agent |
| **Engineering** | Opencode | Yes | Alice (FE), Bob (BE), Carol (DevOps) |
| **QA** | Opencode | Yes | Tester1, Tester2, Automation |
| **Marketing** | Ironclaw | Yes | Content, Analytics, Campaigns |
| **Sales** | Ironclaw | No | 1 Sales agent |

**Data Storage:**

- **SQLite**: Decisions, conversations, tasks, agent state
- **Git (cortex/)**: Company docs, specs, long-term knowledge
- **Export**: Regularly sync SQLite → markdown in cortex/

---

## Workspace-Department Mapping

| Workspace | Department | Primary Tool | Visible Work | Key Responsibilities |
|-----------|-----------|--------------|--------------|---------------------|
| Super+1 | **CEO Dashboard** | Custom TUI | All departments, issue board, activity feed | Overview, prioritization, unblocking |
| Super+2 | **Product** | Ironclaw | Roadmap, PRDs, user stories | Requirements, prioritization, stakeholder management |
| Super+3 | **Engineering** | Opencode + tdl | Code editor, test output, build logs | Feature implementation, code review, architecture |
| Super+4 | **QA Lab** | Ironclaw + Browser | Test results, browser automation, bug reports | Testing, validation, quality assurance |
| Super+5 | **Design** | Ironclaw | Design files, prototypes, asset generation | UX/UI, branding, creative assets |
| Super+6 | **DevOps** | Ironclaw + Shell | Deployment logs, infra status, monitoring | Infrastructure, CI/CD, security |
| Super+7 | **Marketing** | Ironclaw | Campaign docs, analytics, content calendar | Campaigns, content, lead generation |
| Super+8 | **Sales** | Ironclaw | CRM view, outreach templates, demo scripts | Prospecting, demos, closing |
| Super+9 | **Cortex** | Ironclaw | Git operations, knowledge queries | Knowledge management, documentation |

---

## Platform Abstraction Layer

Octopod is built for **Linux/Hyprland first**, but designed with platform abstractions that could enable macOS or generic Linux support in the future.

### Platform Trait

```rust
// src/platform/mod.rs

pub trait Platform: Send + Sync {
    fn name(&self) -> &str;
    
    /// Check if platform is available
    fn is_available(&self) -> bool;
    
    /// Spawn a window in specific workspace
    async fn spawn_in_workspace(
        &self,
        command: &str,
        workspace: u8,
        config: WindowConfig,
    ) -> Result<WindowHandle>;
    
    /// List active windows
    async fn list_windows(&self) -> Result<Vec<WindowInfo>>;
    
    /// Switch to workspace
    async fn focus_workspace(&self, workspace: u8) -> Result<()>;
    
    /// Get current workspace
    async fn current_workspace(&self) -> Result<u8>;
    
    /// Platform supports workspace-per-department
    fn supports_workspaces(&self) -> bool;
}

pub enum PlatformType {
    Omarchy,    // Full Hyprland integration (default)
    Generic,    // Single-window TUI mode (future: macOS, other Linux)
}
```

### Omarchy Platform (Default)

**Features:**
- Hyprland workspace integration (Super+1 through Super+9)
- tmux session management per department
- Waybar status integration
- Native Omarchy theming

**Implementation:**
```rust
// src/platform/omarchy.rs

pub struct OmarchyPlatform {
    hyprland: HyprlandClient,
    tmux: TmuxManager,
    waybar: WaybarIntegration,
}

impl Platform for OmarchyPlatform {
    async fn spawn_in_workspace(&self, command: &str, workspace: u8, config: WindowConfig) -> Result<WindowHandle> {
        // Use hyprctl to dispatch to workspace
        // Spawn terminal with tmux session
        // Return handle for monitoring
    }
}
```

### Generic Platform (Future)

**For macOS or non-Hyprland Linux:**
- Single TUI window with tabs/panes for departments
- No workspace spreading (Super+1-9)
- Still functional, just less integrated with window manager workspaces

**Implementation:**
```rust
// src/platform/generic.rs

pub struct GenericPlatform {
    // No external dependencies
}

impl Platform for GenericPlatform {
    async fn spawn_in_workspace(&self, command: &str, _workspace: u8, config: WindowConfig) -> Result<WindowHandle> {
        // Spawn in current terminal as tmux pane/window
        // All departments visible via TUI navigation
    }
    
    fn supports_workspaces(&self) -> bool {
        false  // No native workspace support
    }
}
```

### Configuration

**~/.config/octopod/config.toml:**
```toml
[platform]
# Options: "omarchy" (default), "generic"
type = "omarchy"

[platform.omarchy]
terminal = "alacritty"
tmux_session_prefix = "octopod"
use_waybar = true

[platform.generic]
# Settings for non-Hyprland platforms (future)
window_mode = "tabs"  # tabs | panes
```

### Platform Detection

```rust
fn detect_platform() -> PlatformType {
    if is_omarchy() || has_hyprland() {
        PlatformType::Omarchy
    } else {
        PlatformType::Generic
    }
}

fn is_omarchy() -> bool {
    Path::new("/usr/bin/omarchy-version").exists()
        || env::var("XDG_CURRENT_DESKTOP").unwrap_or_default() == "Omarchy"
}

fn has_hyprland() -> bool {
    Command::new("which").arg("hyprctl").output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
```

### Implementation Priority

**Phase 0-5:** Implement OmarchyPlatform only
- Perfect Hyprland integration
- Full workspace support
- tmux + Hyprland + Waybar

**Phase 6+:** Add GenericPlatform
- For macOS support
- For non-Hyprland Linux
- Single-window mode

**Rationale:**
- Focus on making Hyprland/Linux experience excellent
- Generic platform is a "nice to have" for later
- Don't compromise native workspace features for cross-platform

---

## Configuration Architecture

### Global Configuration (~/.config/octopod/)

```
~/.config/octopod/
├── config.toml                 # Global settings
├── themes/
│   └── octopus.toml           # Color scheme & branding
├── companies/                 # Multiple company support
│   ├── acme-corp/
│   │   └── config.toml
│   └── my-startup/
│       └── config.toml
└── templates/                 # Reusable department configs
    ├── saas-standard.toml
    ├── lean-startup.toml
    └── enterprise.toml
```

**~/.config/octopod/config.toml:**
```toml
[ui]
terminal = "alacritty"        # alacritty | ghostty | kitty
theme = "octopus"
animation_enabled = true

[workspaces]
base_workspace = 1            # Super+1 is CEO
workspace_count = 9

[tools]
ironclaw_path = "ironclaw"
opencode_path = "opencode"
tdl_path = "tdl"
git_path = "git"
gh_path = "gh"
glab_path = "glab"

[backend]
# Coordinator backend: "ironclaw" (default) or "openclaw"
coordinator = "ironclaw"

[backend.ironclaw]
security_level = "strict"
audit_logging = true

[coding]
# Coding agent: "opencode" (default), "aider", "claude-code"
agent = "opencode"

[coding.opencode]
model = "claude-3-5-sonnet-20241022"
visible_progress = true

[platform]
# Platform: "omarchy" (default) or "generic"
type = "omarchy"

[platform.omarchy]
terminal = "alacritty"
tmux_session_prefix = "octopod"
use_waybar = true

[defaults]
provider = "openrouter"
model = "moonshotai/kimi-k2.5"
```

### Company Configuration (./.octopod/)

```
./.octopod/
├── config.toml                # Company-specific settings
├── .gitignore                # Ignore subdirectories (for multi-repo)
├── departments.toml          # Department definitions
├── agents/
│   ├── product.toml
│   ├── engineering.toml
│   ├── qa.toml
│   ├── design.toml
│   ├── devops.toml
│   ├── marketing.toml
│   ├── sales.toml
│   └── cortex.toml
├── cortex/                   # Knowledge base
│   ├── company/
│   │   ├── vision.md
│   │   ├── values.md
│   │   ├── guidelines.md
│   │   └── decisions/
│   ├── product/
│   │   ├── roadmap.md
│   │   ├── prds/
│   │   └── user_stories/
│   ├── engineering/
│   │   ├── backend/
│   │   │   ├── architecture.md
│   │   │   └── api_specs/
│   │   ├── frontend/
│   │   │   ├── design_system.md
│   │   │   └── component_library/
│   │   └── shared/
│   │       ├── coding_standards.md
│   │       └── deployment.md
│   ├── qa/
│   │   ├── test_plans/
│   │   └── testing_strategy.md
│   ├── design/
│   │   ├── brand_guidelines.md
│   │   └── design_system/
│   ├── devops/
│   │   ├── infrastructure/
│   │   └── runbooks/
│   ├── marketing/
│   │   ├── campaigns/
│   │   └── content_calendar/
│   ├── sales/
│   │   ├── playbooks/
│   │   └── demo_scripts/
│   └── skills/
│       └── SKILL.md          # Agent skill definitions
└── state/                    # Persistent agent state
    ├── product/
    ├── engineering/
    ├── qa/
    ├── design/
    ├── devops/
    ├── marketing/
    ├── sales/
    └── cortex/
```

**./.octopod/config.toml:**
```toml
[company]
name = "Acme Corp"
description = "AI-powered widget platform"
website = "https://acme.example.com"

[git]
forge = "github"              # github | gitlab | forgejo
default_repo = "acme-platform"
issue_tracker = "github"      # Where to file issues

[cortex]
auto_commit = true
commit_message_template = "[Cortex] {summary}"

[departments]
enabled = ["product", "engineering", "qa", "design", "devops", "marketing", "sales"]

[communication]
inter_dept_chat = true        # Agents can message each other
decision_logging = true       # Log decisions to cortex
```

---

## Department Specifications

### 1. Product Department (Super+2)

**Role:** Product Manager
**Primary Model:** Strong reasoning model (Claude 3.5 Sonnet, GPT-4o)
**Skills:** Roadmap planning, user story writing, prioritization, stakeholder communication

**Cortex Structure:**
```
cortex/product/
├── roadmap.md               # Current roadmap
├── backlog.md               # Prioritized backlog
├── prds/                    # Product Requirements Documents
│   ├── 001-auth-system.md
│   └── 002-billing-v2.md
├── user_stories/
│   └── us-001-signup-flow.md
└── decisions/
    └── 2024-01-15-prioritization-framework.md
```

**Tools:**
- Query cortex for context
- Create/update PRDs
- File GitHub issues
- Communicate with Engineering
- Present roadmap to CEO

**TUI Layout:**
```
┌────────────────────────────────────────────┐
│ 🎯 Product - Acme Corp Roadmap Q1 2026    │
├────────────────────────────────────────────┤
│ Status: Planning Sprint 3                 │
│ Active Issues: 12 | Blocked: 2 | Done: 45 │
├────────────────────────────────────────────┤
│ ProductAgent: Should we prioritize the   │
│ billing integration over the auth system │
│ refactor? Engineering says auth is       │
│ blocking 3 other features...             │
├────────────────────────────────────────────┤
│ [CEO] > prioritize billing              │
├────────────────────────────────────────────┤
│ ProductAgent: Roger that. Updating       │
│ roadmap and notifying Engineering...     │
└────────────────────────────────────────────┘
```

### 2. Engineering Department (Super+3)

**Role:** Senior Full-Stack Engineers
**Primary Tool:** Opencode + tdl
**Model:** Code-capable model (Claude 3.5 Sonnet, DeepSeek Coder)
**Skills:** Feature implementation, code review, testing, architecture

**Unique Features:**
- Each repo/domain gets its own opencode instance
- Visible `tdl c` output in bottom pane
- Can spawn sub-agents for different tasks
- Integrates with QA for handoff

**Cortex Structure:**
```
cortex/engineering/
├── backend/
│   ├── architecture.md
│   ├── api_specs/
│   └── database_schema/
├── frontend/
│   ├── design_system.md
│   └── component_library/
└── shared/
    ├── coding_standards.md
    └── deployment.md
```

**TUI Layout (2-pane):**
```
┌────────────────────────────────────────────┐
│ 🛠️ Engineering - auth-service             │
├────────────────────────────────────────────┤
│ EngAgent: Starting JWT implementation...   │
│ EngAgent: Created auth middleware          │
│ EngAgent: Writing tests...                 │
├────────────────────────────────────────────┤
│ [CEO] > What's the status on OAuth?       │
├────────────────────────────────────────────┤
│ EngAgent: OAuth is 80% complete. Blocked  │
│ on product decision about social providers│
└────────────────────────────────────────────┘
                    ────────────────────────────────────────────
🛠️ tdl c - auth-service (Running)
$ cargo test
running 5 tests
test auth::test_jwt ... ok
test auth::test_oauth ... ok
test auth::test_refresh ... ok
test auth::test_logout ... ok
test auth::test_middleware ... ok

test result: ok. 5 passed; 0 failed; 0 ignored
```

### 3. QA Lab (Super+4)

**Role:** QA Engineers
**Primary Tool:** Ironclaw + Browser automation
**Model:** Detail-oriented model (Claude 3.5 Sonnet)
**Skills:** Test planning, browser automation, bug reporting, regression testing

**Unique Features:**
- Can run headful browser tests (visible automation)
- Automated screenshot comparison
- Bug reports auto-filed to GitHub Issues
- Integrates with Engineering for reproduction

**Cortex Structure:**
```
cortex/qa/
├── test_plans/
│   ├── tp-001-user-signup.md
│   └── tp-002-checkout-flow.md
├── testing_strategy.md
└── test_data/
    └── fixtures/
```

**TUI Layout:**
```
┌────────────────────────────────────────────┐
│ 🧪 QA Lab - Test: user-signup-flow        │
├────────────────────────────────────────────┤
│ QAAgent: Running E2E tests for signup...  │
│ QAAgent: Test 1/5: Valid email signup ✓   │
│ QAAgent: Test 2/5: Invalid email ✓        │
│ QAAgent: Test 3/5: Duplicate email...     │
├────────────────────────────────────────────┤
│ [CEO] > Any blockers?                     │
├────────────────────────────────────────────┤
│ QAAgent: Found bug in duplicate email     │
│ handling. Filing issue #127...            │
└────────────────────────────────────────────┘
                    ────────────────────────────────────────────
🎥 Browser View (Playwright)
[Live browser showing test execution]
```

### 4. Design Department (Super+5)

**Role:** UX/UI Designers
**Primary Tool:** Ironclaw + Design tools (Figma API, etc.)
**Model:** Creative model (Claude 3.5 Sonnet, GPT-4o)
**Skills:** UX research, wireframing, design systems, brand assets

### 5. DevOps Department (Super+6)

**Role:** DevOps Engineers
**Primary Tool:** Ironclaw + Shell
**Model:** Systems-focused model
**Skills:** Infrastructure, CI/CD, monitoring, security

### 6. Marketing Department (Super+7)

**Role:** Marketing Managers
**Primary Tool:** Ironclaw
**Model:** Creative + analytical model
**Skills:** Campaign planning, content creation, analytics, SEO

### 7. Sales Department (Super+8)

**Role:** Sales Representatives
**Primary Tool:** Ironclaw + CRM integrations
**Model:** Conversational model
**Skills:** Prospecting, demos, objection handling, closing

### 8. Cortex Agent (Super+9)

**Role:** Knowledge Librarian
**Primary Tool:** Ironclaw + Git
**Model:** Research-focused model
**Skills:** Information retrieval, documentation, Git operations

**Responsibilities:**
- Answer questions from other agents about company knowledge
- Manage Git operations for the cortex
- Maintain documentation standards
- Track decisions and learnings

---

## Backend Implementation: Ironclaw (Default)

### Ironclaw as Coordinator

*This is the default backend implementation. See [Backend Abstraction Layer](#backend-abstraction-layer) for the trait definition and OpenClaw support roadmap.*

**Central Ironclaw Instance** runs persistently:

**Central Ironclaw Instance** runs persistently:

```rust
// Ironclaw configuration for coordinator
// ~/.config/ironclaw/octopod-coordinator.toml

[agent]
name = "OctopodCoordinator"
persona = "coordinator"

[channels]
gateway_enabled = true
gateway_port = 28473
cli_enabled = true

[tools]
// Custom tools for Octopod
enabled = ["spawn_agent", "send_message", "query_cortex", "file_issue"]
```

**Department-Specific Ironclaw Instances**:
Each department runs its own ironclaw with a specific persona:

```rust
// ~/.config/ironclaw/octopod-product.toml

[agent]
name = "ProductManager"
persona = "product"
department = "product"

[llm]
backend = "openrouter"
model = "anthropic/claude-3.5-sonnet"

[octopod]
cortex_path = "./.octopod/cortex"
state_path = "./.octopod/state/product"
```

### Inter-Department Communication

Agents communicate via Ironclaw's built-in channels:

```rust
// Product agent messages Engineering
#[tool]
async fn request_engineering_review(prd_id: String) -> Result<()> {
    self.send_message(
        to: "engineering",
        message: format!("New PRD ready for review: {}", prd_id),
        priority: "high"
    ).await
}

// Engineering agent responds
#[tool]
async fn provide_estimate(task_id: String, hours: u32) -> Result<()> {
    self.send_message(
        to: "product",
        message: format!("Estimate for {}: {} hours", task_id, hours)
    ).await
}
```

### Opencode Spawning

Ironclaw can spawn opencode instances for coding tasks:

```rust
#[tool]
async fn spawn_coding_agent(
    repo: String,
    task: String,
    workspace: u8
) -> Result<AgentHandle> {
    // Spawn opencode in specific workspace
    let cmd = format!(
        "hyprctl dispatch workspace {} && \
         alacritty -e opencode --task '{}' --repo {}",
        workspace, task, repo
    );
    
    let handle = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(&cmd)
        .spawn()?;
    
    // Register with coordinator for monitoring
    self.register_coding_agent(handle.id(), repo).await?;
    
    Ok(AgentHandle { pid: handle.id() })
}
```

---

## CLI Interface Design

### Commands

```bash
# Initialize Octopod in current directory
octopod init [--template saas-standard|lean|enterprise]

# Spawn commands
octopod spawn                    # Interactive: choose which departments
octopod spawn --all              # Spawn all departments
octopod spawn --dashboard        # Spawn only CEO dashboard
octopod spawn product            # Spawn specific department
octopod spawn product qa         # Spawn multiple departments

# Department management
octopod status                   # Show all department statuses
octopod stop <dept>              # Stop a department
octopod restart <dept>           # Restart a department
octopod logs <dept>              # View department logs

# Cortex management
octopod cortex query "<question>"    # Query company knowledge
octopod cortex update                  # Pull latest cortex changes
octopod cortex commit "<message>"      # Commit cortex changes

# Issue management
octopod issues list              # List all issues across repos
octopod issues create --title "..." --dept engineering
octopod issues assign <id> --to qa

# Configuration
octopod config get <key>
octopod config set <key> <value>
octopod onboard                  # Re-run onboarding wizard

# Utilities
octopod doctor                   # Run diagnostics
octopod version
octopod help
```

### Onboarding Flow

```
┌─────────────────────────────────────────────┐
│ 🐙 Welcome to Octopod                        │
│                                             │
│   Many-Armed Company Orchestration          │
│                                             │
│ Press Enter to continue...                  │
└─────────────────────────────────────────────┘

[Step 1: Prerequisites Check]

Octopod requires the following tools. The onboarding wizard will check for each
and provide installation instructions if anything is missing.

**Core Requirements:**
✓ Linux (tested on Omarchy with Hyprland)
✓ tmux (terminal multiplexer)
✓ Hyprland (Wayland compositor) - for workspace integration
✓ Git (version control)
✓ GitHub CLI (gh) or GitLab CLI (glab) - for issue management

**AI/LLM Tools:**
✓ ironclaw - Coordinator backend (install: cargo install ironclaw)
✓ opencode - Coding agent (install: curl -fsSL https://opencode.ai/install | bash)
  Alternative: paru -S opencode (on Omarchy)
✓ tdl - Development loop visibility (install: cargo install tdl)

**Optional but Recommended:**
✓ cargo (Rust toolchain) - for installing Rust-based tools
✓ node/npm - some tools may require Node.js
✓ docker - for sandboxed operations (ironclaw feature)

**Linux Installation (Omarchy Recommended):**
```bash
# Core tools (usually pre-installed on Omarchy)
sudo pacman -S tmux git github-cli

# Rust toolchain (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# AI tools
cargo install ironclaw
curl -fsSL https://opencode.ai/install | bash  # or: paru -S opencode
cargo install tdl
```

✓ All prerequisites met!

[Step 2: Company Setup]
Company Name: Acme Corp
Description: AI-powered widget platform
Website (optional): https://acme.example.com

[Step 3: Git Configuration]
Forge: GitHub
Default Repository: acme-corp/platform
GitHub CLI (gh) detected ✓

[Step 4: LLM Provider]
Provider: OpenRouter
API Key: sk-or-... ✓ (detected from env)
Model: moonshotai/kimi-k2.5

[Step 5: Department Configuration]
Template: SaaS Standard
Departments: Product, Engineering, QA, Design, DevOps, Marketing, Sales

[Step 6: Terminal Selection]
Terminal: Alacritty ✓ (detected)

[Step 7: Initialization]
✓ Created ./.octopod/
✓ Initialized Git repository
✓ Created department configurations
✓ Generated SKILL.md templates

🚀 Ready! Next steps:
  1. octopod spawn --all    # Launch all departments
  2. Super+1 for CEO Dashboard
  3. Super+2-9 for departments
```

---

## Data Models

### Company

```rust
struct Company {
    id: String,                    // UUID
    name: String,
    description: String,
    website: Option<String>,
    root_path: PathBuf,            // Where .octopod/ lives
    created_at: DateTime<Utc>,
    config: CompanyConfig,
}

struct CompanyConfig {
    git_forge: GitForge,           // GitHub | GitLab | Forgejo
    default_repo: String,
    issue_tracker: IssueTracker,
    departments: Vec<DepartmentId>,
}
```

### Department

```rust
struct Department {
    id: DepartmentId,              // product | engineering | qa | etc.
    name: String,
    description: String,
    workspace: u8,                 // Super+N
    status: DepartmentStatus,
    config: DepartmentConfig,
    state: DepartmentState,
}

enum DepartmentStatus {
    Stopped,
    Starting,
    Running,
    Error(String),
    Paused,
}

struct DepartmentConfig {
    persona: String,               // ironclaw persona file
    model: String,                 // LLM model ID
    skills: Vec<String>,           // Skill IDs from SKILL.md
    tools: Vec<String>,            // Enabled tool names
    auto_spawn: bool,              // Start with --all?
}

struct DepartmentState {
    pid: Option<u32>,              // Process ID if running
    last_activity: DateTime<Utc>,
    current_task: Option<String>,
    messages: Vec<ChatMessage>,
}
```

### Work Item

```rust
struct WorkItem {
    id: String,
    title: String,
    description: String,
    item_type: WorkItemType,       // Feature | Bug | Task | Research
    status: WorkItemStatus,
    priority: Priority,
    department: DepartmentId,      // Owning department
    assignee: Option<AgentId>,
    parent: Option<String>,        // Parent work item ID
    children: Vec<String>,         // Sub-tasks
    github_issue: Option<u64>,     // Linked GitHub issue
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
}
```

### Agent

```rust
struct Agent {
    id: AgentId,
    name: String,
    department: DepartmentId,
    role: String,
    personality: String,           // How they communicate
    skills: Vec<Skill>,
    state: AgentState,
}

struct AgentState {
    conversation_history: Vec<Message>,
    active_tasks: Vec<String>,
    context: HashMap<String, Value>, // Department-specific state
}
```

### Message

```rust
struct Message {
    id: String,
    from: AgentId,
    to: Option<AgentId>,           // None = broadcast
    department: DepartmentId,
    content: String,
    message_type: MessageType,     // Chat | Decision | Request | Response
    timestamp: DateTime<Utc>,
    metadata: HashMap<String, Value>,
}
```

---

## Implementation Phases

**CURRENT STATUS:** Phase 0-1 partially complete. Need to clarify architecture based on learnings.

### Phase 0: Foundation (Week 1) - PARTIALLY DONE ✅/🔄

**Goal:** Get Octopod installed and the basic architecture working

**Deliverables:**
- [x] Create project structure in `/home/ajdepew/Repos/octopod/`
- [x] **Backend Abstraction Layer** - Basic trait defined
  - [x] Define `ClawBackend` trait
  - [x] Implement `BackendFactory`
  - [🔄] Implement `IronclawBackend` (default) - needs update for WebSocket orchestration
- [🔄] **Coding Agent Abstraction Layer** - Stub exists
  - [x] Define `CodingAgent` trait
  - [ ] Implement `CodingAgentFactory`
  - [ ] Implement `OpencodeAgent` (default)
- [x] **Platform Abstraction Layer** - Basic implementation
  - [x] Define `Platform` trait
  - [x] Implement `OmarchyPlatform` (default)
  - [x] tmux session management
  - [x] hyprctl workspace switching
- [x] **Core Commands**
  - [x] Implement onboarding wizard (`octopod onboard`)
  - [x] Implement `octopod init` command
  - [x] Implement prerequisite checks with install guidance
- [x] **State Management** - NEEDS IMPLEMENTATION
  - [ ] SQLite database setup
  - [ ] Decision tracking tables
  - [ ] Message/conversation tables
  - [ ] Agent state tables
- [x] Create configuration file structures (all abstraction layers)
- [x] Set up build system (Cargo workspace)
- [x] Create basic TUI framework (ratatui)

**LEARNINGS:**
- Ironclaw can only run one instance (port 8080 conflict)
- Architecture needs to shift: Ironclaw = orchestrator, opencode = workers
- Need SQLite for state (decisions, conversations, agent assignments)
- Department TUIs are WebSocket clients, not separate ironclaw instances

**UPDATED Success Criteria:**
- User can run `octopod onboard` and complete setup ✅
- User can run `octopod init` to create a new company ✅
- CEO Dashboard TUI opens and shows departments ✅
- **NEW:** Ironclaw plugin/extension for orchestration
- **NEW:** SQLite state database with essential tables
- **NEW:** WebSocket communication between dashboard and ironclaw

### Phase 1: CEO Dashboard (Week 2) - PARTIALLY DONE ✅/🔄

**Goal:** Build the CEO dashboard and spawn system

**Deliverables:**
- [x] Implement CEO dashboard TUI - Basic version done
- [x] Department status overview - Shows status
- [x] Activity feed - Basic mock data
- [🔄] Decision tracking - UI exists, needs SQLite backend
- [🔄] Issue board view (GitHub/GitLab integration) - Not started
- [🔄] Implement `octopod spawn` commands - Basic tmux spawn works
- [x] Workspace spawning via hyprland - Working
- [x] Session persistence (tmux) - Working

**UPDATED Deliverables (based on architecture clarification):**
- [ ] Ironclaw WebSocket client connection
- [ ] Real-time department status from Ironclaw
- [ ] Decision creation and approval flow
- [ ] Chat message display in dashboard
- [ ] SQLite integration for decisions/tasks

**Success Criteria:**
- `octopod` opens CEO Dashboard with 3 tabs (Departments, Decisions, Activity) ✅
- Can see department status (Stopped/Starting/Running) ✅
- Can spawn departments (creates tmux session) ✅
- **NEW:** Decisions are stored in SQLite and persist
- **NEW:** Real-time updates via WebSocket to Ironclaw

### Phase 2: Cortex Agent (Week 3)

**Goal:** Build the knowledge management system

**Deliverables:**
- [ ] Cortex agent implementation
- [ ] Git integration for cortex
- [ ] Knowledge query system
- [ ] Decision logging
- [ ] SKILL.md parser
- [ ] `octopod cortex` commands

**Success Criteria:**
- Cortex agent can answer questions about company knowledge
- Changes to cortex are automatically committed to Git
- Agents can query cortex for context

### Phase 3: Core Departments (Week 4-5)

**Goal:** Implement Product, Engineering, and QA

**Deliverables:**
- [ ] Product department TUI
- [ ] Engineering department with opencode integration
- [ ] QA department with browser automation
- [ ] Inter-department communication
- [ ] Work item routing

**Success Criteria:**
- Product can create PRDs and file issues
- Engineering can implement features with visible progress
- QA can test and report bugs
- Work flows between departments

### Phase 4: Remaining Departments (Week 6)

**Goal:** Implement Design, DevOps, Marketing, Sales

**Deliverables:**
- [ ] Design department
- [ ] DevOps department
- [ ] Marketing department
- [ ] Sales department

**Success Criteria:**
- All 7 departments are functional
- Each has appropriate tools and integrations

### Phase 5: Polish & Integration (Week 7)

**Goal:** Integration, testing, and polish

**Deliverables:**
- [ ] Waybar integration (status widget)
- [ ] Complete documentation
- [ ] Error handling & recovery
- [ ] Performance optimization
- [ ] Testing & bug fixes

**Success Criteria:**
- Full end-to-end workflow works
- Stable under normal use
- Good error messages and recovery

### Phase 6: OpenClaw Backend (Future)

**Goal:** Add OpenClaw as an alternative backend

**Deliverables:**
- [ ] Implement `OpenclawBackend` trait
- [ ] Backend switching command (`octopod backend switch`)
- [ ] Feature parity testing
- [ ] Configuration migration tool
- [ ] Documentation for backend differences
- [ ] Community plugin support research

**Success Criteria:**
- User can switch between Ironclaw and OpenClaw backends
- Core functionality works with both backends
- Clear documentation of tradeoffs
- Migration path is smooth

**Decision Point:**
This phase should only be attempted after Ironclaw integration is stable and feature-complete. The abstraction layer ensures we can add this later without major refactoring.

---

## Technical Stack

### Core
- **Language:** Rust
- **TUI Framework:** ratatui + crossterm
- **Async Runtime:** tokio
- **Serialization:** serde + toml

### Omarchy Integration
- **Window Manager:** Hyprland (via hyprctl)
- **Terminal Multiplexer:** tmux
- **Status Bar:** Waybar (custom module)
- **Terminal:** Alacritty (configurable)

### AI/LLM
- **Coordinator:** ironclaw (persistent daemon)
- **Coding Agents:** opencode (spawned per task)
- **Dev Loop:** tdl (visible progress)
- **Provider:** OpenRouter (configurable)

### Git & Issues
- **VCS:** Git
- **GitHub:** gh CLI
- **GitLab:** glab CLI
- **Issue Tracking:** GitHub/GitLab Issues API

### Data Storage
- **Config:** TOML files
- **State:** JSON/ron files
- **Knowledge:** Git repository (cortex)
- **Agent Memory:** Ironclaw's built-in persistence

---

## Directory Structure

```
/home/ajdepew/Repos/octopod/
├── Cargo.toml                    # Workspace root
├── README.md
├── LICENSE
├── .plans/                       # Architecture & planning docs
│   └── architecture.md          # This document
├── src/
│   ├── main.rs                   # CLI entry point
│   ├── lib.rs                    # Library exports
│   ├── cli/                      # CLI commands
│   │   ├── mod.rs
│   │   ├── onboard.rs           # Onboarding wizard
│   │   ├── init.rs              # octopod init
│   │   ├── spawn.rs             # octopod spawn
│   │   ├── status.rs            # octopod status
│   │   ├── stop.rs              # octopod stop
│   │   ├── cortex.rs            # octopod cortex
│   │   ├── issues.rs            # octopod issues
│   │   ├── config.rs            # octopod config
│   │   └── doctor.rs            # octopod doctor
│   ├── core/                     # Core functionality
│   │   ├── mod.rs
│   │   ├── company.rs           # Company struct & ops
│   │   ├── department.rs        # Department management
│   │   ├── workspace.rs         # Hyprland workspace mgmt
│   │   ├── config.rs            # Configuration handling
│   │   └── error.rs             # Error types
│   ├── tui/                      # TUI components
│   │   ├── mod.rs
│   │   ├── app.rs               # Main TUI app
│   │   ├── dashboard.rs         # CEO dashboard
│   │   ├── components/          # Reusable widgets
│   │   │   ├── mod.rs
│   │   │   ├── header.rs
│   │   │   ├── sidebar.rs
│   │   │   ├── chat.rs
│   │   │   ├── status.rs
│   │   │   └── issues.rs
│   │   └── themes/              # Theme definitions
│   │       ├── mod.rs
│   │       └── octopus.rs       # Octopus theme
│   ├── departments/              # Department implementations
│   │   ├── mod.rs
│   │   ├── product.rs
│   │   ├── engineering.rs
│   │   ├── qa.rs
│   │   ├── design.rs
│   │   ├── devops.rs
│   │   ├── marketing.rs
│   │   ├── sales.rs
│   │   └── cortex.rs
│   ├── backends/                 # Pluggable claw backends
│   │   ├── mod.rs               # Backend trait & factory
│   │   ├── ironclaw.rs          # Ironclaw backend implementation
│   │   ├── ironclaw/
│   │   │   ├── coordinator.rs   # Ironclaw coordinator
│   │   │   ├── client.rs        # Ironclaw client
│   │   │   └── tools.rs         # Ironclaw-specific tools
│   │   ├── openclaw.rs          # OpenClaw backend implementation (Phase 6+)
│   │   └── types.rs             # Shared backend types
│   ├── coding/                   # Pluggable coding agents
│   │   ├── mod.rs               # CodingAgent trait & factory
│   │   ├── opencode.rs          # Opencode implementation (default)
│   │   ├── aider.rs             # Aider implementation (Phase 4+)
│   │   ├── claude_code.rs       # Claude Code implementation (future)
│   │   └── types.rs             # Shared coding types
│   ├── platform/                 # Pluggable platforms
│   │   ├── mod.rs               # Platform trait & factory
│   │   ├── omarchy.rs           # Omarchy/Hyprland implementation (default)
│   │   ├── omarchy/
│   │   │   ├── hyprland.rs      # Hyprland control
│   │   │   ├── tmux.rs          # Tmux session mgmt
│   │   │   └── waybar.rs        # Waybar integration
│   │   └── generic.rs           # Generic TUI implementation (future)
│   ├── git/                      # Git integration
│   │   ├── mod.rs
│   │   ├── forge.rs             # GitHub/GitLab abstraction
│   │   └── cortex.rs            # Cortex operations
│   └── state/                    # State management
│       ├── mod.rs
│       ├── persistence.rs       # Save/load agent state
│       └── migrations.rs        # State migrations
├── templates/                    # Project templates
│   ├── saas-standard/
│   ├── lean-startup/
│   └── enterprise/
└── docs/                         # Documentation
    ├── getting-started.md
    ├── departments.md
    ├── cortex.md
    └── api.md
```

---

## Success Metrics

**Phase 0:**
- Onboarding completes without errors
- `octopod init` creates valid configuration

**Phase 1:**
- CEO dashboard displays in Super+1
- Can spawn all 7 departments
- Departments persist across TUI restarts

**Phase 2:**
- Cortex answers queries accurately
- Git commits happen automatically
- SKILL.md files are parsed correctly

**Phase 3:**
- Product → Engineering → QA workflow functions
- Code changes are visible in Engineering TUI
- Bugs are filed and tracked

**Phase 4:**
- All departments are operational
- Agents can communicate
- CEO can monitor everything

**Phase 5:**
- End-to-end workflow: Idea → Implementation → Test → Deploy
- System is stable for daily use
- Documentation is complete

---

## Risk Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| Ironclaw/opencode API changes | High | Pin versions, wrap in abstraction layer |
| Hyprland breaking changes | Medium | Detect version, adapt commands |
| LLM rate limits | Medium | Cache responses, queue requests |
| Git authentication issues | Medium | Support multiple auth methods, good error messages |
| Workspace conflicts | Low | Check before spawning, provide recovery |

---

## Future Enhancements (Post-MVP)

- [ ] OpenClaw backend support (Phase 6 - see implementation phases)
- [ ] Mobile companion app (view-only)
- [ ] Voice commands for CEO
- [ ] Advanced analytics dashboard
- [ ] Multi-company support (switch between companies)
- [ ] Custom department templates
- [ ] Third-party integrations (Slack, Discord, Notion)
- [ ] AI-powered code review
- [ ] Automated deployment pipelines
- [ ] Performance benchmarking
- [ ] Team collaboration (multiple human CEOs)

---

**Document Version:** 1.2  
**Last Updated:** 2026-04-02  
**Status:** Ready for Implementation

## Changelog

**v1.2 (2026-04-02)**
- Added "Swappable Opinions" philosophy section
- Added Coding Agent Abstraction Layer (Opencode, Aider, Claude Code support)
- Added Platform Abstraction Layer (Omarchy vs Generic)
- Added comprehensive Prerequisites section with Omarchy installation instructions
- Updated Phase 0 to include all three abstraction layers (backend, coding, platform)
- Updated directory structure to include `src/coding/` and `src/platform/`
- Updated configuration examples with backend, coding, and platform sections
- Clarified opencode installation: `curl -fsSL https://opencode.ai/install | bash` or `paru -S opencode`

**v1.1 (2026-04-02)**
- Added Backend Abstraction Layer section with trait design
- Added Ironclaw vs OpenClaw comparison and rationale
- Updated Phase 0 to include backend abstraction implementation
- Added Phase 6 for OpenClaw backend support
- Updated directory structure to include `src/backends/`
- Renamed "Ironclaw Integration Architecture" to "Backend Implementation: Ironclaw (Default)"
- Added backend configuration options to config examples
