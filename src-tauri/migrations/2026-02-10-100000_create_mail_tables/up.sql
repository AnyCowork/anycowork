CREATE TABLE mail_threads (
    id TEXT PRIMARY KEY NOT NULL,
    subject TEXT NOT NULL,
    is_read INTEGER NOT NULL DEFAULT 0,
    is_archived INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE mail_messages (
    id TEXT PRIMARY KEY NOT NULL,
    thread_id TEXT NOT NULL REFERENCES mail_threads(id) ON DELETE CASCADE,
    sender_type TEXT NOT NULL,
    sender_agent_id TEXT,
    recipient_type TEXT NOT NULL,
    recipient_agent_id TEXT,
    content TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
