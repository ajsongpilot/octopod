-- Migration 010: Add agent_sessions table
-- Tracks opencode sessions created by octopod agents

CREATE TABLE IF NOT EXISTS agent_sessions (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    department_id TEXT NOT NULL,
    session_id TEXT NOT NULL,        -- opencode's session ID
    process_id INTEGER,             -- PID of the opencode process
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'completed', 'stopped')),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    ended_at TIMESTAMP,
    
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    FOREIGN KEY (department_id) REFERENCES departments(id) ON DELETE CASCADE
);

CREATE INDEX idx_agent_sessions_task ON agent_sessions(task_id);
CREATE INDEX idx_agent_sessions_session_id ON agent_sessions(session_id);
CREATE INDEX idx_agent_sessions_status ON agent_sessions(status);
