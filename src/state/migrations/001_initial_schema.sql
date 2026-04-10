-- Migration 001: Initial Schema
-- Creates the core tables for Octopod state management

-- Enums as check constraints (SQLite doesn't support enum types)
-- Department: product, engineering, qa, design, devops, marketing, sales, legal, finance
-- Status: pending, approved, rejected, escalated, cancelled
-- Priority: p0 (critical), p1 (high), p2 (medium), p3 (low)
-- MessageType: chat, decision_request, decision_response, command, notification
-- TaskStatus: todo, in_progress, blocked, review, done, cancelled
-- AgentStatus: idle, working, error, offline

-- Companies table (supports multiple projects)
CREATE TABLE IF NOT EXISTS companies (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Departments table
CREATE TABLE IF NOT EXISTS departments (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    name TEXT NOT NULL,
    slug TEXT NOT NULL, -- product, engineering, etc.
    description TEXT,
    workspace INTEGER NOT NULL, -- Super+N workspace number
    config_json TEXT, -- JSON blob for department-specific config
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE CASCADE,
    UNIQUE(company_id, slug)
);

-- Agents table (multi-agent departments)
CREATE TABLE IF NOT EXISTS agents (
    id TEXT PRIMARY KEY,
    department_id TEXT NOT NULL,
    name TEXT NOT NULL,
    role TEXT, -- e.g., "Frontend Engineer", "Product Manager"
    personality TEXT, -- Description of communication style
    model TEXT, -- LLM model used
    config_json TEXT, -- Agent-specific configuration
    status TEXT NOT NULL DEFAULT 'idle' CHECK (status IN ('idle', 'working', 'error', 'offline')),
    current_task_id TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_seen_at TIMESTAMP,
    
    FOREIGN KEY (department_id) REFERENCES departments(id) ON DELETE CASCADE
);

-- Decisions table (CEO decision tracking)
CREATE TABLE IF NOT EXISTS decisions (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    department_id TEXT, -- Which department requested it
    requested_by TEXT, -- Agent ID or "ceo"
    priority TEXT NOT NULL DEFAULT 'p2' CHECK (priority IN ('p0', 'p1', 'p2', 'p3')),
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'rejected', 'escalated', 'cancelled')),
    context_json TEXT, -- Additional context (links, references, etc.)
    
    -- Decision outcome
    approved_by TEXT, -- Agent ID or "ceo"
    decision_notes TEXT, -- Why the decision was made
    
    -- Timestamps
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    resolved_at TIMESTAMP,
    
    FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE CASCADE,
    FOREIGN KEY (department_id) REFERENCES departments(id) ON DELETE SET NULL,
    FOREIGN KEY (requested_by) REFERENCES agents(id) ON DELETE SET NULL,
    FOREIGN KEY (approved_by) REFERENCES agents(id) ON DELETE SET NULL
);

-- Conversations/Messages table (Slack-like chat)
CREATE TABLE IF NOT EXISTS conversations (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    department_id TEXT, -- NULL = company-wide
    title TEXT,
    conversation_type TEXT NOT NULL DEFAULT 'channel' CHECK (conversation_type IN ('channel', 'dm', 'thread')),
    parent_message_id TEXT, -- For threads
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    archived_at TIMESTAMP, -- Soft delete
    
    FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE CASCADE,
    FOREIGN KEY (department_id) REFERENCES departments(id) ON DELETE SET NULL,
    FOREIGN KEY (parent_message_id) REFERENCES messages(id) ON DELETE CASCADE
);

-- Messages table (chat history)
CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    from_agent_id TEXT, -- NULL = system message or CEO
    to_agent_id TEXT, -- NULL = broadcast to conversation
    content TEXT NOT NULL,
    message_type TEXT NOT NULL DEFAULT 'chat' CHECK (message_type IN ('chat', 'decision_request', 'decision_response', 'command', 'notification', 'file', 'error')),
    metadata_json TEXT, -- Flexible metadata (file attachments, reactions, etc.)
    
    -- Soft delete for audit trail
    deleted_at TIMESTAMP,
    deleted_by TEXT,
    
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE,
    FOREIGN KEY (from_agent_id) REFERENCES agents(id) ON DELETE SET NULL,
    FOREIGN KEY (to_agent_id) REFERENCES agents(id) ON DELETE SET NULL
);

-- Tasks table (work items)
CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    department_id TEXT NOT NULL,
    assigned_to TEXT, -- Agent ID
    created_by TEXT, -- Agent ID or "ceo"
    
    -- Task details
    title TEXT NOT NULL,
    description TEXT,
    task_type TEXT NOT NULL DEFAULT 'task' CHECK (task_type IN ('feature', 'bug', 'task', 'research', 'documentation')),
    status TEXT NOT NULL DEFAULT 'todo' CHECK (status IN ('todo', 'in_progress', 'blocked', 'review', 'done', 'cancelled')),
    priority TEXT NOT NULL DEFAULT 'p2' CHECK (priority IN ('p0', 'p1', 'p2', 'p3')),
    
    -- Relationships
    parent_task_id TEXT, -- For subtasks
    related_decision_id TEXT, -- Decision that spawned this task
    github_issue_number INTEGER, -- Linked GitHub issue
    
    -- Estimation and tracking
    estimated_hours INTEGER,
    actual_hours INTEGER,
    
    -- Timestamps
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    deadline_at TIMESTAMP,
    
    -- Soft delete
    deleted_at TIMESTAMP,
    
    FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE CASCADE,
    FOREIGN KEY (department_id) REFERENCES departments(id) ON DELETE CASCADE,
    FOREIGN KEY (assigned_to) REFERENCES agents(id) ON DELETE SET NULL,
    FOREIGN KEY (created_by) REFERENCES agents(id) ON DELETE SET NULL,
    FOREIGN KEY (parent_task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    FOREIGN KEY (related_decision_id) REFERENCES decisions(id) ON DELETE SET NULL
);

-- Activity log (audit trail)
CREATE TABLE IF NOT EXISTS activity_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    company_id TEXT NOT NULL,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    actor TEXT NOT NULL, -- Agent ID or "ceo" or "system"
    actor_type TEXT NOT NULL DEFAULT 'agent' CHECK (actor_type IN ('agent', 'ceo', 'system')),
    action TEXT NOT NULL, -- spawn, kill, approve, reject, message, task_create, etc.
    target_type TEXT, -- decision, agent, task, department, etc.
    target_id TEXT, -- ID of the target
    details_json TEXT, -- Additional context
    
    FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE CASCADE
);

-- Triggers for updated_at timestamps
CREATE TRIGGER IF NOT EXISTS departments_updated_at 
    AFTER UPDATE ON departments
    FOR EACH ROW
    BEGIN
        UPDATE departments SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;

CREATE TRIGGER IF NOT EXISTS agents_updated_at 
    AFTER UPDATE ON agents
    FOR EACH ROW
    BEGIN
        UPDATE agents SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;

CREATE TRIGGER IF NOT EXISTS decisions_updated_at 
    AFTER UPDATE ON decisions
    FOR EACH ROW
    BEGIN
        UPDATE decisions SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;

CREATE TRIGGER IF NOT EXISTS tasks_updated_at 
    AFTER UPDATE ON tasks
    FOR EACH ROW
    BEGIN
        UPDATE tasks SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;