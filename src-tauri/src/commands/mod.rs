pub mod agents;
pub mod pages;
pub mod sessions;
pub mod skills;
pub mod telegram;
pub mod window;

// Re-export commands for easy registration
pub use agents::*;
pub use pages::*;
pub use sessions::*;
pub use skills::*;
pub use telegram::*;
pub use window::*;
