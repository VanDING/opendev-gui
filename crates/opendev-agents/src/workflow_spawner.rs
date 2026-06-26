use std::sync::Arc;

use opendev_http::AdaptedClient;
use opendev_runtime::{Mailbox, SessionDebugLogger, ToolApprovalSender};
use opendev_tools_core::ToolRegistry;
use opendev_workflow::AgentSpawner;
use tokio_util::sync::CancellationToken;

use crate::subagents::{NoopProgressCallback, SubagentManager};
use crate::traits::TaskMonitor;

pub struct OpenDevAgentSpawner {
    manager: Arc<SubagentManager>,
    tool_registry: Arc<ToolRegistry>,
    http_client: Arc<AdaptedClient>,
    working_dir: String,
    parent_model: String,
    parent_max_tokens: u64,
    parent_reasoning_effort: Option<String>,
}

impl OpenDevAgentSpawner {
    pub fn new(
        manager: Arc<SubagentManager>,
        tool_registry: Arc<ToolRegistry>,
        http_client: Arc<AdaptedClient>,
        working_dir: &str,
        parent_model: &str,
        parent_max_tokens: u64,
    ) -> Self {
        Self {
            manager,
            tool_registry,
            http_client,
            working_dir: working_dir.to_string(),
            parent_model: parent_model.to_string(),
            parent_max_tokens,
            parent_reasoning_effort: None,
        }
    }

    pub fn with_reasoning_effort(mut self, effort: String) -> Self {
        self.parent_reasoning_effort = Some(effort);
        self
    }
}

#[async_trait::async_trait]
impl AgentSpawner for OpenDevAgentSpawner {
    async fn spawn_single(&self, agent_type: &str, prompt: &str) -> Result<String, String> {
        let result = self
            .manager
            .spawn(
                agent_type,
                prompt,
                &self.parent_model,
                Arc::clone(&self.tool_registry),
                Arc::clone(&self.http_client),
                &self.working_dir,
                Arc::new(NoopProgressCallback),
                None::<&dyn TaskMonitor>,
                None::<&ToolApprovalSender>,
                self.parent_max_tokens,
                self.parent_reasoning_effort.clone(),
                None::<CancellationToken>,
                None::<&SessionDebugLogger>,
                None::<&str>,
                None::<&Mailbox>,
            )
            .await
            .map_err(|e| format!("spawn failed: {}", e))?;
        Ok(result.agent_result.content)
    }

    async fn spawn_many(
        &self,
        agent_type: &str,
        items: &[String],
        prompt_template: &str,
    ) -> Result<Vec<String>, String> {
        let mut results = Vec::new();
        for item in items {
            let prompt = prompt_template.replace("{item}", item);
            results.push(self.spawn_single(agent_type, &prompt).await?);
        }
        Ok(results)
    }
}
