# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-04-08

### Added

#### CEO Dashboard
- Interactive TUI dashboard with three view modes (Pending, CEO Queue, Log)
- Decision queue showing HIGH severity decisions awaiting approval
- Severity column with visual indicators (🔴 HIGH 🟡 MED ⚪ LOW)
- Approve/Reject actions for decisions
- Initiative planning view

#### Initiative Workflow
- `InitiativeStatus` state machine: Draft → Proposed → StakeholderReview → Approved → Active → Completed
- Severity field on initiatives (LOW/MEDIUM/HIGH)
- Automatic CEO decision creation for HIGH severity initiatives
- `pending_decision_id` linking initiatives to approval decisions
- `transition_initiative_to_proposed()`, `approve_initiative_decision()`, `reject_initiative_decision()` methods

#### Agent CLI
- `octopod decide propose` - Create initiative with severity
- `octopod decide decide` - Log a decision
- `octopod decide review` - Request stakeholder review
- `octopod decide submit` - Transition initiative to proposed
- `octopod decide start` - Start an initiative
- `octopod decide complete` - Complete an initiative

#### Background Agent Scheduling
- `AgentRunner` for cron-like agent execution
- `AgentSchedule` with configurable intervals per department
- `run_scheduled()` - Background scheduler loop
- `run_once()` - Run agent immediately
- Enable/disable schedules without removing them
- `octopod agent schedule <department> [interval]` - Schedule periodic runs
- `octopod agent list` - Show all schedules
- `octopod agent enable/disable` - Toggle schedules

#### Message Bus Enhancements
- New message types: `DecisionProposal`, `InitiativeUpdate`, `TaskAssignment`, `MeetingRequest`
- Metadata structs for structured message content
- `send_decision_proposal()`, `send_initiative_update()`, `send_task_assignment()`, `send_meeting_request()`
- `broadcast_initiative_update()` - Broadcast to all stakeholders
- `notify_decision_for_approval()` - Notify CEO queue
- Automatic broadcasts on initiative status changes
- Migration 009: `add_initiative_message_types`

#### Database Migrations
- 006: `add_roadmaps_meetings` - Roadmaps and meetings support
- 007: `add_decision_severity` - Decision severity field
- 008: `update_initiative_status` - Initiative status state machine
- 009: `add_initiative_message_types` - New message types

### Changed

- Refactored agent command structure to use subcommands
- `octopod agent run <department>` replaces old single-arg agent command
- Message bus now auto-broadcasts initiative changes

### Fixed

- Agent loop now properly picks up unassigned tasks
- `loop` reserved keyword issue (module renamed to `agent_loop`)

### Architecture

```
src/
├── main.rs              # CLI entry point
├── lib.rs               # Library root
├── cli/                 # CLI commands
│   ├── mod.rs
│   ├── decide.rs        # Initiative/decision commands
│   ├── task.rs          # Task management
│   └── agent.rs         # Agent scheduling
├── state/               # Database layer
│   ├── mod.rs
│   ├── manager.rs       # State coordination
│   ├── message_bus.rs   # Inter-component messaging
│   ├── entities/        # Data types
│   ├── repositories/    # DB access
│   └── migrations/      # Schema changes
├── agent/               # Agent layer
│   ├── mod.rs
│   ├── agent_loop.rs    # Main loop
│   └── runner.rs        # Scheduling
└── tui/                 # Terminal UIs
    ├── ceo_dashboard.rs
    └── department.rs
```

[0.1.0]: https://github.com/octopod/octopod/releases/tag/v0.1.0
