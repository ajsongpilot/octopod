-- Migration 015: Fix roadmaps status_id to be TEXT (not INTEGER)
-- Status values: draft, planning, active, completed, archived

-- Create new table with TEXT status_id
CREATE TABLE roadmaps_new (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    period_start TEXT NOT NULL,
    period_end TEXT NOT NULL,
    status_id TEXT DEFAULT 'draft' CHECK (status_id IN ('draft', 'planning', 'active', 'completed', 'archived')),
    goals_json TEXT,
    created_by TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Copy data - convert integers to text if needed
INSERT INTO roadmaps_new 
SELECT 
    id,
    company_id,
    name,
    description,
    period_start,
    period_end,
    CASE 
        WHEN CAST(status_id AS TEXT) IN ('1', 'draft') THEN 'draft'
        WHEN CAST(status_id AS TEXT) IN ('2', 'planning') THEN 'planning'
        WHEN CAST(status_id AS TEXT) IN ('3', 'active') THEN 'active'
        WHEN CAST(status_id AS TEXT) IN ('4', 'completed') THEN 'completed'
        WHEN CAST(status_id AS TEXT) IN ('5', 'archived') THEN 'archived'
        ELSE 'draft'
    END as status_id,
    goals_json,
    created_by,
    created_at,
    updated_at
FROM roadmaps;

-- Drop old table and rename
DROP TABLE roadmaps;
ALTER TABLE roadmaps_new RENAME TO roadmaps;

-- Recreate indexes
DROP INDEX IF EXISTS idx_roadmaps_status;
DROP INDEX IF EXISTS idx_roadmaps_company;

CREATE INDEX idx_roadmaps_status ON roadmaps(status_id);
CREATE INDEX idx_roadmaps_company ON roadmaps(company_id);
