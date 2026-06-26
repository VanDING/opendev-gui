use std::collections::HashMap;

use opendev_tools_core::{BaseTool, ToolCategory, ToolContext, ToolResult};

#[derive(Debug)]
pub struct RunWorkflowTool;

#[async_trait::async_trait]
impl BaseTool for RunWorkflowTool {
    fn name(&self) -> &str {
        "RunWorkflow"
    }

    fn description(&self) -> &str {
        "Execute a multi-phase workflow with sub-agents. \
         Use pipeline mode for independent items, barrier mode for coordinated phases. \
         Supports loop-until-count, loop-until-dry, and loop-until-budget patterns."
    }

    fn parameter_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "workflow_json": {
                    "type": "string",
                    "description": "JSON-encoded WorkflowDefinition with name, description, phases, and optional loop config"
                }
            },
            "required": ["workflow_json"]
        })
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Session
    }

    async fn execute(
        &self,
        _args: HashMap<String, serde_json::Value>,
        _ctx: &ToolContext,
    ) -> ToolResult {
        ToolResult::fail(
            "RunWorkflow tool is not yet fully wired in this build. \
             To execute workflows, use spawn_subagent with appropriate agent types.",
        )
    }
}
