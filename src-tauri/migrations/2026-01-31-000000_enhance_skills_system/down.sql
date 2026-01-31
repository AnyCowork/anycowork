-- Remove skill_files table
DROP INDEX IF EXISTS idx_skill_files_skill_id;
DROP TABLE IF EXISTS skill_files;

-- Note: SQLite doesn't support DROP COLUMN directly in older versions
-- These columns will remain but can be ignored if migration is reverted
-- In production, a proper migration would recreate the table without these columns
