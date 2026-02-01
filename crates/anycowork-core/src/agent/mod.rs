//! Agent system for AI coordination

mod config;
mod coordinator;
pub mod hook;

pub use config::AgentConfig;
pub use coordinator::AgentCoordinator;
pub use hook::AnyCoworkHook;
