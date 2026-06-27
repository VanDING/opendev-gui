# Testing Strategy

## Principles

1. **Every behavioral change is backed by tests.**
2. **Unit tests for logic, integration tests for boundaries.**
3. **CI gates prevent regression** (partially — some gates are report-only for now).
4. **Test files live alongside source code** as `module_tests.rs` or in a `tests/` directory
   for integration tests.

## Test Organization

| Type | Location | Count (approx) | Run Command |
|---|---|---|---|
| Unit tests | `*_tests.rs` next to source | 3,000+ | `cargo test --workspace --lib` |
| Integration tests | `tests/` per crate | ~12 test files | `cargo test --workspace --test '*'` |
| Benchmarks | `benches/` | 1 (agents) | `cargo bench` |

### Naming Convention

- Unit test files: `{module}_tests.rs` (e.g., `event_store_tests.rs`).
- Integration test files: `tests/integration.rs` within the crate.
- Test functions: descriptive names using `_` separator.
- Test modules: `#[cfg(test)] mod tests { ... }` within the file.

## What to Test

### Must Test
- Public API of every crate (every `pub fn` with non-trivial logic).
- Error paths (network failures, file not found, permission denied).
- Edge cases (empty inputs, maximum sizes, boundary values).
- Data serialization/deserialization roundtrips.
- State transitions (session status, tool execution states).

### Should Test
- Property-based testing for parsing/validation logic (when deterministic tests
  are insufficient — previous `proptest` usage was evaluated and removed in v0.1.9).
- Concurrent access patterns for shared state.
- Cross-crate integration paths.

### Don't Test
- LLM provider responses (test the adapter mapping, not the provider's API).
- Trivial getters/setters.
- External crate behavior (assume `rusqlite`, `reqwest` etc. work correctly).

## Current CI Test Status

- `cargo test --workspace --all-features` passes (~1,068 tests, 1 known OAuth network
  test failure that is non-blocking).
- CI test job is `continue-on-error: true` due to known flaky tests on Linux.
- All contributors should run `cargo test --workspace --lib` locally before pushing.

## Running Tests

```bash
# All library tests
cargo test --workspace --lib

# All integration tests
cargo test --workspace --test '*'

# Single crate
cargo test -p opendev-history

# Single test
cargo test -p opendev-history -- event_store_tests
```
