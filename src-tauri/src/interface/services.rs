//! Service injection — creates and wires Application Services with their dependencies.

use crate::application::AppServices;
use crate::application::chat_service::ChatService;
use crate::application::config_service::ConfigService;
use crate::application::file_service::FileService;
use crate::application::mcp_service::MCPService;
use crate::application::session_service::SessionService;
use crate::application::skill_service::SkillService;
use crate::application::system_service::SystemService;
use crate::application::workflow_service::WorkflowService;

use opendev_config::ModelRegistry;
use opendev_history::SessionManager;
use opendev_models::AppConfig;

/// Build all application services with the given dependencies.
pub fn build_services(
    config: AppConfig,
    session_manager: SessionManager,
    model_registry: ModelRegistry,
    working_dir: String,
) -> AppServices {
    let chat_config = config.clone();
    AppServices {
        config: ConfigService::new(config, model_registry),
        session: SessionService::new(session_manager, working_dir.clone()),
        chat: ChatService::new(
            opendev_history::SessionManager::new(
                std::path::PathBuf::from(&working_dir).join(".opendev").join("sessions"),
            )
            .unwrap_or_else(|_| {
                let paths =
                    opendev_config::Paths::new(Some(std::path::PathBuf::from(&working_dir)));
                SessionManager::new(paths.global_sessions_dir())
                    .expect("Failed to create session manager for chat service")
            }),
            chat_config,
        ),
        workflow: WorkflowService::new(),
        mcp: MCPService::new(working_dir.clone()),
        skill: SkillService::new(),
        file: FileService::new(working_dir.clone()),
        system: SystemService::new(working_dir),
    }
}
