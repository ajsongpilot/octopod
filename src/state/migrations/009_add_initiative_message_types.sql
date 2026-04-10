-- Add new message types for initiative and decision workflows
-- This updates the CHECK constraint on message_type

-- First, recreate the messages table with updated constraints
-- SQLite doesn't support ALTER TABLE to change CHECK constraints, so we recreate

-- Create new messages table with expanded message types
CREATE TABLE IF NOT EXISTS messages_new (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    from_agent_id TEXT,
    to_agent_id TEXT,
    content TEXT NOT NULL,
    message_type TEXT NOT NULL DEFAULT 'chat' CHECK (message_type IN (
        'chat', 
        'decision_request', 
        'decision_response',
        'decision_proposal',
        'initiative_update',
        'task_assignment',
        'meeting_request',
        'command', 
        'notification', 
        'file', 
        'error'
    )),
    metadata_json TEXT,
    deleted_at TIMESTAMP,
    deleted_by TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE,
    FOREIGN KEY (from_agent_id) REFERENCES agents(id) ON DELETE SET NULL,
    FOREIGN KEY (to_agent_id) REFERENCES agents(id) ON DELETE SET NULL
);

-- Copy data from old table
INSERT INTO messages_new SELECT * FROM messages;

-- Drop old table
DROP TABLE messages;

-- Rename new table to original name
ALTER TABLE messages_new RENAME TO messages;

-- Recreate indexes
CREATE INDEX IF NOT EXISTS idx_messages_conversation_id ON messages(conversation_id);
CREATE INDEX IF NOT EXISTS idx_messages_from_agent_id ON messages(from_agent_id);
CREATE INDEX IF NOT EXISTS idx_messages_to_agent_id ON messages(to_agent_id);
CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages(created_at);
CREATE INDEX IF NOT EXISTS idx_messages_conversation_created ON messages(conversation_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_messages_conversation_time ON messages(conversation_id, created_at DESC);
