# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) and other AI assistants when working with code in this repository.

## Project Overview

**AnyCowork** is an **open-source collaborative AI assistant platform** built as a native desktop application using **Tauri** and **Rust**. It combines agentic workflows, automation, and multi-platform connectivity (Telegram) with a "Local-First" philosophy.

**Key Philosophy**: "Local-first" - all data is stored locally in SQLite, AI API calls are the only network dependency.

## Architecture Summary

```
┌─────────────────────────────────────────────────────────────┐
│                    Platform Adapters                         │
├──────────────┬──────────────┬───────────────┬───────────────┤
│ Tauri Desktop│   CLI        │  Server/API   │ Tauri Mobile  │
│ (current)    │  (future)    │  (future)     │ (future)      │
└──────┬───────┴──────┬───────┴───────┬───────┴───────┬───────┘
       │              │               │               │
       └──────────────┴───────┬───────┴───────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     anycowork-core                           │
│  Platform-Independent Agent Library                          │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │ Agent System    │  │ Tool System     │  │ Skills       │ │
│  │ - Coordinator   │  │ - BashTool      │  │ - Loader     │ │
│  │ - Planner       │  │ - FileTool      │  │ - Executor   │ │
│  │ - Router        │  │ - SearchTool    │  │ - Registry   │ │
│  │ - Executor      │  │ - OfficeTool    │  │              │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │ Sandbox         │  │ Permissions     │  │ Events       │ │
│  │ - Docker        │  │ - Manager       │  │ - Channel    │ │
│  │ - WASM (future) │  │ - Policies      │  │ - Subscriber │ │
│  │ - Native        │  │ - Cache         │  │              │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                        rig-core                              │
│  - Provider clients (OpenAI, Anthropic, Gemini, Ollama...)  │
│  - Tool trait & ToolSet                                      │
│  - PromptHook for approvals                                  │
│  - Multi-turn agent execution                                │
│  - Streaming                                                 │
└─────────────────────────────────────────────────────────────┘

AnyCowork Desktop (Tauri) - Current Implementation
├── Frontend (React + Vite)
│   ├── UI Components (shadcn/ui)
│   ├── State Management (React Query)
│   └── Tauri IPC Client
│
├── Tauri Adapter (anycowork-tauri)
│   ├── Tauri Commands (IPC handlers)
│   ├── Permission Handler (UI-based approvals)
│   ├── Event Bridge (Core → Frontend)
│   └── Database Integration (Diesel + SQLite)
│
├── Core Library (anycowork-core)
│   ├── Agent System (Coordinator, Planner, Router)
│   ├── Tool System (Bash, Filesystem, Search, Office)
│   ├── Sandbox System (Docker, Native)
│   ├── Permission System (Platform-agnostic)
│   ├── Event System (Platform-agnostic)
│   └── Skills System (Loader, Executor)
│
└── Storage
    └── SQLite Database (local)
```

### Technology Stack

- **Core Library**: anycowork-core (Platform-independent Rust library)
- **Backend**: Rust, Tauri 2.0, Diesel ORM
- **AI**: OpenAI GPT via rig-core
- **Sandbox**: Docker (isolation), Native (fallback)
- **Telegram**: teloxide (async Telegram bot framework)
- **Frontend**: React 19, Vite, TypeScript, Tailwind CSS
- **UI**: shadcn/ui, Radix UI primitives
- **Database**: SQLite with Diesel migrations

## Project Structure

