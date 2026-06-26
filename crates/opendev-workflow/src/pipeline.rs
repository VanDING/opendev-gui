use crate::types::{AgentSpawner, WorkflowDefinition, WorkflowPhase};

#[derive(Debug)]
pub struct PipelineResult {
    pub phase_results: Vec<Vec<String>>,
    pub total_items: usize,
    pub completed_items: usize,
}

pub async fn execute_pipeline(
    definition: &WorkflowDefinition,
    spawner: &dyn AgentSpawner,
) -> Result<PipelineResult, String> {
    let total = definition.phases.first().map(|p| p.items.len()).unwrap_or(0);
    let mut completed = 0;
    let mut phase_results = Vec::new();

    for item_index in 0..total {
        let mut prev_result = None;
        let mut item_results = Vec::new();
        for phase in &definition.phases {
            let item = phase.items.get(item_index).map(|s| s.as_str()).unwrap_or("");
            let result =
                execute_phase_on_item(phase, item, prev_result.as_deref(), spawner).await?;
            item_results.push(result.clone());
            prev_result = Some(result);
        }
        completed += 1;
        phase_results.push(item_results);
    }

    Ok(PipelineResult { phase_results, total_items: total, completed_items: completed })
}

async fn execute_phase_on_item(
    phase: &WorkflowPhase,
    item: &str,
    previous_result: Option<&str>,
    spawner: &dyn AgentSpawner,
) -> Result<String, String> {
    let prompt = phase.prompt_template.replace("{item}", item);
    let prompt = if let Some(prev) = previous_result {
        format!("{prompt}\n\nPrevious result: {prev}")
    } else {
        prompt
    };
    spawner.spawn_single(&phase.agent_type, &prompt).await
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockSpawner;

    #[async_trait::async_trait]
    impl AgentSpawner for MockSpawner {
        async fn spawn_single(&self, agent: &str, prompt: &str) -> Result<String, String> {
            Ok(format!("[{agent}] {prompt}"))
        }

        async fn spawn_many(
            &self,
            agent: &str,
            items: &[String],
            template: &str,
        ) -> Result<Vec<String>, String> {
            let mut results = Vec::new();
            for item in items {
                let prompt = template.replace("{item}", item);
                results.push(format!("[{agent}] {prompt}"));
            }
            Ok(results)
        }
    }

    #[tokio::test]
    async fn pipeline_handles_multiple_items() {
        let definition = WorkflowDefinition {
            name: "test".into(),
            description: "test".into(),
            phases: vec![
                WorkflowPhase {
                    title: "explore".into(),
                    mode: "pipeline".into(),
                    agent_type: "Explore".into(),
                    max_concurrency: 1,
                    items: vec!["a.rs".into(), "b.rs".into()],
                    prompt_template: "Explore {item}".into(),
                    prompt: None,
                },
                WorkflowPhase {
                    title: "verify".into(),
                    mode: "pipeline".into(),
                    agent_type: "Verify".into(),
                    max_concurrency: 1,
                    items: vec!["a.rs".into(), "b.rs".into()],
                    prompt_template: "Verify {item}".into(),
                    prompt: None,
                },
            ],
            r#loop: None,
        };

        let result = execute_pipeline(&definition, &MockSpawner).await.unwrap();
        assert_eq!(result.total_items, 2);
        assert_eq!(result.completed_items, 2);
        assert_eq!(result.phase_results.len(), 2);
        assert_eq!(result.phase_results[0].len(), 2);
        assert!(
            result.phase_results[0][0].contains("[Explore]")
                && result.phase_results[0][0].contains("a.rs")
        );
        assert!(
            result.phase_results[0][1].contains("[Verify]")
                && result.phase_results[0][1].contains("a.rs")
        );
        assert!(result.phase_results[1][1].contains("Previous result:"));
    }
}
