# Technology Stack & Architecture

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

### Technology Stack

- **Core Library**: anycowork-core (Platform-independent Rust library)
- **Backend**: Rust, Tauri 2.0, Diesel ORM
- **AI**: Gemini 3 Pro (Primary), rig-core (Orchestration)
- **Extensibility**: MCP (Model Context Protocol) for tools
- **Sandbox**: Docker (isolation), Native (fallback)
- **Telegram**: teloxide (async Telegram bot framework)
- **Frontend**: React 19, Vite, TypeScript, Tailwind CSS
- **UI**: shadcn/ui, Radix UI primitives
- **Database**: SQLite with Diesel migrations

## Workspace Structure

The project uses a Cargo workspace with three main crates:

1. **anycowork-core** - Platform-independent core library
   - Agent system (Coordinator, Planner, Router, Executor)
   - Tool implementations (Bash, Filesystem, Search, Office)
   - Sandbox abstractions (Docker, Native)
   - Permission system (platform-agnostic)
   - Event system (platform-agnostic)
   - Skills system (Loader, Executor)

2. **anycowork-tauri** - Tauri platform adapter
   - Tauri command implementations
   - Permission handler (UI-based approvals)
   - Event bridge (Core → Frontend)
   - Platform-specific integrations

3. **src-tauri** - Tauri application entry point
   - Application state management
   - Database integration (Diesel + SQLite)
   - Command registration
   - Window management
