-- Normalizing the schema for AnyCowork Rust version

-- 1. Fix sessions table
CREATE TABLE sessions_new (
  id TEXT NOT NULL PRIMARY KEY,
  agent_id TEXT NOT NULL,
  title TEXT,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  archived INTEGER NOT NULL DEFAULT 0,
  pinned INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY (agent_id) REFERENCES agents(id)
);

-- Try to copy data if table exists, otherwise it's just a fresh table
INSERT INTO sessions_new (id, agent_id, title, archived, pinned)
SELECT id, agent_id, title, archived, pinned
FROM sessions;

DROP TABLE sessions;
ALTER TABLE sessions_new RENAME TO sessions;

-- 2. Fix messages table
CREATE TABLE messages_new (
  id TEXT NOT NULL PRIMARY KEY,
  role TEXT NOT NULL,
  content TEXT NOT NULL,
  session_id TEXT NOT NULL,
  metadata_json TEXT,
  tokens INTEGER,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

INSERT INTO messages_new (id, role, content, session_id, metadata_json, tokens)
SELECT id, role, content, session_id, metadata_json, tokens
FROM messages;

DROP TABLE messages;
ALTER TABLE messages_new RENAME TO messages;
