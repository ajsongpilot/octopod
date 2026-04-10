# Octopod Multi-Agent Orchestration Design

## Core Philosophy
We build the orchestration (message bus, task routing, state management) in Octopod.
Ironclaw becomes the "smart coordinator" - an AI agent with special tools to orchestrate others.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    OCTOPOD ORCHESTRATOR                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │  Message    │  │   Task      │  │    State Manager        │  │
│  │   Bus       │  │   Queue     │  │    (SQLite)             │  │
│  │             │  │             │  │                         │  │
│  │ • Routing   │  │ • Priority  │  │ • Agent states          │  │
│  │ • Broadcast │  │ • Blocking  │  │ • Task assignments      │  │
│  │ • DM        │  │ • Handoffs  │  │ • Conversation history  │  │
│  └──────┬──────┘  └──────┬──────┘  └───────────┬─────────────┘  │
│         │                │                      │                │
│         └────────────────┴──────────────────────┘                │
│                          │                                       │
│                   HTTP API (for Ironclaw)                        │
└──────────────────────────┬──────────────────────────────────────┘
                           │
    ┌──────────────────────┼──────────────────────┐
    │                      │                      │
┌───▼────┐           ┌────▼────┐           ┌────▼────┐
│Product │           │Engineer │           │Ironclaw │
│Agent   │           │  Agent  │           │Coord.  │
│(opencode)│         │(opencode)│          │(special)│
└───┬────┘           └────┬────┘           └────┬────┘
    │                     │                      │
    └─────────────────────┴──────────────────────┘
                          │
                   ┌──────┴──────┐
                   │   Cortex    │
                   │  (Git/docs) │
                   └─────────────┘
```

## Components

### 1. Message Bus (SQLite + WebSocket)
- Departments can send messages to each other
- Broadcast or DM
- Real-time updates to CEO Dashboard

### 2. Task Queue
- Priority-based (P0, P1, P2, P3)
- Blocked/waiting status
- Automatic handoffs

### 3. Ironclaw Integration
- Custom WASM tool: `octopod` 
- Gives Ironclaw access to orchestration
- Ironclaw can: query status, route tasks, make decisions

### 4. Department Agents
- Opencode with custom identities
- Can: read tasks, send messages, request decisions
- Auto-report progress

## Implementation Phases

### Phase 1: Message Bus (Foundation)
- [ ] Add `messages` table with routing
- [ ] WebSocket server for real-time updates
- [ ] Department agents can send/receive messages
- [ ] CEO Dashboard shows live chat

### Phase 2: Task Queue
- [ ] Task creation and assignment
- [ ] Priority handling
- [ ] Blocked/waiting states
- [ ] Automatic handoffs

### Phase 3: Ironclaw Integration
- [ ] Build WASM tool for Ironclaw
- [ ] Ironclaw can query orchestrator
- [ ] Ironclaw can assign tasks
- [ ] Ironclaw can make recommendations

### Phase 4: Self-Healing Cortex
- [ ] Agents auto-update cortex
- [ ] Ironclaw reviews and refines
- [ ] SKILL.md auto-generation

## Key Design Decisions

### Why not build WASM channel?
- Complex, time-consuming
- HTTP API is simpler for MVP
- Can migrate to WASM later

### Why SQLite for message bus?
- Persistent (survives crashes)
- Queryable history
- Easy to debug
- Can add WebSocket for real-time

### Ironclaw's Role
Not the orchestrator, but the "smart coordinator":
- Has tools to query all departments
- Can suggest task assignments
- Makes recommendations, not decisions
- CEO (you) approve/override

## Success Metrics

1. **Zero manual coordination**: Departments talk to each other
2. **Automatic task routing**: PRD → Implementation → Testing
3. **Context-aware decisions**: Ironclaw knows full state before recommending
4. **Self-documenting**: Cortex updates automatically

## Next Steps

Start with Phase 1: Message Bus
- Simple, high impact
- Enables all other features
- Proves the architecture works

Ready to build?