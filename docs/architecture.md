# AnyCowork Architecture

> **Last Updated**: 2026-01-20
> **Version**: 0.1.0

## Overview

AnyCowork is a native desktop AI assistant platform built with **Tauri** and **Rust**. It provides:

- **Native Performance**: Rust backend with minimal resource usage (**Optimized**)
- **Platform Independence**: Core library (anycowork-core) runs on Desktop, CLI, Server, Mobile
- **Smart AI**: Gemini 3 Pro via rig-core & MCP integration (**Smart**)
- **Safe Execution**: Human-in-the-loop permission system (**Safe**)
- **Sandboxed Tools**: Docker-based isolation for untrusted code execution
- **Telegram Integration**: Multi-bot support with teloxide
- **Local-First**: SQLite database with Diesel ORM

---

## Architecture Layers

```
┌─────────────────────────────────────────────────────────────┐
│                    PRESENTATION LAYER                        │
│  ┌──────────────────────────────────────────────────────┐   │
│  │           React + Vite Frontend                       │   │
│  │  • Components (shadcn/ui + Tailwind)                 │   │
│  │  • React Query for state management                  │   │
│  │  • Tauri IPC for backend communication               │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                           │
                    Tauri IPC Commands
                           │
┌─────────────────────────────────────────────────────────────┐
│                    ADAPTER LAYER                             │
│  ┌──────────────────────────────────────────────────────┐   │
│  │         anycowork-tauri (Platform Adapter)            │   │
│  │  • Tauri Commands (IPC handlers)                     │   │
│  │  • Permission Handler (UI-based approvals)           │   │
│  │  • Event Bridge (Core → Frontend)                    │   │
│  │  • Database Integration (Diesel + SQLite)            │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                           │
┌─────────────────────────────────────────────────────────────┐
│                    CORE LIBRARY LAYER                        │
│  ┌──────────────────────────────────────────────────────┐   │
│  │         anycowork-core (Platform-Independent)         │   │
│  │  ┌─────────────────┐  ┌─────────────────┐           │   │
│  │  │ Agent System    │  │ Tool System     │           │   │
│  │  │ - Coordinator   │  │ - BashTool      │           │   │
│  │  │ - Planner       │  │ - FileTool      │           │   │
│  │  │ - Router        │  │ - SearchTool    │           │   │
│  │  │ - Executor      │  │ - OfficeTool    │           │   │
│  │  └─────────────────┘  └─────────────────┘           │   │
│  │  ┌─────────────────┐  ┌─────────────────┐           │   │
│  │  │ Sandbox         │  │ Permissions     │           │   │
│  │  │ - Docker        │  │ - Manager       │           │   │
│  │  │ - WASM (future) │  │ - Policies      │           │   │
│  │  │ - Native        │  │ - Cache         │           │   │
│  │  └─────────────────┘  └─────────────────┘           │   │
│  │  ┌─────────────────┐  ┌─────────────────┐           │   │
│  │  │ Skills          │  │ Events          │           │   │
│  │  │ - Loader        │  │ - Channel       │           │   │
│  │  │ - Executor      │  │ - Subscriber    │           │   │
│  │  │ - Registry      │  │                 │           │   │
│  │  └─────────────────┘  └─────────────────┘           │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                           │
┌─────────────────────────────────────────────────────────────┐
│                    AI PROVIDER LAYER                         │
│  ┌──────────────────────────────────────────────────────┐   │
│  │              rig-core Framework                       │   │
│  │  ┌──────────────────────────────────────────────┐    │   │
│  │  │  Provider Clients (OpenAI, Anthropic, etc.)  │    │   │
│  │  │  • GPT-4, Claude, Gemini models              │    │   │
│  │  │  • Streaming responses                       │    │   │
│  │  │  • Tool/function calling                     │    │   │
│  │  │  • PromptHook for approvals                  │    │   │
│  │  └──────────────────────────────────────────────┘    │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                           │
┌─────────────────────────────────────────────────────────────┐
│                    DATA LAYER                                │
│  ┌──────────────────────────────────────────────────────┐   │
│  │         Diesel ORM + SQLite                           │   │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐               │   │
│  │  │ Agents  │ │ Sessions│ │Telegram │               │   │
│  │  │         │ │         │ │ Configs │               │   │
│  │  └─────────┘ └─────────┘ └─────────┘               │   │
│  │                                                       │   │
│  │            SQLite Database (anycowork.db)            │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## Core Components

### 0. anycowork-core (Platform-Independent Library)

**Purpose**: Provide platform-agnostic agent functionality

**Key Modules**:

**Agent System** (`agent/`):
- **Coordinator**: Orchestrates multi-agent workflows
- **Planner**: Creates execution plans from user requests
- **Router**: Routes tasks to appropriate agents
- **Executor**: Executes agent tasks with tool support

**Tool System** (`tools/`):
- **BashTool**: Execute shell commands in sandbox
- **FilesystemTool**: File operations (read, write, search)
- **SearchTool**: Code and content search
- **OfficeTool**: Document manipulation (future)

**Sandbox System** (`sandbox/`):
- **Sandbox trait**: Platform-agnostic execution interface
- **DockerSandbox**: Isolated Docker container execution
- **NativeSandbox**: Direct host execution (with permissions)

**Permission System** (`permissions/`):
- **PermissionHandler trait**: Platform-specific approval interface
- **PermissionManager**: Caching and policy enforcement
- **PermissionType**: Shell, Filesystem, Network permissions

**Event System** (`events/`):
- **EventChannel**: Broadcast channel for agent events
- **AgentEvent**: Type-safe event definitions
- **Platform-agnostic**: Adapters bridge to UI

**Skills System** (`skills/`):
- **SkillLoader**: Load skills from filesystem
- **SkillParser**: Parse skill.yaml definitions
- **SkillTool**: Execute skills in sandbox

### 1. anycowork-tauri (Tauri Adapter)

**Purpose**: Bridge anycowork-core to Tauri platform

**Key Components**:

**Commands** (`commands.rs`):
- Tauri command implementations
- Uses anycowork-core functionality
- Registered in src-tauri/lib.rs

**Permission Handler** (`permissions.rs`):
- Implements PermissionHandler trait
- Emits permission requests to frontend
- Handles approval/rejection responses

**Event Bridge** (`events.rs`):
- Subscribes to anycowork-core events
- Emits to Tauri frontend via window.emit()
- Session-scoped event routing

### 2. Tauri Application (src-tauri)

**Purpose**: Application entry point and state management

**Key Responsibilities**:
- Initialize application state
- Register Tauri commands from anycowork-tauri
- Manage database connection pool
- Window event handling

**Application State**:
```rust
pub struct AppState {
    pub db_pool: DbPool,
    pub telegram_manager: TelegramBotManager,
    pub core_coordinator: Arc<AgentCoordinator>, // from anycowork-core
}
```

### 3. Telegram Integration (telegram.rs)

**Purpose**: Manage multiple Telegram bot instances

**Components**:
- **TelegramBotManager**: Lifecycle management for bots
- **BotShutdownSender**: Graceful shutdown channel

**Features**:
- Start/stop individual bots
- Auto-start bots on application launch
- Chat ID filtering for security
- Integration with anycowork-core agent system

**Bot Flow**:
```
Telegram Message → teloxide Dispatcher
                        ↓
                        ↓
                Create/get session
                        ↓
                Execute via anycowork-core AgentCoordinator
                        ↓
                Send response to Telegram
