use crate::types::{AgentSpawner, WorkflowDefinition};

pub async fn execute_barrier(
    definition: &WorkflowDefinition,
    spawner: &dyn AgentSpawner,
) -> Result<Vec<String>, String> {
    let mut results = Vec::new();
    for phase in &definition.phases {
        let phase_results =
            spawner.spawn_many(&phase.agent_type, &phase.items, &phase.prompt_template).await?;
        results.extend(phase_results);
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::WorkflowPhase;

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
    async fn barrier_executes_all_phases() {
        let definition = WorkflowDefinition {
            name: "test".into(),
            description: "test".into(),
            phases: vec![
                WorkflowPhase {
                    title: "phase1".into(),
                    mode: "barrier".into(),
                    agent_type: "Explore".into(),
                    max_concurrency: 1,
                    items: vec!["a.rs".into(), "b.rs".into()],
                    prompt_template: "Explore {item}".into(),
                    prompt: None,
                },
                WorkflowPhase {
                    title: "phase2".into(),
                    mode: "barrier".into(),
                    agent_type: "Verify".into(),
                    max_concurrency: 1,
                    items: vec!["a.rs".into(), "b.rs".into()],
                    prompt_template: "Verify {item}".into(),
                    prompt: None,
                },
            ],
            r#loop: None,
        };

        let results = execute_barrier(&definition, &MockSpawner).await.unwrap();
        assert_eq!(results.len(), 4);
        assert!(results[0].contains("[Explore]"));
        assert!(results[2].contains("[Verify]"));
    }
}
