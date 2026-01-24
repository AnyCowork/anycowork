# AnyCowork Agent System Design

## Overview

The goal is to build a flexible, secure, and interactive agent system for AnyCowork. This system will allow agents to execute complex tasks using tools, while giving users granular control through a permission system. The architecture supports both single-agent and multi-agent workflows with planning and execution phases.

**Core Design Goals:**
1. **Robustness & Extensibility**: Simplify the flow while making it easier to add new capabilities.
2. **Full Message Storage**: Store and display messages correctly and fully (custom truncation only for context window, not for storage).
3. **Multi-Threading/Parallelism**: Allow independent tasks to run in parallel.
4. **Multi-Agent Architecture**: Support planning agents and executing agents working together.
5. **Security & Trust**: Granular permission system for user control.

---

## Core Concepts

### 1. Agents & Agent Client Protocol (ACP)

An `Agent` is a configured entity that uses an LLM to perform tasks. We adopt an **ACP-like architecture** where agents can be treated as clients or servers, allowing for:

- **Agent-to-Agent Communication**: A primary agent delegating tasks to sub-agents.
- **Standardized Interface**: `session/new`, `session/prompt`, `session/load`.
- **Role Specialization**: Different agents for planning vs. execution.

**Agent Types:**
- **Planning Agent**: Analyzes requests and creates structured execution plans.
- **Executing Agent**: Picks up tasks from the plan and executes them using tools.
- **Primary Agent**: Drives the chat and coordinates workflow.
- **Sub-agents**: Specialized for specific tasks.

### 2. Multi-Agent Architecture

#### The Coordinator (Engine)

The central engine that manages the lifecycle of a request.

- **Input**: User message.
- **State**: Current `Plan`, execution history.
- **Loop**:
  1. If no Plan exists, ask **Planning Agent** to create one.
  2. If Plan exists, check for next available tasks (considering dependencies).
  3. Dispatch tasks to **Executing Agents** (Worker Pool).
  4. Collect results and update state.
  5. Update Plan (mark tasks as done/failed).
  6. Repeat until Plan is complete or failed.

#### Planning Agent

- **Role**: Architect and task decomposer.
- **Tools**: None (or minimal - Search/Read for context only).
- **Output**: A structured `Plan` (JSON format).
  - `tasks`: List of atomic, executable tasks.
  - `dependencies`: Task dependency graph.
- **Prompt Example**: *"You are a senior planner. Break down this request into atomic, executable steps with clear dependencies..."*

#### Executing Agent (Worker)

- **Role**: Task executor.
- **Tools**: Full access - Filesystem, Search, Bash, etc.
- **Input**: A single `Task` description + context.
- **Output**: Result of execution (success/failure with details).
- **Behavior**:
  - Multi-turn tool execution within a task.
  - Explicit success/failure reporting.
  - Can request additional context if needed.

#### Parallel Execution

- The Coordinator analyzes task dependencies.
- If `Task A` and `Task B` have no dependencies, spawn concurrent `tokio::task` for each.
- Join results before proceeding to dependent tasks.
- **Risk Mitigation**: Start with sequential execution for safety, but design structure to support parallelism.

### 3. Tools & MCP (Model Context Protocol)

We integrate the **Model Context Protocol (MCP)** to standardize tool and resource discovery.

- **MCP Client**: AnyCowork backend acts as an MCP Client.
- **MCP Servers**: Connect to external tools (Database, git, etc.) or internal modules.
- **Transport**: JSON-RPC over stdio (local) or SSE (remote).

**Rust Implementation:**
- Use `jsonrpc-core` or similar to implement the MCP protocol.
- `McpManager` to handle connections and lifecycle.
- Tool registry for dynamic tool discovery.

### 4. Permissions

A critical layer for security and trust.

- **Granular Control**: Per-tool, per-resource (file path) verification.
- **Flow**:
  ```
  Tool Call -> Permission Check -> (Block & Ask UI) -> User Decision -> Resume
  ```
- **Persistence**: Support "Always Allow" rules for specific sessions or global scope.
- **Approval Types**:
  - Manual approval required
  - Auto-approve for whitelisted commands/tools
  - Smart approval based on risk assessment

