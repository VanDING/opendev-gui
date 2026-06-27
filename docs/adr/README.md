# Architecture Decision Records

This directory records architecture decisions made for OpenDev Desktop.

## What is an ADR?

An Architecture Decision Record (ADR) documents a significant architectural decision,
its context, alternatives considered, and consequences. ADRs are **immutable once accepted**
— if a decision is superseded, a new ADR is created that references the old one.

## When to Write an ADR

Write an ADR when:
- Adding a new crate to the workspace
- Changing the crate dependency graph
- Adopting a new external dependency
- Changing the agent execution model
- Changing the data persistence strategy
- Adding or removing a user interface
- Changing security architecture

## ADR Template

```markdown
# ADR-NNN: Title

## Status

[Proposed | Accepted | Superseded | Deprecated]

Accepted YYYY-MM-DD (if accepted)

## Context

What is the issue that motivated this decision? What forces are at play?
What constraints exist?

## Decision

What is the change that is being proposed or implemented?

## Alternatives

What other options were considered? Why were they rejected?

## Consequences

What becomes easier? What becomes harder? What trade-offs are made?

## References

- Related ADRs: ADR-NNN
- Related documents: [link]
```

## Index

| ADR | Title | Status | Date |
|---|---|---|---|
| [001](001-rust-edition-2024.md) | Rust Edition 2024 | Accepted | 2026-06-24 |
| [002](002-event-sourced-sessions.md) | Event-Sourced Session History | Accepted | 2026-06-24 |
| [003](003-provider-adapter-pattern.md) | Provider Adapter Pattern for LLMs | Accepted | 2026-06-24 |
| [004](004-workspace-monorepo.md) | Cargo Workspace Monorepo | Accepted | 2026-06-24 |
| [005](005-tool-trait-design.md) | BaseTool Trait with Sensible Defaults | Accepted | 2026-06-24 |
| [006](006-agent-subagent-architecture.md) | Agent/Subagent Architecture | Accepted | 2026-06-24 |
| [007](007-sqlite-persistence.md) | SQLite for Persistence | Accepted | 2026-06-24 |
