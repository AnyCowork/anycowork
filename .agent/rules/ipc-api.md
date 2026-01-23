# Tauri IPC API Reference

## Commands

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
