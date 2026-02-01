//! Platform-agnostic permission system

mod handler;
mod manager;
mod scope;
mod types;

pub use handler::{AllowAllHandler, DenyAllHandler, PermissionHandler};
pub use manager::PermissionManager;
pub use scope::{ScopeEnforcer, ScopeType};
pub use types::{PermissionRequest, PermissionResponse, PermissionType};
