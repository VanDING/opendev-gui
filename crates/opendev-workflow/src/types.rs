use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub name: String,
    pub description: String,
    pub phases: Vec<WorkflowPhase>,
    #[serde(default)]
    pub r#loop: Option<LoopConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowPhase {
    pub title: String,
    #[serde(default = "default_mode")]
    pub mode: String,
    #[serde(default = "default_agent_type")]
    pub agent_type: String,
    #[serde(default = "default_concurrency")]
    pub max_concurrency: usize,
    #[serde(default)]
    pub items: Vec<String>,
    pub prompt_template: String,
    #[serde(default)]
    pub prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopConfig {
    pub r#type: String,
    #[serde(default)]
    pub max_dry_rounds: u32,
    #[serde(default)]
    pub max_iterations: u32,
    #[serde(default)]
    pub target_count: u32,
}

#[async_trait::async_trait]
pub trait AgentSpawner: Send + Sync {
    async fn spawn_single(&self, agent_type: &str, prompt: &str) -> Result<String, String>;

    async fn spawn_many(
        &self,
        agent_type: &str,
        items: &[String],
        prompt_template: &str,
    ) -> Result<Vec<String>, String>;
}

fn default_mode() -> String {
    "pipeline".into()
}

fn default_agent_type() -> String {
    "explore".into()
}

fn default_concurrency() -> usize {
    10
}
