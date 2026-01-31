CREATE TABLE mcp_servers (
  id TEXT NOT NULL PRIMARY KEY,
  name TEXT NOT NULL,
  server_type TEXT NOT NULL, -- 'stdio' or 'sse'
  command TEXT, -- for stdio
  args TEXT, -- json array for stdio
  env TEXT, -- json object for stdio
  url TEXT, -- for sse
  is_enabled INTEGER NOT NULL DEFAULT 1,
  template_id TEXT, -- to track which template was used
  created_at BIGINT NOT NULL,
  updated_at BIGINT NOT NULL
);
