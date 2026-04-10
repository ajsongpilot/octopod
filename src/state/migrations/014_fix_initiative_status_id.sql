-- Migration 014: Fix initiatives status_id to be INTEGER
-- This handles both TEXT (legacy) and INTEGER (correct) stored values

-- Create new table with INTEGER status_id
CREATE TABLE initiatives_new (
    id TEXT PRIMARY KEY,
    roadmap_id TEXT NOT NULL,
    department_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    status_id INTEGER DEFAULT 1,
    priority TEXT DEFAULT 'p2',
    severity TEXT DEFAULT 'medium',
    stakeholder_depts_json TEXT,
    pending_decision_id TEXT,
    created_by TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    file_path TEXT
);

-- Copy data with proper type conversion
-- Uses NULLIF and CAST to safely handle both TEXT and INTEGER sources
INSERT INTO initiatives_new 
SELECT 
    id,
    roadmap_id,
    department_id,
    title,
    description,
    CAST(NULLIF(status_id, '') AS INTEGER) as status_id,
    priority,
    severity,
    stakeholder_depts_json,
    pending_decision_id,
    created_by,
    created_at,
    updated_at,
    file_path
FROM initiatives
WHERE status_id IS NOT NULL AND status_id != '';

-- If the above fails (e.g., non-numeric TEXT values), try with CASE
-- This handles legacy data with text status names
INSERT OR IGNORE INTO initiatives_new 
SELECT 
    id,
    roadmap_id,
    department_id,
    title,
    description,
    CASE 
        WHEN status_id = '1' OR status_id = 'draft' THEN 1
        WHEN status_id = '2' OR status_id = 'proposed' THEN 2
        WHEN status_id = '3' OR status_id = 'stakeholder_review' THEN 3
        WHEN status_id = '4' OR status_id = 'approved' THEN 4
        WHEN status_id = '5' OR status_id = 'active' THEN 5
        WHEN status_id = '6' OR status_id = 'completed' THEN 6
        WHEN status_id = '7' OR status_id = 'cancelled' THEN 7
        WHEN status_id = '8' OR status_id = 'archived' THEN 8
        ELSE 1
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
