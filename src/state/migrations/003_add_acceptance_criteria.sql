-- Migration 003: Add acceptance_criteria to tasks
-- PM-style acceptance criteria for stories/tasks

ALTER TABLE tasks ADD COLUMN acceptance_criteria TEXT;
