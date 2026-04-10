-- Add file_path column to initiatives for markdown file support
ALTER TABLE initiatives ADD COLUMN file_path TEXT;
