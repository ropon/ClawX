// ClawX application constants

pub const OPENCLAW_GATEWAY_PORT: u16 = 18789;

// Store file names (without .json extension)
pub const AGENT_STORE_NAME: &str = "clawx-agents";
pub const KNOWLEDGE_STORE_NAME: &str = "clawx-knowledge";
pub const WORKFLOW_STORE_NAME: &str = "clawx-workflows";

// Default agent values
pub const DEFAULT_AGENT_ID: &str = "default";
pub const DEFAULT_AGENT_NAME: &str = "Assistant";
pub const DEFAULT_AGENT_AVATAR: &str = "\u{1F916}"; // 🤖
pub const DEFAULT_AGENT_DESCRIPTION: &str = "Default AI assistant";
pub const DEFAULT_TEMPERATURE: f64 = 0.7;

// Knowledge defaults
pub const DEFAULT_CHUNK_SIZE: usize = 500;
pub const DEFAULT_CHUNK_OVERLAP: usize = 100;

// Workflow defaults
pub const DEFAULT_WORKFLOW_ICON: &str = "\u{1F504}"; // 🔄
