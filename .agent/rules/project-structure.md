# Project Structure

## Directory Layout

```
.
├── crates/                     # Rust workspace crates
│   ├── anycowork-core/         # Platform-independent core library
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── agent/          # Multi-agent system
│   │       │   ├── mod.rs
│   │       │   ├── coordinator.rs
│   │       │   ├── planner.rs
│   │       │   ├── router.rs
│   │       │   └── executor.rs
│   │       ├── tools/          # Rig Tool implementations
│   │       │   ├── mod.rs
│   │       │   ├── bash.rs
│   │       │   ├── filesystem.rs
│   │       │   ├── search.rs
│   │       │   └── office.rs
│   │       ├── sandbox/        # Execution environments
│   │       │   ├── mod.rs
│   │       │   ├── docker.rs
│   │       │   ├── native.rs
│   │       │   └── traits.rs
│   │       ├── skills/         # Skill system
│   │       │   ├── mod.rs
│   │       │   ├── loader.rs
│   │       │   ├── parser.rs
│   │       │   └── tool.rs
│   │       ├── permissions/    # Permission system
│   │       │   ├── mod.rs
│   │       │   ├── manager.rs
│   │       │   └── policy.rs
│   │       ├── events/         # Event system
│   │       │   ├── mod.rs
│   │       │   ├── types.rs
│   │       │   └── channel.rs
│   │       └── config.rs       # Configuration
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
├── CLAUDE.md                   # Agent guidelines
└── .env.example
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
