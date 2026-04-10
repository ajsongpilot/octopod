# Contributing to Octopod

Thank you for your interest in contributing! This document provides guidelines and instructions for contributing to Octopod.

## Quick Links

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Pull Request Process](#pull-request-process)
- [Style Guide](#style-guide)
- [Testing](#testing)
- [Commit Messages](#commit-messages)

## Code of Conduct

Please read and follow our [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md). We expect all contributors to uphold these standards.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/YOUR_HANDLE/octopod.git
   cd octopod
   ```
3. **Add the upstream remote**:
   ```bash
   git remote add upstream https://github.com/octopod/octopod.git
   ```
4. **Create a branch** for your changes:
   ```bash
   git checkout -b feature/your-feature-name
   ```

## Development Setup

### Prerequisites

- Rust 1.75+ (install via [rustup](https://rustup.rs/))
- tmux
- git
- SQLite development libraries (usually pre-installed)

### Build

```bash
# Build the project
cargo build

# Run in development mode
cargo run

# Run all tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

### Useful Commands

```bash
# Check formatting
cargo fmt --check

# Auto-fix formatting
cargo fmt

# Run clippy lints
cargo clippy -- -D warnings

# Update dependencies
cargo update

# Build release version
cargo build --release
```

## Making Changes

### Finding Issues

Look for issues labeled with:
- [`good first issue`](https://github.com/octopod/octopod/labels/good%20first%20issue) — Entry points for new contributors
- [`help wanted`](https://github.com/octopod/octopod/labels/help%20wanted) — Issues that need contributions
- [`enhancement`](https://github.com/octopod/octopod/labels/enhancement) — Feature requests
- [`bug`](https://github.com/octopod/octopod/labels/bug) — Bug fixes

### Types of Contributions

- **Bug fixes** — Fix something that doesn't work correctly
- **Features** — Add new functionality
- **Documentation** — Improve docs, comments, or examples
- **Tests** — Add or improve test coverage
- **Refactoring** — Improve code structure without behavior change
- **Performance** — Make code faster or more efficient

## Pull Request Process

### 1. Before Starting

For significant changes, please open an issue first to discuss the direction. This prevents duplicate work and ensures your contribution aligns with project goals.

### 2. Keep Changes Focused

- One feature or fix per PR
- If your PR contains multiple unrelated changes, consider splitting it
- Reference relevant issues in your PR description

### 3. Update Documentation

- Update README.md if adding user-facing features
- Add doc comments to new public functions/types
- Update this CONTRIBUTING.md if changing contribution guidelines

### 4. Run Checks

Before submitting, ensure:

```bash
cargo test
cargo fmt
cargo clippy -- -D warnings
cargo build --release
```

### 5. Write a Good PR Description

```markdown
## Summary
Brief description of what this PR does

## Changes
- Change 1
- Change 2

## Motivation
Why is this change needed? What problem does it solve?

## Testing
How was this tested?

## Screenshots (if applicable)
Include before/after or UI changes
```

### 6. Submit

1. Push your branch: `git push origin feature/your-feature-name`
2. Open a Pull Request against `main`
3. Fill out the PR template
4. Wait for review — we typically respond within 2-3 days

### 7. Addressing Feedback

- Be responsive to review comments
- Don't take feedback personally — code review is about the code
- Ask for clarification if feedback is unclear
- Make requested changes in new commits (preserve history)

## Style Guide

### Rust Code

- Use `cargo fmt` for formatting
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use meaningful names — avoid abbreviations except well-known ones (ID, DB, etc.)
- Document public APIs with doc comments
- Prefer Result for error handling; use anyhow for application errors

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): short description

Longer explanation if needed. Wrap at 72 characters.

Footer with issue reference (optional):
Fixes #123
Closes #456
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

Examples:
```
feat(initiative): add severity levels to initiative workflow
fix(agent): prevent double-picking of tasks
docs(readme): update installation instructions
refactor(state): extract decision creation into separate method
```

### File Structure

```
src/
├── main.rs          # CLI entry point, argument parsing
├── lib.rs           # Library root, module definitions
├── cli/             # CLI commands
├── state/           # Database, entities, repositories
├── agent/           # Agent loop and scheduling
├── tui/             # Terminal user interfaces
├── backends/        # Backend abstractions
└── platform/        # Platform-specific code (tmux, hyprland)
```

## Testing

### Unit Tests

```bash
# Run all tests
cargo test

# Run tests for specific module
cargo test module_name

# Run tests with output
RUST_LOG=debug cargo test -- --nocapture
```

### Integration Tests

Place in `tests/` directory:
```rust
#[tokio::test]
async fn my_integration_test() {
    // Test implementation
}
```

### Database Tests

Tests use temporary databases:
```rust
use tempfile::tempdir;

#[tokio::test]
async fn test_with_temp_db() {
    let dir = tempdir().unwrap();
    let state = StateManager::init_for_project(&dir.path()).await.unwrap();
    // test code
}
```

## Project Structure

### State Layer (`src/state/`)

- `entities/` — Data types (Task, Decision, Initiative, etc.)
- `repositories/` — Database access patterns
- `manager.rs` — High-level state coordination
- `message_bus.rs` — Inter-component messaging
- `migrations/` — Database schema changes

### Agent Layer (`src/agent/`)

- `agent_loop.rs` — Main agent decision loop
- `runner.rs` — Background scheduling
- `mod.rs` — Agent backend abstractions

### CLI Layer (`src/cli/`)

Each command has its own file:
- `decide.rs` — Initiative/decision commands
- `task.rs` — Task management
- `agent.rs` — Agent scheduling

### TUI Layer (`src/tui/`)

- `ceo_dashboard.rs` — CEO dashboard interface
- `department.rs` — Department TUI

## Questions?

- Open an issue for bugs or feature requests
- Join our [Discord](https://discord.gg/octopod) for real-time discussion
- Tag maintainers for review

## Recognition

Contributors will be recognized in our release notes and on the project page.

---

*Last updated: 2026-04-08*
