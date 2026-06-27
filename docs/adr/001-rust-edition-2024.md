# ADR-001: Rust Edition 2024

## Status

Accepted 2026-06-24

## Context

The Rust 2024 edition introduced new language features and changed some default behaviors
(lifetime elision, `impl Trait` capture rules, `unsafe` block semantics). The project needed
to choose which edition to target. Both edition 2021 and 2024 are stable options.

The decision was made at project inception to target Edition 2024 to access the latest
language features and future-proof the codebase.

## Decision

The workspace and every crate uses `edition = "2024"`. The `rust-version` is set to `"1.94"`
in the workspace manifest.

`rustfmt.toml` also uses `edition = "2024"` along with `max_width = 100` and
`use_small_heuristics = "Max"`.

## Alternatives

- **Edition 2021** — wider compatibility but misses `unsafe` block changes and lifetime
  elision improvements.
- **No explicit edition** — defaults to 2015, which would block modern idioms.

## Consequences

- Contributors need Rust 1.85+ (edition 2024 stabilized in 1.85).
- The project can use `unsafe` blocks with the new stricter rules.
- `impl Trait` in return position behaves predictably with the new capture rules.
- `cargo fix --edition` may be needed if the edition feature set changes.

## References

- Rust Edition 2024 Guide: https://doc.rust-lang.org/nightly/edition-guide/rust-2024/
- `rustfmt.toml` at project root
- `Cargo.toml` workspace `[workspace.package]` section
