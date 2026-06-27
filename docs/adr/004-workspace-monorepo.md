# ADR-004: Cargo Workspace Monorepo

## Status

Accepted 2026-06-24

## Context

The project spans multiple concerns: domain types, config, HTTP, agent orchestration,
tools, UI interfaces, and infrastructure. These could be organized as:
1. A single crate — simple but hard to maintain as it grows.
2. Multiple repos — clean separation but cross-cutting changes require multi-repo coordination.
3. A cargo workspace monorepo — single repo, multiple crates, shared dependency versions.

The project needs clear crate boundaries for testability and team scaling, while keeping
cross-crate refactoring practical.

## Decision

The project uses a Cargo workspace monorepo with 24 members (as of v0.1.9).
Shared dependencies are declared once in the root `Cargo.toml` under
`[workspace.dependencies]` and referenced with `workspace = true` in sub-crates.

Key rules:
- No git dependencies are allowed (enforced by `cargo-deny`).
- All crates use `version = "0.1.6"`, `edition.workspace = true`, `license.workspace = true`.
- The workspace `package.version` is "0.1.9" (the next release will reconcile this mismatch).
- Crate dependency graph is a DAG — no circular dependencies (enforced by `cargo check`).

## Alternatives

- **Single crate** — works at small scale but 20+ modules in one crate means compile
  times, unclear boundaries, and accidental coupling.
- **Multi-repo** — each crate in its own repo with version bumps for cross-cutting changes.
  Practical but slow for a project under active development.
- **Git submodules** — worse than multi-repo; submodule management overhead.

## Consequences

- `cargo test --workspace` tests all crates in one command.
- `cargo check --workspace` catches cross-crate compatibility issues.
- Cross-crate refactoring is a single PR.
- All crates share the same dependency versions — no version skew.
- Compile times are longer for `--workspace` operations, but individual crate compilation
  is fast when only that crate changes.

## References

- `Cargo.toml` at project root — `[workspace]` section
- `deny.toml` — `[sources]` section: `unknown-git = "deny"`
- `ARCHITECTURE.md` — crate dependency graph
