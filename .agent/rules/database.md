# Database Schema & Management

## Schema Overview

The application uses SQLite with Diesel ORM.

### Tables

1. **agents** - AI agent definitions
   - id, name, description, system_prompt, created_at, updated_at

2. **messages** - Conversation history
   - id, role, content, agent_id (FK), created_at

3. **telegram_configs** - Telegram bot configurations
   - id, bot_token, agent_id (FK), is_active, allowed_chat_ids, created_at, updated_at

## Management

### Migration Commands

Run in `src-tauri/`:

```bash
# Run migrations
diesel migration run

# Revert last migration
diesel migration revert
```

### Models

Models are defined in `src-tauri/src/models.rs` and derive `Queryable`, `Selectable`, and `Serialize`.
