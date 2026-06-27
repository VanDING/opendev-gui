# How to Add a New Tool

This guide describes how to add a new tool (e.g., a database query tool, a cloud
API tool) that the agent can call during its ReAct loop.

## Overview

Tools implement the `BaseTool` trait from `opendev-tools-core`. The trait has 20+
methods with sensible defaults — you only need to override 4:

- `name()` — tool identifier (used by LLM to call it)
- `description()` — LLM-facing description (prompt engineering matters here)
- `parameter_schema()` — JSON Schema for parameters
- `execute()` — execution logic

## Steps

### 1. Create the tool module

Create a directory under `crates/opendev-tools-impl/src/{tool_name}/`:

```
crates/opendev-tools-impl/src/{tool_name}/
├── mod.rs          — Tool struct + BaseTool implementation
└── {tool_name}_tests.rs  — Tests (optional but encouraged)
```

### 2. Implement BaseTool

```rust
use opendev_tools_core::{BaseTool, ToolError, ToolResult};
use serde_json::Value;

pub struct MyNewTool;

#[async_trait]
impl BaseTool for MyNewTool {
    fn name(&self) -> &'static str {
        "my_new_tool"
    }

    fn description(&self) -> &'static str {
        "Description that tells the LLM what this tool does and when to use it"
    }

    fn parameter_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "input": {
                    "type": "string",
                    "description": "The input parameter"
                }
            },
            "required": ["input"]
        })
    }

    async fn execute(&self, params: Value) -> ToolResult<Value> {
        let input = params.get("input")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("input is required".into()))?;

        // Your tool logic here
        let result = do_something(input).await?;

        Ok(serde_json::json!({
            "result": result
        }))
    }
}
```

### 3. Register the tool

Edit `crates/opendev-tools-impl/src/lib.rs`:

```rust
mod my_new_tool;
use my_new_tool::MyNewTool;

// In the function that builds the ToolRegistry:
registry.register(MyNewTool);
```

### 4. Add to prompt system (optional)

If the tool needs special prompt instructions, add them in
`crates/opendev-agents/src/prompts/`.

### 5. Test

```bash
cargo test -p opendev-tools-impl
cargo check --workspace
```

## When to Override Additional Methods

The `BaseTool` trait provides default implementations for these. Override them
only when needed:

| Method | When to Override |
|---|---|
| `permission_required()` | Tool needs user approval before execution |
| `cache_ttl()` | Results are cacheable |
| `parallel_support()` | Tool can run in parallel with other tools |
| `timeout_seconds()` | Default timeout is insufficient |
| `format_output()` | Custom result formatting needed |

## Tool Design Principles

1. **Single responsibility** — each tool does one thing well.
2. **Descriptive name** — the name tells the LLM when to use it.
3. **Clear parameter schema** — use JSON Schema with descriptions for each parameter.
4. **Good error messages** — return helpful errors that tell the LLM what went wrong.
5. **Fast by default** — if the tool might be slow, document it in the description.

## Existing Tools (Reference)

See `crates/opendev-tools-impl/src/` for existing implementations:
- `file_read/` — read files
- `file_edit/` — edit files
- `file_write/` — write files
- `bash/` — shell execution
- `web_fetch/` — HTTP fetching
- `web_search/` — web search
- `memory/` — memory recall
- And many more...
