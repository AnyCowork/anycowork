-- Add enhanced fields to agent_skills table
ALTER TABLE agent_skills ADD COLUMN source_path TEXT;
ALTER TABLE agent_skills ADD COLUMN category TEXT DEFAULT 'General';
ALTER TABLE agent_skills ADD COLUMN requires_sandbox INTEGER DEFAULT 0;
ALTER TABLE agent_skills ADD COLUMN sandbox_config TEXT;

-- Add scope fields to agents table
ALTER TABLE agents ADD COLUMN scope_type TEXT DEFAULT 'global';
ALTER TABLE agents ADD COLUMN workspace_path TEXT;

-- Create skill_files table for storing bundled files
CREATE TABLE skill_files (
    id TEXT NOT NULL PRIMARY KEY,
    skill_id TEXT NOT NULL,
    relative_path TEXT NOT NULL,
    content TEXT NOT NULL,
    file_type TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (skill_id) REFERENCES agent_skills(id) ON DELETE CASCADE,
    UNIQUE(skill_id, relative_path)
);

CREATE INDEX idx_skill_files_skill_id ON skill_files(skill_id);
