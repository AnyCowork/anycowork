# AnyCowork Agent System Design

## Overview (Inspired by OpenCode)

The goal is to build a flexible, secure, and interactive agent system for AnyCowork.
This system will allow agents to execute complex tasks using tools, while giving users granular control through a permission system.
Agents can be "primary" (driving the chat) or "subagents" (specialized tasks).

## Core Concepts

### 1. Agents & Agent Client Protocol (ACP)
An `Agent` is a configured entity that uses an LLM to perform tasks.
We will adopt an **ACP-like architecture** where agents can be treated as clients or servers, allowing for:
-   **Agent-to-Agent Communication**: A primary agent delegating tasks to sub-agents.
-   **Standardized Interface**: `session/new`, `session/prompt`, `session/load`.

### 2. Tools & MCP (Model Context Protocol)
We will integrate the **Model Context Protocol (MCP)** to standardize tool and resource discovery.
-   **MCP Client**: The AnyCowork backend acts as an MCP Client.
-   **MCP Servers**: Connect to external tools (Database, git, etc.) or internal modules.
-   **Transport**: JSON-RPC over stdio (local) or SSE (remote).

**Rust Implementation:**
-   Use `jsonrpc-core` or similar to implement the MCP protocol.
-   `McpManager` to handle connections and lifecycle.

### 3. Permissions
A critical layer for security and trust.
-   **Granular Control**: Per-tool, per-resource (file path) verification.
-   **Flow**: Tool Call -> Permission Check -> (Block & Ask UI) -> User Decision -> Resume.
-   **Persistence**: Support "Always Allow" rules for specific sessions or global scope.

### 4. Workflows & Events (Event Bus)
The system works on an Event Bus model.
-   **User Message** -> Triggers Agent.
-   **Stream Events**:
    -   `text-delta`: Standard chat output.
    -   `reasoning-delta`: "Thinking" process (e.g., `<think>...</think>` content).
    -   `tool-call`: Request to execute a tool.
    -   `permission-request`: System asking for user approval.

### 5. Checkpoints & State Management
To support robust long-running tasks:
-   **Snapshots**: Capture the state of the workspace (file hashes) before and after tool execution.
-   **Diffs**: Compute and store changes (`additions`, `deletions`).
-   **Usage**: Allow users to "rollback" to a previous state (basic implementation via git or simple backup).

## Interaction Model

### Frontend
-   **Chat Stream**: Renders text and "Thinking" blocks separately (collapsible "Thinking" UI).
-   **Intervention UI**: "Pending Approval" cards for permission requests.
-   **Artifacts**: Display diffs or file changes summary after tool execution.

### Backend Structure
```
src-tauri/src/
  agents/
    mod.rs         # Agent logic
    processor.rs   # Stream & Logic Processor (new)
  mcp/             # MCP Implementation
    client.rs
    transport.rs
  permissions/
    mod.rs         # Permission logic
  snapshots/       # State management
    mod.rs
```
