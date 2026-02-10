CREATE TABLE tasks (
  id TEXT NOT NULL PRIMARY KEY,
  title TEXT NOT NULL,
  description TEXT,
  status TEXT NOT NULL DEFAULT 'pending', -- pending, in_progress, completed, failed
  priority INTEGER NOT NULL DEFAULT 0, -- 0: Low, 1: Medium, 2: High
  session_id TEXT,
  agent_id TEXT,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE SET NULL,
  FOREIGN KEY(agent_id) REFERENCES agents(id) ON DELETE SET NULL
);
