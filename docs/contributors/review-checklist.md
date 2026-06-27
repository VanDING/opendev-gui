# Code Review Checklist

## Architecture & Design

- [ ] Does the change align with the Constitution principles? (`docs/constitution.md`)
- [ ] Does the change respect the layer rules? (`docs/architecture/crate-layering.md`)
  - Interfaces → Orchestration → Domain → Infrastructure
  - No upward dependencies
- [ ] Are new crates justified? Is there an ADR?
- [ ] Does the change follow existing patterns (trait-based abstraction, registry,
  event sourcing, composition root)?

## Correctness

- [ ] Are error cases handled? (network errors, file not found, permission denied)
- [ ] Are edge cases covered? (empty inputs, max sizes, concurrent access)
- [ ] Is there Mutex poison recovery where appropriate?
- [ ] Is `spawn_blocking` used for file I/O and CPU-heavy operations?
- [ ] Are `unsafe` blocks justified with `// SAFETY:` comments?

## Testing

- [ ] Are there tests for new functionality?
- [ ] Are there regression tests for bug fixes?
- [ ] Do existing tests still pass?
- [ ] Are error paths tested, not just happy paths?

## Security

- [ ] Are API keys/credentials handled securely? (no logging, no exposure in errors)
- [ ] Are `unsafe` blocks necessary and correct?
- [ ] Are file paths validated? (path traversal prevention)
- [ ] Is SSRF protection in place for HTTP-fetching tools?
- [ ] Are new dependencies reviewed for license and security?

## Documentation

- [ ] Are public API items documented with doc comments?
- [ ] Are breaking changes documented?
- [ ] Do affected architecture docs need updating?
- [ ] Does a how-to guide need updating/creating? (especially for new tools,
  providers, or memory backends)

## Code Style

- [ ] Does `cargo fmt` pass?
- [ ] No new clippy warnings (pre-existing warnings are tracked, don't add more)
- [ ] No `unwrap()` or `expect()` in library code (acceptable in binaries with
  a clear message)
- [ ] No `todo!()` or `unimplemented!()` in production code paths

## Performance

- [ ] Are hot-path regexes compiled with `LazyLock`?
- [ ] Is blocking I/O wrapped in `spawn_blocking`?
- [ ] Are allocations in hot paths minimized?
- [ ] Is async used appropriately? (no async-over-async for synchronous work)