```

### 4. Database Layer (database.rs, schema.rs, models/)

**Purpose**: Persistent storage with Diesel ORM

**Tables**:
- `agents` - Agent configurations
- `sessions` - Chat sessions
- `telegram_configs` - Telegram bot settings
- `mcp_servers` - MCP server configurations
- `skills` - Skill definitions and metadata

**Connection Pool**:
```rust
pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;
```

**Migrations**: Managed via Diesel CLI, auto-run on startup

---

## Frontend Architecture

### Technology Stack

- **Framework**: React 19
- **Build Tool**: Vite
- **Language**: TypeScript
- **Styling**: Tailwind CSS + shadcn/ui
- **State**: React Query (TanStack Query)
- **Routing**: React Router

### Project Structure

```
frontend/
├── src/
│   ├── routes/           # Page components
│   │   ├── HomePage.tsx
│   │   ├── ChatPage.tsx
│   │   ├── AgentsPage.tsx
│   │   └── SettingsPage.tsx
│   ├── components/       # Feature components
│   │   └── layout/
│   │       └── Sidebar.tsx
│   ├── lib/              # Utilities
│   │   └── a2ui-processor.ts
│   └── layouts/
│       └── MainLayout.tsx
├── components/           # Shared UI components
│   ├── ui/               # shadcn/ui components
│   └── ...
├── hooks/                # React hooks
└── lib/
    └── anycowork-api.ts  # Tauri IPC wrapper
```

### Tauri IPC Communication

Frontend communicates with Rust backend via Tauri's invoke API:

```typescript
// lib/anycowork-api.ts
import { invoke } from '@tauri-apps/api/core';

export async function createAgent(agent: AgentCreate): Promise<Agent> {
  return invoke('create_agent', { agent });
}

export async function chat(agentId: string, message: string): Promise<void> {
  return invoke('chat', { agentId, message });
}
```

### Event Listening

```typescript
import { listen } from '@tauri-apps/api/event';

// Listen for agent events
listen(`session:${sessionId}`, (event) => {
  const agentEvent = event.payload as AgentEvent;
  switch (agentEvent.type) {
    case 'Token':
      appendMessage(agentEvent.content);
      break;
    case 'JobCompleted':
      finishMessage(agentEvent.message);
      break;
  }
});
```

---

## Data Flow

### Chat Message Flow

```
1. User types message in ChatPage
        ↓
2. Frontend calls invoke('chat', { agentId, message })
        ↓
3. Tauri routes to chat() command in lib.rs
        ↓
