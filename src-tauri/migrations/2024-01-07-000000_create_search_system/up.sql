CREATE TABLE search_config (
  id INTEGER PRIMARY KEY DEFAULT 1,
  enabled INTEGER NOT NULL DEFAULT 1,
  auto_index INTEGER NOT NULL DEFAULT 1,
  index_pages INTEGER NOT NULL DEFAULT 1,
  index_blocks INTEGER NOT NULL DEFAULT 1,
  index_kb_files INTEGER NOT NULL DEFAULT 1,
  llm_type TEXT NOT NULL DEFAULT 'openai',
  llm_model TEXT NOT NULL DEFAULT 'gpt-4',
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CHECK (id = 1)
);

CREATE TABLE search_stats (
  id INTEGER PRIMARY KEY DEFAULT 1,
  total_documents INTEGER NOT NULL DEFAULT 0,
  total_pages INTEGER NOT NULL DEFAULT 0,
  total_blocks INTEGER NOT NULL DEFAULT 0,
  total_kb_files INTEGER NOT NULL DEFAULT 0,
  last_rebuild_at TIMESTAMP,
  CHECK (id = 1)
);

-- Insert default config
INSERT INTO search_config (id) VALUES (1);
INSERT INTO search_stats (id) VALUES (1);