```
.
├── crates/                     # Rust workspace crates
│   ├── anycowork-core/         # Platform-independent core library
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── agent/          # Multi-agent system
│   │       ├── tools/          # Rig Tool implementations
│   │       ├── sandbox/        # Execution environments
│   │       ├── skills/         # Skill system
│   │       ├── permissions/    # Permission system
│   │       ├── events/         # Event system
│   │       └── config.rs
│   │
│   └── anycowork-tauri/        # Tauri platform adapter
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── commands.rs     # Tauri commands
│           ├── events.rs       # Tauri event emission
│           └── permissions.rs  # Tauri permission handler
│
├── src/                        # React frontend source
│   ├── routes/                 # Page components (react-router)
│   ├── components/             # Shared UI components
│   ├── hooks/                  # React hooks
│   ├── lib/
│   │   ├── anycowork-api.ts    # Tauri IPC client
│   │   └── hooks/              # API hooks
│   ├── layouts/                # Layout components
│   ├── main.tsx                # Entry point
│   ├── App.tsx                 # Router setup
│   └── index.html
│
├── src-tauri/                  # Tauri app entry point
│   ├── src/
│   │   ├── lib.rs              # Main app, Tauri commands, AppState
│   │   ├── main.rs             # Entry point
│   │   ├── database.rs         # Database connection pool
│   │   ├── models/             # Diesel models
│   │   └── schema.rs           # Diesel schema
│   ├── migrations/             # Diesel migrations
│   ├── skills/                 # Skill definitions
│   ├── icons/                  # App icons
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── docs/                       # Documentation
│   ├── anycowork-core-plan.md # Core library architecture plan
│   ├── architecture.md         # System architecture
│   ├── build.md                # Build instructions
│   └── ui-design.md            # UI design guidelines
│
├── public/                     # Static assets
├── Cargo.toml                  # Workspace root
├── package.json
├── vite.config.ts
├── tailwind.config.ts
├── README.md
├── CLAUDE.md                   # This file
└── .env.example
```

## Development Commands

### Full Stack (Recommended)

```bash
# Run both frontend and backend in development mode
npm run tauri dev
```

### Backend (Rust + Tauri)

```bash
# Build entire workspace
cargo build

# Build specific crate
cargo build -p anycowork-core
cargo build -p anycowork-tauri

# Run tests
cargo test

# Run tests for specific crate
cargo test -p anycowork-core

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

## Key Files to Know

### Core Library (anycowork-core)

- `crates/anycowork-core/src/lib.rs` - Core library entry point
- `crates/anycowork-core/src/agent/coordinator.rs` - Agent coordination logic
- `crates/anycowork-core/src/agent/planner.rs` - Planning agent
- `crates/anycowork-core/src/tools/` - Rig Tool implementations (bash, filesystem, etc.)
- `crates/anycowork-core/src/sandbox/` - Sandbox abstractions (Docker, native)
- `crates/anycowork-core/src/permissions/` - Permission system
- `crates/anycowork-core/src/events/` - Platform-agnostic event system
- `crates/anycowork-core/src/skills/` - Skill loading and execution

### Tauri Adapter (anycowork-tauri)

- `crates/anycowork-tauri/src/lib.rs` - Tauri adapter entry point
- `crates/anycowork-tauri/src/commands.rs` - Tauri command implementations
- `crates/anycowork-tauri/src/events.rs` - Event bridge to frontend
- `crates/anycowork-tauri/src/permissions.rs` - Tauri permission handler

### Backend (Tauri App)

- `src-tauri/src/lib.rs` - Main app logic, AppState, command registration
- `src-tauri/src/main.rs` - Application entry point
- `src-tauri/src/database.rs` - Database connection pool setup
- `src-tauri/src/models/` - Diesel models (Agent, Session, etc.)
- `src-tauri/src/schema.rs` - Diesel table definitions

### Frontend (Most Important)

- `src/lib/anycowork-api.ts` - API client with all Tauri IPC calls
- `src/hooks/use-anycowork.ts` - React Query hooks (if applicable) or check `hooks/`
- `src/routes/` - Page components
- `src/App.tsx` - Router setup

### Configuration

- `Cargo.toml` - Workspace root configuration
- `crates/anycowork-core/Cargo.toml` - Core library dependencies
- `crates/anycowork-tauri/Cargo.toml` - Tauri adapter dependencies
- `src-tauri/Cargo.toml` - Tauri app dependencies
- `src-tauri/tauri.conf.json` - Tauri configuration
- `package.json` - Frontend dependencies
- `.env` - Environment variables (API keys)

## Database Schema

### Tables

1. **agents** - AI agent definitions
   - id, name, description, system_prompt, created_at, updated_at

2. **messages** - Conversation history
   - id, role, content, agent_id (FK), created_at

3. **telegram_configs** - Telegram bot configurations
   - id, bot_token, agent_id (FK), is_active, allowed_chat_ids, created_at, updated_at

## Tauri Commands (IPC)

Commands are defined in `src-tauri/src/lib.rs` using `#[tauri::command]`.

