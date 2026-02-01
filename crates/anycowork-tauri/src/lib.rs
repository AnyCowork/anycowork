//! Tauri adapter for AnyCowork core library
//!
//! This crate provides Tauri-specific implementations of the core abstractions:
//! - `TauriPermissionHandler` - Routes permission requests to Tauri window
//! - `TauriEventBridge` - Bridges core events to Tauri event system

mod events;
mod permissions;

pub use events::{spawn_event_bridge, TauriEventBridge};
pub use permissions::TauriPermissionHandler;

// Re-export core types for convenience
pub use anycowork_core::prelude::*;
pub use anycowork_core::{
    agent, config, events as core_events, permissions as core_permissions, sandbox, skills, tools,
};
