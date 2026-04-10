-- Migration 016: Convert initiatives status_id to TEXT
-- The InitiativeStatus enum now uses snake_case strings

-- Create new table with TEXT status_id
CREATE TABLE initiatives_new (
    id TEXT PRIMARY KEY,
    roadmap_id TEXT NOT NULL,
    department_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    status_id TEXT DEFAULT 'draft' CHECK (status_id IN ('draft', 'proposed', 'stakeholder_review', 'approved', 'active', 'completed', 'cancelled', 'closed', 'archived')),
    priority TEXT DEFAULT 'p2',
    severity TEXT DEFAULT 'medium',
    stakeholder_depts_json TEXT,
    pending_decision_id TEXT,
    created_by TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    file_path TEXT
);

-- Copy data with proper type conversion from INTEGER to TEXT
INSERT INTO initiatives_new 
SELECT 
    id,
    roadmap_id,
    department_id,
    title,
    description,
    CASE CAST(status_id AS INTEGER)
        WHEN 1 THEN 'draft'
        WHEN 2 THEN 'proposed'
        WHEN 3 THEN 'stakeholder_review'
        WHEN 4 THEN 'approved'
        WHEN 5 THEN 'active'
        WHEN 6 THEN 'completed'
        WHEN 7 THEN 'cancelled'
        WHEN 8 THEN 'closed'
        WHEN 9 THEN 'archived'
        ELSE 'draft'
    END as status_id,
    priority,
    severity,
    stakeholder_depts_json,
    pending_decision_id,
    created_by,
    created_at,
    updated_at,
    file_path
FROM initiatives;

-- Drop old table and rename
DROP TABLE initiatives;
ALTER TABLE initiatives_new RENAME TO initiatives;

-- Recreate indexes
DROP INDEX IF EXISTS idx_initiatives_status;
DROP INDEX IF EXISTS idx_initiatives_roadmap;
DROP INDEX IF EXISTS idx_initiatives_pending_decision;

CREATE INDEX idx_initiatives_status ON initiatives(status_id);
CREATE INDEX idx_initiatives_roadmap ON initiatives(roadmap_id);
CREATE INDEX idx_initiatives_pending_decision ON initiatives(pending_decision_id);
