CREATE TABLE IF NOT EXISTS agents (
  id TEXT NOT NULL PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT,
  status TEXT NOT NULL DEFAULT 'active',
  personality TEXT,
  tone TEXT,
  expertise TEXT,
  ai_provider TEXT NOT NULL DEFAULT 'openai',
  ai_model TEXT NOT NULL DEFAULT 'gpt-4o',
  ai_temperature FLOAT NOT NULL DEFAULT 0.7,
  ai_config TEXT NOT NULL DEFAULT '{}',
  system_prompt TEXT,
  permissions TEXT,
  working_directories TEXT,
  skills TEXT,
  mcp_servers TEXT,
  messaging_connections TEXT,
  knowledge_bases TEXT,
  api_keys TEXT,
  created_at BIGINT NOT NULL DEFAULT 0,
  updated_at BIGINT NOT NULL DEFAULT 0,
  platform_configs TEXT,
  execution_settings TEXT
);

CREATE TABLE IF NOT EXISTS sessions (
  id TEXT NOT NULL PRIMARY KEY,
  agent_id TEXT NOT NULL,
  title TEXT,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  archived INTEGER NOT NULL DEFAULT 0,
  pinned INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY (agent_id) REFERENCES agents(id)
);

CREATE TABLE IF NOT EXISTS messages (
  id TEXT NOT NULL PRIMARY KEY,
  role TEXT NOT NULL,
  content TEXT NOT NULL,
  session_id TEXT NOT NULL,
  metadata_json TEXT,
  tokens INTEGER,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (session_id) REFERENCES sessions(id)
);
