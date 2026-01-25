pub mod agent;
pub mod page;
pub mod session;
pub mod skill;
pub mod telegram;

// Re-export commonly used types
pub use agent::{AIConfigDto, Agent, AgentCharacteristicsDto, AgentDto, AgentUpdateDto, NewAgent};
pub use page::{
    Attachment, Block, NewAttachment, NewBlock, NewPage, Page, UpdateBlock, UpdatePage,
};
pub use session::{Message, NewMessage, NewSession, Session, UpdateSession};
pub use skill::{
    AgentSkill, AgentSkillAssignment, NewAgentSkill, NewAgentSkillAssignment, UpdateAgentSkill,
};
pub use telegram::{NewTelegramConfig, TelegramConfig, UpdateTelegramConfig};

pub mod execution;
pub use execution::{Plan, TaskSpec};
pub mod execution_state;
pub use execution_state::{PlanUpdate, TaskState};
