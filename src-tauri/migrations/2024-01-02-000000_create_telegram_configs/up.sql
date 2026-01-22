CREATE TABLE telegram_configs (
  id TEXT NOT NULL PRIMARY KEY,
  bot_token TEXT NOT NULL,
  agent_id TEXT NOT NULL,
  is_active INTEGER NOT NULL DEFAULT 0,
  allowed_chat_ids TEXT,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (agent_id) REFERENCES agents(id)
);

CREATE INDEX idx_telegram_configs_agent_id ON telegram_configs(agent_id);
CREATE INDEX idx_telegram_configs_is_active ON telegram_configs(is_active);
