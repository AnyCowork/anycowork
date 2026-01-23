# Development Workflow

## Development Commands

### Full Stack (Recommended)

```bash
# Run both frontend and backend in development mode
npm run tauri dev
```

### Backend (Rust + Tauri)

```bash
cd src-tauri

# Build
cargo build

# Run tests
cargo test

# Format code
cargo fmt

# Check lints
cargo clippy

# Build for production
cargo tauri build
```

### Frontend (React + Vite)

```bash
# Install dependencies
npm install

# Development server (browser only, no Tauri APIs)
npm run dev

# Build for production
npm run build

# Lint
npm run lint
```

### Database

```bash
cd src-tauri

# Run migrations (automatic on startup, but can run manually)
diesel migration run

# Create new migration
diesel migration generate migration_name

# Revert migration
diesel migration revert
```

## Adding New Features

### Adding a New Tauri Command

1. Define the command in `src/lib.rs`:
```rust
#[tauri::command]
async fn my_command(state: State<'_, AppState>, param: String) -> Result<MyResult, String> {
    // Implementation
}
```

2. Register in `invoke_handler`:
```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands
    my_command,
])
```

3. Add to frontend API (`frontend/lib/anycowork-api.ts`):
```typescript
myCommand: async (param: string) => {
    return invoke<MyResult>('my_command', { param });
},
```

### Adding a New Database Table

1. Create migration:
```bash
cd src-tauri
diesel migration generate create_my_table
```

2. Edit `up.sql` and `down.sql`

3. Update `src/schema.rs` with table definition

4. Add model in `src/models.rs`:
```rust
#[derive(Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::my_table)]
pub struct MyModel { ... }
```

### Adding Frontend Components

1. Use shadcn/ui components when possible
2. Follow existing patterns in `frontend/src/routes/`
3. Use React Query hooks for data fetching
4. Define types in `frontend/lib/anycowork-api.ts`

## Common Development Tasks

### Testing Telegram Bot

1. Create a bot via @BotFather on Telegram
2. Get the bot token
3. Add configuration via Settings UI
4. Start the bot
5. Send a message to the bot

### Debugging

**Backend:**
- Check console output from `cargo tauri dev`
- Use `log::info!()`, `log::error!()` for logging
- Set `RUST_LOG=debug` for verbose output

**Frontend:**
- Browser DevTools console
- React Query DevTools (in dev mode)
- Check Network tab for IPC calls

**Database:**
- SQLite file is in system data directory
- Use `sqlite3` CLI to inspect

## Security Considerations

1. **API Keys** - Store in `.env`, never commit
2. **Telegram Tokens** - Stored encrypted in database (future)
3. **File Access** - Agent tools restricted to workspace
4. **Command Execution** - Requires user approval
