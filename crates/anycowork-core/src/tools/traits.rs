//! Tool trait definitions

use async_trait::async_trait;
use rig::tool::Tool;

/// Trait for AnyCowork tools that extends Rig Tool with additional metadata
#[async_trait]
pub trait AnyCoworkTool: Tool {
    /// Check if the result needs summarization
    fn needs_summarization(&self, _args: &Self::Args, _result: &Self::Output) -> bool {
        false
    }

    /// Check if this tool requires user approval explicitly before execution
    /// Note: PermissionManager checks happen during execution. This check is for
    /// pre-execution UI signals if needed.
    fn requires_approval(&self, _args: &Self::Args) -> bool {
        false
    }
}
