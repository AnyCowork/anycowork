pub mod agents;
pub mod mail;
pub mod mcp;
pub mod pages;
pub mod sessions;
pub mod skills;
pub mod telegram;
pub mod window;

// Re-export commands for easy registration
pub use agents::*;
pub use mail::*;
pub use mcp::*;
pub use pages::*;
pub use sessions::*;
pub use skills::*;
pub use telegram::*;
pub mod transcribe;
pub use transcribe::*;

pub mod tasks;
pub use tasks::*;

pub mod voice;
pub use voice::*;

pub mod settings;
pub use settings::*;
