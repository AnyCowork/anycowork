//! AnyCowork Core Library
//!
//! A platform-independent Rust library for building AI agents with:
//! - Multi-agent coordination
//! - Sandboxed tool execution
//! - Permission/approval system
//! - Skill system for extensibility
//! - Platform-agnostic event system
//!
//! This library is designed to be used by platform adapters (Tauri, CLI, Server, etc.)

pub mod agent;
pub mod config;
pub mod events;
pub mod permissions;
pub mod sandbox;
pub mod skills;
pub mod tools;

// Re-export commonly used types
pub use agent::{AgentConfig, AgentCoordinator};
pub use config::CoreConfig;
pub use events::{AgentEvent, EventChannel, StepStatus, ToolStep};
pub use permissions::{PermissionHandler, PermissionManager, PermissionRequest, PermissionType};
pub use sandbox::{ExecutionResult, Sandbox, SandboxConfig};
pub use tools::{AnyCoworkTool, ToolError};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::agent::{AgentConfig, AgentCoordinator};
    pub use crate::events::{AgentEvent, EventChannel, StepStatus, ToolStep};
    pub use crate::permissions::{
        PermissionHandler, PermissionManager, PermissionRequest, PermissionType,
    };
    pub use crate::sandbox::{ExecutionResult, Sandbox, SandboxConfig};
    pub use crate::tools::{AnyCoworkTool, ToolError};
    pub use rig::tool::Tool;
}