4. AgentLoop.run() starts in background task
        ↓
5. Events emitted via window.emit()
        ↓
6. Frontend receives events via listen()
        ↓
7. UI updates in real-time (streaming tokens)
```

### Approval Workflow

```
1. Agent determines tool needs approval
        ↓
2. Emit ApprovalRequired event
        ↓
3. Frontend shows approval dialog
        ↓
4. User clicks Approve/Reject
        ↓
5. Frontend calls approve_action or reject_action
        ↓
6. Oneshot channel resolves
        ↓
7. AgentLoop continues or skips tool
```

---

## Configuration

### Environment Variables

```bash
# Required
OPENAI_API_KEY=your_api_key

# Optional
RUST_LOG=info    # Logging level
```

### Tauri Configuration (tauri.conf.json)

```json
{
  "identifier": "com.anycowork.app",
  "productName": "AnyCowork",
  "version": "0.1.0",
  "build": {
    "frontendDist": "../frontend/dist"
  },
  "app": {
    "windows": [{
      "title": "AnyCowork",
      "width": 1200,
      "height": 800
    }]
  }
}
```

### Database Location

- **Linux**: `~/.local/share/com.anycowork.app/anycowork.db`
- **macOS**: `~/Library/Application Support/com.anycowork.app/anycowork.db`
- **Windows**: `%APPDATA%\com.anycowork.app\anycowork.db`

---

## Security

### Sandboxing

- Tauri provides process isolation
- File operations restricted to app data directory
- File operations restricted to app data directory
 - Network requests go through system proxy
 
 ### Docker Sandbox (New)
 
 **Purpose**: Safe execution of untrusted code (Skills, Bash)
 
 **Features**:
 - **Isolation**: Runs code in `debian:stable-slim` or `alpine` containers
 - **Resource Limits**: Configurable RAM/CPU limits (e.g., 256MB RAM)
 - **Network**: Optional network access
 - **Mounts**: Workspace mounted as R/W, Skill files as R/O
 
 **Enforcement**:
 - Agents can enforce "Sandbox Mode" globally via `execution_settings`
 - Skills can require "Sandbox Mode" in `skill.yaml`
 - If Sandbox is required but Docker is missing, execution is blocked
 
 ### API Key Management

### API Key Management

- Keys stored in environment variables
- Never exposed to frontend
- Not stored in database

### Input Validation

- Diesel ORM prevents SQL injection
- Serde deserialization validates input types
- Path traversal prevented in file operations

---

## Design Patterns

### Command Pattern
**Used in**: Tauri IPC commands
```rust
#[tauri::command]
async fn create_agent(state: State<'_, AppState>, agent: NewAgent) -> Result<Agent, String>
```

### Observer Pattern
**Used in**: Event emission for real-time updates
```rust
window.emit(&format!("session:{}", session_id), event)?;
```

### Repository Pattern
**Used in**: Database operations
```rust
pub fn create_agent(conn: &mut SqliteConnection, new_agent: &NewAgent) -> QueryResult<Agent>
```

### State Machine
**Used in**: Agent execution status tracking
```
Pending → Running → (Waiting for Approval) → Completed/Failed
```

---

## Technology Stack

### Workspace Structure

The project uses a Cargo workspace with three main crates:

1. **anycowork-core** (`crates/anycowork-core/`)
   - Platform-independent core library
   - Agent system, tools, sandbox, permissions, events, skills
   - Can be used in CLI, server, mobile applications
   - No platform-specific dependencies

2. **anycowork-tauri** (`crates/anycowork-tauri/`)
   - Tauri platform adapter
   - Bridges anycowork-core to Tauri
   - Implements platform-specific handlers
   - Depends on anycowork-core

3. **src-tauri** (Tauri application)
   - Application entry point
   - Database integration
   - State management
   - Depends on anycowork-tauri

### Backend (Rust)
- **Framework**: Tauri 2.0
- **ORM**: Diesel
- **Database**: SQLite
- **AI**: rig-core (OpenAI)
- **Telegram**: teloxide
- **Async**: Tokio
- **Serialization**: Serde

### Frontend (TypeScript)
- **Framework**: React 19
- **Build**: Vite
- **Styling**: Tailwind CSS
- **Components**: shadcn/ui, Radix UI
- **State**: React Query
- **Routing**: React Router

---

## Future Enhancements

### Short-term
- Multiple AI provider support (Anthropic, Gemini)
- Conversation history persistence
- Custom tool definitions

### Medium-term
- Plugin system for extensibility
- Voice input/output
- Multi-window support

### Long-term
- P2P federation between instances
- Cloud sync (optional)
- Mobile companion app

---

## References

- **Tauri**: https://tauri.app/
- **Diesel**: https://diesel.rs/
- **rig-core**: https://github.com/0xPlaygrounds/rig
- **teloxide**: https://github.com/teloxide/teloxide
- **shadcn/ui**: https://ui.shadcn.com/

---

**Last Updated**: 2026-01-20
**Maintained By**: AnyCowork Development Team
