-- Migration 007: Add severity to decisions and initiative linkage

-- Add severity column to decisions
ALTER TABLE decisions ADD COLUMN severity TEXT DEFAULT 'medium';

-- Add initiative_id column to decisions
ALTER TABLE decisions ADD COLUMN initiative_id TEXT;

-- Create index for querying pending high-severity decisions (CEO queue)
CREATE INDEX IF NOT EXISTS idx_decisions_severity_status ON decisions(severity, status);

-- Create index for decisions by initiative
CREATE INDEX IF NOT EXISTS idx_decisions_initiative ON decisions(initiative_id);
