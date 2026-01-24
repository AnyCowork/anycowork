pub mod agent;
pub mod session;
pub mod telegram;
pub mod page;
pub mod skill;

// Re-export commonly used types
pub use agent::{Agent, NewAgent, AgentDto, AgentCharacteristicsDto, AIConfigDto, AgentUpdateDto};
pub use session::{Session, NewSession, UpdateSession, Message, NewMessage};
pub use telegram::{TelegramConfig, NewTelegramConfig, UpdateTelegramConfig};
pub use page::{Page, NewPage, UpdatePage, Block, NewBlock, UpdateBlock, Attachment, NewAttachment};
pub use skill::{AgentSkill, NewAgentSkill, UpdateAgentSkill, AgentSkillAssignment, NewAgentSkillAssignment};

pub mod execution;
pub use execution::{Plan, TaskSpec};
pub mod execution_state;
pub use execution_state::{PlanUpdate, TaskState};