### 5. Workflows & Events (Event Bus)

The system works on an Event Bus model for real-time communication.

**Stream Events:**
- `text-delta`: Standard chat output.
- `reasoning-delta`: "Thinking" process (e.g., `<think>...</think>` content).
- `tool-call`: Request to execute a tool.
- `permission-request`: System asking for user approval.
- `plan-update`: Task list updates (progress, status changes).
- `task-started`: Individual task execution begins.
- `task-completed`: Individual task execution completes.
- `job-completed`: Entire execution job finishes.

**User Message Flow:**
```
User Message -> Coordinator
  -> Planning Agent (if no plan)
  -> Plan Created (emit plan-update)
  -> Execute Tasks (emit task-started/completed)
  -> Tools (emit tool-call, permission-request if needed)
  -> Job Complete (emit job-completed)
```

### 6. Checkpoints & State Management

To support robust long-running tasks:

- **Snapshots**: Capture workspace state (file hashes) before and after tool execution.
- **Diffs**: Compute and store changes (`additions`, `deletions`).
- **Usage**: Allow users to "rollback" to a previous state (basic implementation via git or simple backup).
- **Message Storage**:
  - **Database**: Full message content stored without truncation.
  - **Context Window**: Truncation only when loading history for LLM (`optimize_history_by_tokens`).
  - **Distinction**: Messages categorized as "Plan", "Task Assignment", "Task Execution", "Task Result".

---

## Data Structures

### Plan & Task Specifications

```rust
struct Plan {
    tasks: Vec<TaskSpec>,
}

struct TaskSpec {
    id: String,
    description: String,
    dependencies: Vec<String>, // IDs of tasks that must finish first
    status: TaskStatus,
}

enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

struct ExecutionJob {
    id: String,
    session_id: String,
    query: String,
    status: JobStatus,
    plan: Option<Plan>,
    steps: Vec<ExecutionStep>,
    current_step_index: usize,
    final_response: Option<String>,
    error: Option<String>,
    created_at: String,
    completed_at: Option<String>,
}
```

### Message Storage Schema

```sql
-- Messages table needs to distinguish between different message types
CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    role TEXT NOT NULL, -- 'user', 'assistant', 'system', 'tool'
    content TEXT NOT NULL, -- Full content, no truncation
    metadata_json TEXT, -- Store plan data, task info, etc.
    message_type TEXT, -- 'plan', 'task_assignment', 'task_execution', 'task_result', 'normal'
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Optional: Dedicated tasks table for better queryability
CREATE TABLE tasks (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    job_id TEXT,
    description TEXT NOT NULL,
    status TEXT NOT NULL,
    dependencies TEXT, -- JSON array of task IDs
    result TEXT,
    error TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP
);
```

---

## Implementation Details

### Refactoring Steps

1. **Refactor `save_message`**:
   - Remove truncation from storage logic.
   - Add `message_type` field for categorization.
   - Store full content and metadata separately.

2. **Define `PlanningAgent`**:
   - Create prompt templates for plan generation.
   - Define Plan/Task structs and serialization.
   - Implement planning loop with structured output.

3. **Refactor `AgentLoop` to `AgentCoordinator`**:
   - Implement the Coordinator logic.
   - Add `run_planning_phase(user_message) -> Plan`.
   - Add `run_execution_phase(plan)`.
   - Integrate Planning Agent.
   - Implement Task Dispatcher with dependency resolution.

4. **Frontend Updates**:
   - Handle `Plan` events and display task list.
   - Real-time task status updates.
   - Collapsible task details showing execution logs.
   - Display task dependencies as a graph or list.

### Agent Coordinator Functions

```rust
impl AgentCoordinator {
    // Create execution plan
    async fn run_planning_phase(&mut self, user_message: &str) -> Result<Plan>;

    // Execute plan tasks
    async fn run_execution_phase(&mut self, plan: &Plan) -> Result<()>;

    // Dispatch single task to worker
    async fn execute_task(&mut self, task: &TaskSpec, context: &ExecutionContext) -> Result<TaskResult>;

    // Check if task dependencies are met
    fn can_execute_task(&self, task: &TaskSpec, plan: &Plan) -> bool;

    // Find all tasks ready for execution
    fn get_ready_tasks(&self, plan: &Plan) -> Vec<&TaskSpec>;

    // Update plan with task results
    fn update_plan(&mut self, task_id: &str, result: TaskResult);
}
```

