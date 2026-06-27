//! Application Services — business logic entry points.
//!
//! These services coordinate domain operations and call infrastructure.
//! They do NOT depend on any interface framework (tauri, axum, clap).
//! They do NOT know about Desktop, HTTP, CLI, or any platform.

pub mod config_service;
pub mod session_service;
pub mod chat_service;
pub mod workflow_service;
pub mod mcp_service;
pub mod skill_service;
pub mod file_service;
pub mod system_service;

pub use config_service::ConfigService;
pub use session_service::SessionService;
pub use chat_service::ChatService;
pub use workflow_service::WorkflowService;
pub use mcp_service::MCPService;
pub use skill_service::SkillService;
pub use file_service::FileService;
pub use system_service::SystemService;

/// Aggregated application services for dependency injection.
pub struct AppServices {
    pub config: ConfigService,
    pub session: SessionService,
    pub chat: ChatService,
    pub workflow: WorkflowService,
    pub mcp: MCPService,
    pub skill: SkillService,
    pub file: FileService,
    pub system: SystemService,
}
