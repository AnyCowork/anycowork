pub mod agents;
pub mod sessions;
pub mod telegram;
pub mod pages;
pub mod skills;
pub mod window;

// Re-export commands for easy registration
pub use agents::*;
pub use sessions::*;
pub use telegram::*;
pub use pages::*;
pub use skills::*;
pub use window::*;
