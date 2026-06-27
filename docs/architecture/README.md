# Architecture Documentation

This directory documents the current architecture of OpenDev Desktop.
The high-level overview lives in `ARCHITECTURE.md` at the project root.
These sub-documents provide depth on specific architectural concerns.

## Documents

| Document | What It Covers | When to Update |
|---|---|---|
| `crate-layering.md` | Crate dependency graph, layering rules, layer violation tracking | When crate dependencies change |
| `data-flow.md` | Core data flow: user input → agent → LLM → tools → response | When the agent loop changes |
| `frontend.md` | TypeScript/React frontend architecture, component tree, state management | When frontend architecture changes |
| `security-model.md` | Security architecture: authentication, permissions, secrets, SSRF | When security mechanisms change |

## Architecture Review Cadence

- **Each release cycle**: Review crate layering for violations.
- **Each new crate**: Must be placed in the correct layer and reviewed for dependency
  direction.
- **Each security mechanism change**: Security model doc must be updated.

## Relationship to Root ARCHITECTURE.md

`ARCHITECTURE.md` provides the high-level overview (4 layers, core data flow, crate
dependency graph, security model summary, performance characteristics, testing strategy).
These sub-docs provide the detail that would make `ARCHITECTURE.md` unreadable.
