-- Migration 013: Add file_path to decisions for markdown file support
ALTER TABLE decisions ADD COLUMN file_path TEXT;
