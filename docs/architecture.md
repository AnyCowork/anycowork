# AnyCowork Architecture

> **Last Updated**: 2026-01-20
> **Version**: 0.1.0

## Overview

AnyCowork is a native desktop AI assistant platform built with **Tauri** and **Rust**. It provides:

- **Native Performance**: Rust backend with minimal resource usage
- **AI Integration**: OpenAI GPT via rig-core framework
- **Telegram Integration**: Multi-bot support with teloxide
- **Local-First**: SQLite database with Diesel ORM
- **Event-Driven**: Real-time communication via Tauri events

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
│                    APPLICATION LAYER                         │
│  ┌──────────────────────────────────────────────────────┐   │
│  │            Tauri Commands (lib.rs)                    │   │
│  │  • Agent CRUD operations                             │   │
│  │  • Chat message handling                             │   │
│  │  • Telegram bot management                           │   │
│  │  • Approval workflow                                 │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                           │
┌─────────────────────────────────────────────────────────────┐
│                    SERVICE LAYER                             │
│  ┌───────────┐  ┌───────────┐  ┌───────────┐               │
│  │  Agent    │  │ Telegram  │  │  Events   │               │
│  │  Loop     │  │  Manager  │  │  System   │               │
│  │(agents.rs)│  │(telegram.rs)│ │(events.rs)│               │
│  └───────────┘  └───────────┘  └───────────┘               │
└─────────────────────────────────────────────────────────────┘
                           │
┌─────────────────────────────────────────────────────────────┐
│                    AI PROVIDER LAYER                         │
│  ┌──────────────────────────────────────────────────────┐   │
│  │              rig-core Framework                       │   │
│  │  ┌──────────────────────────────────────────────┐    │   │
│  │  │            OpenAI Client                      │    │   │
│  │  │  • GPT-4, GPT-3.5 models                     │    │   │
│  │  │  • Streaming responses                       │    │   │
│  │  │  • Tool/function calling                     │    │   │
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

### 1. Tauri Application (lib.rs)

**Purpose**: Main application entry point and IPC command definitions

**Key Responsibilities**:
- Initialize application state
- Define Tauri commands for frontend communication
- Manage shared state (database pool, bot manager, pending approvals)
- Window event handling

**Application State**:
```rust
pub struct AppState {
    pub db_pool: DbPool,
    pub telegram_manager: TelegramBotManager,
    pub pending_approvals: Arc<DashMap<String, oneshot::Sender<bool>>>,
}
```

**Command Categories**:
- Agent management: `create_agent`, `get_agents`
- Chat: `chat`
- Telegram: `create_telegram_config`, `start_telegram_bot`, etc.
- Approval: `approve_action`, `reject_action`

### 2. Agent System (agents.rs)

**Purpose**: Execute AI-powered conversations with tool support

**Components**:
- **AgentLoop**: Manages conversation state and AI interactions
- **ExecutionJob**: Represents a running agent task
- **ExecutionStep**: Individual tool execution within a job

**Execution Flow**:
```
User Message → AgentLoop.run()
                    ↓
            Emit "Thinking" event
                    ↓
            Check for tool calls (e.g., "delete")
                    ↓
            [If tool needs approval]
                    ↓
            Emit ApprovalRequired → Wait for user response
                    ↓
            Execute tool or skip
                    ↓
            Stream AI response via rig-core
                    ↓
            Emit Token events (streaming)
                    ↓
            Emit JobCompleted
```

**Event Types**:
- `JobStarted` - Agent begins processing
- `Thinking` - Agent is processing
- `ApprovalRequired` - Tool needs user approval
- `StepApproved` / `StepRejected` - Approval result
- `StepCompleted` - Tool execution finished
- `Token` - Streaming response chunk
- `JobCompleted` - Agent finished

### 3. Telegram Integration (telegram.rs)

**Purpose**: Manage multiple Telegram bot instances

**Components**:
- **TelegramBotManager**: Lifecycle management for bots
- **BotShutdownSender**: Graceful shutdown channel

**Features**:
- Start/stop individual bots
- Auto-start bots on application launch
- Chat ID filtering for security
- Integration with agent system

**Bot Flow**:
```
Telegram Message → teloxide Dispatcher
                        ↓
                Check allowed chats
                        ↓
                Find linked agent
                        ↓
                Create/get session
                        ↓
                Execute via AgentLoop
                        ↓
                Send response to Telegram
```

### 4. Database Layer (database.rs, schema.rs, models.rs)

**Purpose**: Persistent storage with Diesel ORM

**Tables**:
- `agents` - Agent configurations
- `sessions` - Chat sessions
- `telegram_configs` - Telegram bot settings

**Connection Pool**:
```rust
pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;
```

**Migrations**: Managed via Diesel CLI, auto-run on startup

### 5. Event System (events.rs)

**Purpose**: Type-safe event definitions for frontend communication

**Event Structures**:
```rust
pub enum AgentEvent {
    JobStarted { job: ExecutionJob },
    Thinking { message: String },
    ApprovalRequired { job: ExecutionJob, step: ExecutionStep },
    StepApproved { job: ExecutionJob, step: ExecutionStep },
    StepRejected { job: ExecutionJob, step: ExecutionStep },
    StepCompleted { job: ExecutionJob, step: ExecutionStep },
    Token { content: String },
    JobCompleted { job: ExecutionJob, message: String },
}
```

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
- Network requests go through system proxy

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
