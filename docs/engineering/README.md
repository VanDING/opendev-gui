# Engineering Standards

This directory documents engineering standards, tools, and practices currently in use.
These are **current truth** — they describe how the project is built and tested today.

## Documents

| Document | What It Covers | Stability |
|---|---|---|
| `testing.md` | Testing strategy, test organization, what to test | Stable |
| `ci-cd.md` | CI pipeline, local checks, release process | Stable |
| `coding-standards.md` | Rust style, naming, `rustfmt`, `clippy` policy | Stable |
| `error-handling.md` | Error type patterns, panic policy, recovery | Stable |
| `logging-observability.md` | Logging, tracing, observability setup | Active |

## Tooling Summary

| Tool | Purpose | Configuration |
|---|---|---|
| `cargo fmt` | Code formatting | `rustfmt.toml` |
| `cargo clippy` | Linting | `.cargo/audit.toml` |
| `cargo test` | Testing | — |
| `cargo audit` | Security advisory check | `.cargo/audit.toml` |
| `cargo deny` | License/dependency policy | `deny.toml` |
| `ts-rs` | TypeScript type generation | From Rust `#[derive(TS)]` |