---

## Interaction Model

### Frontend

- **Chat Stream**: Renders text and "Thinking" blocks separately (collapsible "Thinking" UI).
- **Intervention UI**: "Pending Approval" cards for permission requests.
- **Artifacts**: Display diffs or file changes summary after tool execution.
- **Plan View**:
  - Display tasks as an interactive checklist.
  - Real-time updates to task status.
  - Click on task to see detailed execution logs and tool calls.
  - Dependency visualization (optional).

### Backend Structure

```
src-tauri/src/
  agents/
    mod.rs           # Agent trait and common logic
    coordinator.rs   # NEW: Coordinator/Engine implementation
    planner.rs       # NEW: Planning Agent
    executor.rs      # Executing Agent (refactored from existing)
    processor.rs     # Stream & Logic Processor

  mcp/               # Model Context Protocol
    client.rs        # MCP Client implementation
    transport.rs     # stdio/SSE transport
    registry.rs      # Tool registry

  permissions/
    mod.rs           # Permission logic
    rules.rs         # Permission rules engine

  snapshots/         # State management
    mod.rs           # Snapshot/checkpoint logic

  models/
    mod.rs           # Data models
    execution.rs     # NEW: ExecutionJob, Plan, Task models
    execution_state.rs # NEW: State management for jobs

  commands/
    agents.rs        # Tauri commands for agents
    chat.rs          # Chat-specific commands

  events.rs          # Event definitions for frontend
  database.rs        # Database connection
  schema.rs          # Diesel schema
```

---

## Design Decisions & Trade-offs

### Parallel vs Sequential Execution

**Decision**: Start with sequential execution, design for parallel.

**Rationale**:
- **Safety**: Prevents race conditions on file system.
- **Simplicity**: Easier to debug and reason about.
- **Future-proof**: Structure supports parallelism when needed.
- **Opt-in**: Can enable parallel execution for read-only or independent tasks.

### Context Window Management

**Challenge**: Passing full history to every worker might exceed context limits.

**Solution**:
- Workers receive:
  - Current task description (full detail)
  - Summary of previous relevant task results
  - Minimal chat history (not full conversation)
- Full history available on-demand if worker requests it.
- Use `optimize_history_by_tokens` for context management.

### Message Storage

**Decision**: Store full messages, truncate only for LLM context.

**Rationale**:
- Users should be able to review full execution history.
- Storage is cheap, context window is limited.
- Truncation logic separated from storage logic.

### Frontend Display

**Decision**: Separate UI for Plan and Execution.

**Implementation**:
- Scratchpad/Plan panel: Shows task checklist with real-time updates.
- Chat area: Shows user messages and final responses.
- Details view: Click task to see tool execution logs.

---

## Questions & Future Considerations

### Open Questions

1. **Task Granularity**: How fine-grained should tasks be?
   - Too granular: Overhead from planning
   - Too coarse: Defeats the purpose of parallel execution

2. **Error Recovery**: What happens when a task fails?
   - Retry automatically?
   - Ask user for guidance?
   - Replan from failure point?

3. **Resource Limits**: How to prevent runaway tasks?
   - Time limits per task?
   - Token/cost limits per plan?

### Future Enhancements

1. **Task Templates**: Common task patterns (e.g., "read file", "search codebase").
2. **Plan Optimization**: Reorder tasks for optimal execution.
3. **Human-in-the-Loop**: User can modify plan before execution.
4. **Agent Learning**: Remember successful patterns for similar requests.
5. **Distributed Execution**: Run tasks on different machines/containers.

---

## References

- **MCP Specification**: https://modelcontextprotocol.io
- **OpenCode Architecture**: Inspiration for agent-tool interaction model
- **Tauri Event System**: Real-time frontend-backend communication
- **Diesel ORM**: Database schema and migrations

---

**Last Updated**: 2026-01-24
**Status**: Design phase - Implementation in progress
