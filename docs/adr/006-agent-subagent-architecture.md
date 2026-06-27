# ADR-006: Agent/Subagent Architecture

## Status

Accepted 2026-06-24

## Context

The agent needs to handle complex multi-step tasks. A single ReAct loop is limited:
it processes one LLM call at a time. For parallel tasks (e.g., edit multiple files,
search multiple sources), the agent needs to spawn child agents that run independently.

The design must support:
- Main agent with its own ReAct loop
- Subagents spawned for parallel work
- Isolated context per subagent
- Coordination through shared state
- Progress reporting from subagents

## Decision

The agent architecture uses a hierarchy: one `MainAgent` with a `SubagentManager` that
spawns and manages subagents. Each subagent runs its own `ReactLoop` in an isolated
context.

Key components:
- `MainAgent` (`opendev-agents/src/main_agent.rs`) — the top-level agent.
- `ReactLoop` (`opendev-agents/src/react_loop/`) — the core loop that calls LLM,
  dispatches tools, and evaluates whether to continue.
- `SubagentManager` (`opendev-agents/src/subagents/`) — spawns, monitors, and
  coordinates subagents.
- `SubagentRunner` — runs a subagent's ReactLoop in isolation.
- Coordination happens through shared state (TODO lists, file locks) and message passing.
- The `opendev-workflow` crate extends this with pipeline/barrier/loop patterns.

## Alternatives

- **Single monolithic agent** — simpler but cannot parallelize. Any multi-file operation
  becomes sequential.
- **External agent orchestration** — e.g., using an external message queue or workflow
  engine. Adds deployment complexity for a desktop app.
- **Actor model (actix/ractor)** — would provide structured concurrency but adds a
  framework dependency and learning curve.

## Consequences

- Subagents can work in parallel on independent tasks (edit multiple files, search
  multiple sources).
- Each subagent has its own token budget and context window.
- Coordination complexity increases — shared state needs locking (file locks, TODO
  list synchronization).
- Subagent errors need propagation back to the main agent.
- The subagent API (`SubAgentSpec`, `SubagentRunResult`) must remain stable as it's
  used by both the main agent and the workflow engine.

## References

- `crates/opendev-agents/src/main_agent.rs`
- `crates/opendev-agents/src/react_loop/`
- `crates/opendev-agents/src/subagents/`
- `crates/opendev-agents/src/traits.rs` — `BaseAgent` trait
- `crates/opendev-workflow/` — workflow pipeline/barrier/loop
