# Pull Request Guide

## Before Creating a PR

1. **Read the Constitution** — `docs/constitution.md`. Make sure your change aligns
   with the project's design principles.
2. **Check for related ADRs** — `docs/adr/`. If your change contradicts an existing
   ADR, you need a new ADR.
3. **Run the full pre-commit checklist:**

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features
cargo test --workspace --lib
cargo deny check
```

4. **Review your changes** — `git diff --stat` to check scope, `git diff` to check content.

## PR Template

```markdown
## Summary

<!-- One-line description of what this PR does -->

## Changes

<!-- List of changes, organized by crate/file -->

- `opendev-foo/src/bar.rs`: Fixed X by doing Y
- `opendev-baz/src/qux.rs`: Added Z

## Related ADRs / Docs

<!-- Link to any ADRs or documentation changes -->

- ADR-00N: ...
- docs/...

## Testing

<!-- How was this tested? -->

- [ ] cargo test --workspace --lib
- [ ] cargo clippy --workspace --all-targets --all-features (no new warnings)
- [ ] Manual testing: ...

## Breaking Changes

<!-- Any breaking changes? Migration path? -->
```

## PR Guidelines

- **Keep PRs focused** — one logical change per PR. A PR should modify at most 3-5
  crates unless the change is cross-cutting.
- **Write descriptive commit messages** — concise title, blank line, body explaining
  what and why.
- **Include tests** — new functionality needs tests. Bug fixes need regression tests.
- **Update docs** — if your change affects architecture, add/update relevant docs.
  If you're adding a capability, consider adding a how-to guide in `docs/contributors/`.
- **No force pushes** after review has started — it breaks the review history.

## Review Process

1. At least one maintainer review required.
2. CI must pass (fmt and deny are blocking; clippy and test are report-only but
   new failures should be addressed).
3. Review focuses on: correctness, alignment with constitution, test coverage,
   error handling, and documentation.
