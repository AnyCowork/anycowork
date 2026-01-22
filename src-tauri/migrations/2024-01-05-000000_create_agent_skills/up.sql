CREATE TABLE agent_skills (
  id TEXT NOT NULL PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  display_title TEXT NOT NULL,
  description TEXT NOT NULL,
  skill_content TEXT NOT NULL,
  additional_files_json TEXT,
  enabled INTEGER NOT NULL DEFAULT 1,
  version INTEGER NOT NULL DEFAULT 1,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_agent_skills_enabled ON agent_skills(enabled);
CREATE INDEX idx_agent_skills_name ON agent_skills(name);

CREATE TABLE agent_skill_assignments (
  agent_id TEXT NOT NULL,
  skill_id TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (agent_id, skill_id),
  FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE,
  FOREIGN KEY (skill_id) REFERENCES agent_skills(id) ON DELETE CASCADE
);
