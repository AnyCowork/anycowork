CREATE TABLE knowledge_base_items (
  id TEXT NOT NULL PRIMARY KEY,
  type TEXT NOT NULL, -- 'upload' or 'link'
  name TEXT NOT NULL,
  path TEXT,
  pattern TEXT,
  link_type TEXT,
  indexing_status TEXT NOT NULL DEFAULT 'pending',
  indexing_error TEXT,
  total_files INTEGER,
  indexed_files INTEGER NOT NULL DEFAULT 0,
  retry_count INTEGER NOT NULL DEFAULT 0,
  last_indexed_at TIMESTAMP,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_kb_status ON knowledge_base_items(indexing_status);
CREATE INDEX idx_kb_type ON knowledge_base_items(type);
