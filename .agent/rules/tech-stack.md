# Technology Stack & Architecture

## Project Overview

**AnyCowork** is an **open-source collaborative AI assistant platform** built as a native desktop application using **Tauri** and **Rust**. It combines agentic workflows, automation, and multi-platform connectivity (Telegram) with a "Local-First" philosophy.

**Key Philosophy**: "Local-first" - all data is stored locally in SQLite, AI API calls are the only network dependency.

## Architecture Summary

```
AnyCowork Desktop (Tauri)
├── Frontend (React + Vite)
│   ├── UI Components (shadcn/ui)
│   ├── State Management (React Query)
│   └── Tauri IPC Client
│
├── Backend (Rust + Tauri)
│   ├── Tauri Commands (IPC handlers)
│   ├── Agent System (rig-core)
│   ├── Telegram Bot (teloxide)
│   ├── Database (Diesel + SQLite)
│   └── Event System
│
└── Storage
    └── SQLite Database (local)
```

### Technology Stack

- **Backend**: Rust, Tauri 2.0, Diesel ORM
- **AI**: OpenAI GPT via rig-core
- **Telegram**: teloxide (async Telegram bot framework)
- **Frontend**: React 19, Vite, TypeScript, Tailwind CSS
- **UI**: shadcn/ui, Radix UI primitives
- **Database**: SQLite with Diesel migrations
