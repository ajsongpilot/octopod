-- Migration 008: Update initiative status workflow
-- This replaces the old status ( RoadmapStatus ) with InitiativeStatus

-- Initiative status enum
CREATE TABLE IF NOT EXISTS initiative_status (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);
INSERT OR IGNORE INTO initiative_status (id, name) VALUES 
    (1, 'draft'),
    (2, 'proposed'),
    (3, 'stakeholder_review'),
    (4, 'approved'),
    (5, 'active'),
    (6, 'completed'),
    (7, 'cancelled'),
    (8, 'archived');

-- Add severity column to initiatives
ALTER TABLE initiatives ADD COLUMN severity TEXT DEFAULT 'medium';

-- Add pending_decision_id column to link to CEO decisions
ALTER TABLE initiatives ADD COLUMN pending_decision_id TEXT;

-- Update existing initiatives to new status values
-- draft=1, planning=2, active=3, completed=4, archived=5
-- Initiative: draft=1, proposed=2, stakeholder_review=3, approved=4, active=5, completed=6, cancelled=7, archived=8
UPDATE initiatives SET status_id = 1 WHERE status_id = 1; -- draft -> draft
UPDATE initiatives SET status_id = 5 WHERE status_id = 2; -- planning -> active (close enough)
UPDATE initiatives SET status_id = 5 WHERE status_id = 3; -- active -> active
UPDATE initiatives SET status_id = 6 WHERE status_id = 4; -- completed -> completed
UPDATE initiatives SET status_id = 8 WHERE status_id = 5; -- archived -> archived

-- Create index for querying initiatives by status
CREATE INDEX IF NOT EXISTS idx_initiatives_status ON initiatives(status_id);

-- Create index for pending CEO decisions
CREATE INDEX IF NOT EXISTS idx_initiatives_pending_decision ON initiatives(pending_decision_id);
