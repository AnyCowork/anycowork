# Project Structure

## Directory Layout

```
.
├── src/                        # React frontend source
│   ├── routes/                 # Page components (react-router)
│   │   ├── HomePage.tsx
│   │   ├── ChatPage.tsx
│   │   ├── AgentsPage.tsx
│   │   ├── TasksPage.tsx
│   │   ├── SettingsPage.tsx
│   │   └── ...
│   ├── components/             # Shared UI components
│   │   └── ui/                 # shadcn/ui components
│   ├── hooks/                  # React hooks
│   ├── lib/
│   │   ├── anycowork-api.ts    # Tauri IPC client
│   │   └── hooks/              # API hooks
│   ├── layouts/                # Layout components
│   ├── main.tsx                # Entry point
│   ├── App.tsx                 # Router setup
│   └── index.html
│
├── src-tauri/                  # Rust backend
│   ├── src/
│   │   ├── lib.rs              # Main app, Tauri commands, AppState
│   │   ├── main.rs             # Entry point
│   │   ├── agents/             # AI Agent System
│   │   │   ├── mod.rs          # Agent Worker Loop
│   │   │   ├── coordinator.rs  # Agent Coordinator
│   │   │   └── planner.rs      # Planning Agent
│   │   ├── telegram.rs         # Telegram bot manager (teloxide)
│   │   ├── database.rs         # Database connection pool
│   │   ├── models.rs           # Diesel models
│   │   ├── schema.rs           # Diesel schema
│   │   ├── events.rs           # Event types for frontend
│   ├── migrations/             # Diesel migrations
│   ├── icons/                  # App icons
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── docs/                       # Documentation
│   ├── architecture.md         # System architecture
│   ├── build.md                # Build instructions
│   └── ui-design.md            # UI design guidelines
│
├── public/                     # Static assets
├── package.json
├── vite.config.ts
├── tailwind.config.ts
├── README.md
├── CLAUDE.md                   # Agent guidelines
└── .env.example
```

## Key Files to Know

### Backend (Most Important)

- `src-tauri/src/lib.rs` - Main app logic, all Tauri commands defined here
- `src-tauri/src/agents/mod.rs` - Agent Worker Loop & Tool Execution
- `src-tauri/src/agents/coordinator.rs` - Agent Coordinator (Plan -> Execute)
- `src-tauri/src/telegram.rs` - Telegram bot manager using teloxide
- `src-tauri/src/database.rs` - Database connection pool setup
- `src-tauri/src/models.rs` - Diesel models (Agent, Message, TelegramConfig)
- `src-tauri/src/schema.rs` - Diesel table definitions
- `src-tauri/src/events.rs` - Event types emitted to frontend

### Frontend (Most Important)

- `src/lib/anycowork-api.ts` - API client with all Tauri IPC calls
- `src/hooks/use-anycowork.ts` - React Query hooks (if applicable) or check `hooks/`
- `src/routes/` - Page components
- `src/App.tsx` - Router setup

### Configuration

- `src-tauri/Cargo.toml` - Rust dependencies
- `src-tauri/tauri.conf.json` - Tauri configuration
- `package.json` - Frontend dependencies
- `.env` - Environment variables (API keys)
