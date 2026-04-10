-- Migration 004: Add file_path to tasks for markdown storage

ALTER TABLE tasks ADD COLUMN file_path TEXT;
