# ADR-005: BaseTool Trait with Sensible Defaults

## Status

Accepted 2026-06-24

## Context

The agent calls tools (file editing, shell commands, web search, etc.) during its ReAct
loop. Each tool needs: name, description, parameter schema, execution logic, and various
metadata. With 20+ tools, the trait design must minimize boilerplate for common cases
while allowing full customization when needed.

## Decision

The `BaseTool` trait in `opendev-tools-core` provides 20+ methods with sensible defaults.
New tools typically override only 4 methods:
- `name()` — tool identifier
- `description()` — LLM-facing description
- `parameter_schema()` — JSON Schema for parameters
- `execute()` — execution logic

The trait provides default implementations for:
- Error handling and formatting
- Permission checking
- Caching
- Parallel execution support
- Progress reporting
- Approval flow
- Timeout management
- Result formatting

Tools are registered in `ToolRegistry` at startup. The registry supports:
- Lookup by name
- Listing all tools for prompt construction
- Permission queries per tool

## Alternatives

- **Trait with minimum defaults** — every tool implements 10+ methods, lots of duplication.
- **Enum-based dispatch** — adding a tool means adding a variant to the enum and updating
  all match arms; violates Open-Closed Principle.
- **Dynamic dispatch via Box<dyn>** — the chosen approach, using `Box<dyn BaseTool>` in the
  registry. Trade-off: a small vtable cost per call.

## Consequences

- Adding a new tool is a well-defined, low-boilerplate task: implement 4 methods.
- The trait is large (~20 methods) but stable — new methods added with defaults don't
  break existing tools.
- Tool implementations live in `opendev-tools-impl` and depend on `opendev-tools-core`.
  (Known debt: `opendev-tools-impl` also depends on `opendev-agents`, creating a layer
  violation — see ARCHITECTURE.md.)
- The trait provides natural extension points for cross-cutting concerns (permissions,
  caching, progress) without each tool implementing them.

## References

- `crates/opendev-tools-core/src/traits.rs` — `BaseTool` trait
- `crates/opendev-tools-core/src/registry/` — `ToolRegistry`
- `crates/opendev-tools-impl/src/lib.rs` — tool implementations
- `ARCHITECTURE.md` — Known Architecture Debt #1