### Agent Commands
```rust
create_agent(name, description, system_prompt) -> Agent
get_agents() -> Vec<Agent>
chat(agent_id, message) -> String  // Starts background task, emits events
approve_action(step_id) -> ()
reject_action(step_id) -> ()
```

### Telegram Commands
```rust
create_telegram_config(bot_token, agent_id, allowed_chat_ids?) -> TelegramConfig
get_telegram_configs() -> Vec<TelegramConfig>
get_telegram_config(config_id) -> TelegramConfig
update_telegram_config(config_id, new_bot_token?, new_agent_id?, new_is_active?, new_allowed_chat_ids?) -> TelegramConfig
delete_telegram_config(config_id) -> ()
start_telegram_bot(config_id) -> ()
stop_telegram_bot(config_id) -> ()
get_telegram_bot_status(config_id) -> TelegramBotStatus
get_running_telegram_bots() -> Vec<String>
```

## Event System

Events are emitted from Rust to frontend using Tauri's event system.

```rust
// In Rust
window.emit("session:{session_id}", AgentEvent::JobStarted { job });

// In TypeScript
listen("session:xxx", (event) => { ... });
```

### Event Types (in `events.rs`)
- `JobStarted` - Agent job started
- `JobCompleted` - Agent job completed with message
- `StepStarted/StepCompleted` - Tool execution steps
- `ApprovalRequired` - Tool needs user approval
- `StepApproved/StepRejected` - Approval response
- `Thinking` - Agent is processing
- `Error` - Error occurred

## Adding New Features

### Adding Core Library Features

1. Implement in `crates/anycowork-core/src/`:
```rust
// Example: New tool in crates/anycowork-core/src/tools/my_tool.rs
use rig::tool::{Tool, ToolDefinition};

pub struct MyTool { /* ... */ }

impl Tool for MyTool {
    const NAME: &'static str = "my_tool";
    // ... implementation
}
```

2. Export from module:
```rust
// crates/anycowork-core/src/tools/mod.rs
pub mod my_tool;
pub use my_tool::MyTool;
```

3. Use in Tauri adapter:
```rust
// crates/anycowork-tauri/src/commands.rs
use anycowork_core::tools::MyTool;
```

### Adding a New Tauri Command

1. Define the command in `crates/anycowork-tauri/src/commands.rs`:
```rust
#[tauri::command]
pub async fn my_command(
    state: State<'_, AppState>,
    param: String
) -> Result<MyResult, String> {
    // Use anycowork-core functionality
    Ok(result)
}
```

2. Register in `src-tauri/src/lib.rs`:
```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands
    anycowork_tauri::commands::my_command,
])
```

3. Add to frontend API (`lib/anycowork-api.ts`):
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

## Code Style

### Rust
- Use `cargo fmt` for formatting
- Use `cargo clippy` for lints
- Async/await for all I/O operations
- Error handling with `Result<T, String>` for Tauri commands
- Use `log` crate for logging

### TypeScript
- ESLint configured
- Functional components with hooks
- TypeScript strict mode
- Use React Query for server state

### UI Design
- **MUST follow `docs/ui-design.md`** for all frontend work
- Use `lucide-react` for icons
- Use `shadcn/ui` components
- Use Tailwind CSS utilities
- Responsive design (mobile-first)

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

## Notes for AI Assistants

When working with this codebase:

1. **Read this file first** to understand project structure
2. **Understand the workspace structure**: anycowork-core (platform-independent) → anycowork-tauri (adapter) → src-tauri (app)
3. **Core logic goes in anycowork-core** for platform independence
4. **Platform-specific code goes in anycowork-tauri** for Tauri integration
5. **Check existing patterns** before adding new code
6. **Use async/await** in Rust for all I/O
7. **Use React Query** for data fetching in frontend
8. **Follow UI guidelines** in `docs/ui-design.md`
9. **Test both frontend and backend** after changes
10. **Never commit API keys** or secrets
11. **Update types** in both Rust and TypeScript when changing APIs
12. **See docs/anycowork-core-plan.md** for detailed architecture plan

## Version History

- **v0.1.0** - Initial Tauri implementation
  - Agent system with rig-core
  - Telegram bot integration with teloxide
  - React frontend with shadcn/ui
  - SQLite database with Diesel

---

**Built with Tauri, Rust, and React**

*Your collaborative AI assistant* • [anycowork.com](https://www.anycowork.com)
