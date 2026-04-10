-- Migration 006: Add roadmap, initiative, and meeting entities

-- Roadmap status enum
CREATE TABLE IF NOT EXISTS roadmap_status (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);
INSERT OR IGNORE INTO roadmap_status (id, name) VALUES (1, 'draft'), (2, 'planning'), (3, 'active'), (4, 'completed'), (5, 'archived');

-- Roadmap entity - planning period
CREATE TABLE IF NOT EXISTS roadmaps (
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
    updated_at TEXT NOT NULL,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

-- Initiative entity - groupings of tasks under a roadmap
CREATE TABLE IF NOT EXISTS initiatives (
    id TEXT PRIMARY KEY,
    roadmap_id TEXT NOT NULL,
    department_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    status_id INTEGER DEFAULT 1,
    priority TEXT DEFAULT 'p2',
    stakeholder_depts_json TEXT,
    created_by TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (roadmap_id) REFERENCES roadmaps(id),
    FOREIGN KEY (department_id) REFERENCES departments(id),
    FOREIGN KEY (status_id) REFERENCES roadmap_status(id)
);

-- Meeting type enum
CREATE TABLE IF NOT EXISTS meeting_types (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);
INSERT OR IGNORE INTO meeting_types (id, name) VALUES (1, 'planning'), (2, 'refinement'), (3, 'stakeholder_review'), (4, 'sprint_retro'), (5, 'one_on_one'), (6, 'all_hands'), (7, 'ad_hoc');

-- Meeting status enum  
CREATE TABLE IF NOT EXISTS meeting_statuses (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);
INSERT OR IGNORE INTO meeting_statuses (id, name) VALUES (1, 'scheduled'), (2, 'in_progress'), (3, 'completed'), (4, 'cancelled');

-- Meeting entity
CREATE TABLE IF NOT EXISTS meetings (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    title TEXT NOT NULL,
    meeting_type_id INTEGER NOT NULL,
    status_id INTEGER DEFAULT 1,
    scheduled_at TEXT NOT NULL,
    duration_minutes INTEGER DEFAULT 60,
    agenda_json TEXT,
    notes TEXT,
    outcomes_json TEXT,
    related_initiative_id TEXT,
    related_roadmap_id TEXT,
    created_by TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (company_id) REFERENCES companies(id),
    FOREIGN KEY (meeting_type_id) REFERENCES meeting_types(id),
    FOREIGN KEY (status_id) REFERENCES meeting_statuses(id),
    FOREIGN KEY (related_initiative_id) REFERENCES initiatives(id),
    FOREIGN KEY (related_roadmap_id) REFERENCES roadmaps(id)
);

-- Meeting participant entity
CREATE TABLE IF NOT EXISTS meeting_participants (
    id TEXT PRIMARY KEY,
    meeting_id TEXT NOT NULL,
    department_id TEXT NOT NULL,
    agent_id TEXT,
    role TEXT NOT NULL,
    status TEXT DEFAULT 'invited',
    response_message TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (meeting_id) REFERENCES meetings(id),
    FOREIGN KEY (department_id) REFERENCES departments(id),
    FOREIGN KEY (agent_id) REFERENCES agents(id)
);

-- Link tasks to initiatives
ALTER TABLE tasks ADD COLUMN initiative_id TEXT;
