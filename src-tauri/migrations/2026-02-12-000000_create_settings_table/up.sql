-- Create settings table for global application configuration
CREATE TABLE IF NOT EXISTS settings (
    id TEXT PRIMARY KEY NOT NULL,
    key TEXT NOT NULL UNIQUE,
    value TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Insert default AI configuration
INSERT OR IGNORE INTO settings (id, key, value) VALUES
    ('ai_provider', 'ai_provider', 'openai'),
    ('anthropic_api_key', 'anthropic_api_key', ''),
    ('anthropic_model', 'anthropic_model', 'claude-opus-4-5-20251101'),
    ('openai_api_key', 'openai_api_key', ''),
    ('openai_model', 'openai_model', 'gpt-4o'),
    ('gemini_api_key', 'gemini_api_key', ''),
    ('gemini_model', 'gemini_model', 'gemini-2.0-flash-exp'),
    ('max_tokens', 'max_tokens', '4096'),
    ('temperature', 'temperature', '0.7');
