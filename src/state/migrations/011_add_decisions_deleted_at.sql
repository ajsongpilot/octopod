-- Add deleted_at column to decisions table for soft deletes
ALTER TABLE decisions ADD COLUMN deleted_at TIMESTAMP;
