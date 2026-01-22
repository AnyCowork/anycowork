-- Remove indexes
DROP INDEX IF EXISTS idx_sessions_archived;
DROP INDEX IF EXISTS idx_sessions_pinned;

-- Note: SQLite doesn't support DROP COLUMN directly
-- In production, you'd need to recreate the table without these columns
-- For now, we'll leave them (they'll just be ignored)
