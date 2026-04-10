-- Migration 005: Backfill task files for existing tasks
-- This migration is tracked but the actual file backfill is done in Rust
-- after migrations run to ensure files are created with proper content

-- No-op migration - actual work happens in StateManager::backfill_task_files()
