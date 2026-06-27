# CI/CD Pipeline

## Current CI Setup

CI is defined in `.github/workflows/ci.yml` and runs on push/PR to `main`/`master`.

### Jobs

| Job | Status | Blocking? | Command |
|---|---|---|---|
| `fmt` | ✅ | **Yes** | `cargo fmt --all --check` |
| `clippy` | ⚠️ Report-only | No | `cargo clippy --workspace --all-targets --all-features -- -D warnings` |
| `test` | ⚠️ Report-only | No | `cargo test --workspace --all-features` |
| `audit` | ⚠️ Report-only | No | `rustsec/audit-check@v2` |
| `deny` | ✅ | **Yes** | `cargo-deny check --all-features` |

### Policy

- **`fmt` (blocking)**: Unformatted code blocks PR merge. Always run `cargo fmt --all` before pushing.
- **`deny` (blocking)**: License or dependency policy violations block PR merge.
- **`clippy` (report-only)**: ~40 pre-existing warnings are tracked. New warnings are
  expected to be fixed, but pre-existing ones don't block CI.
- **`test` (report-only)**: Known flaky tests on Linux do not block CI. All tests should
  pass on macOS (primary development platform).
- **`audit` (report-only)**: 17 advisories are explicitly ignored (GTK3 transitive,
  `unic-*` family, `hickory-proto`). New advisory findings should be reviewed.

### Secrets

CI uses `GITHUB_TOKEN` for authentication. No LLM API keys are configured in CI —
tests that require API keys are skipped in CI.

## Local Checks

Before committing:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features
cargo test --workspace --lib
cargo audit --no-fetch       # Requires advisory DB
cargo deny check
```

The `npm run build` command validates the frontend build.

## Release Process

Current release process (v0.1.8 initial, v0.1.9 current):
1. Bump version in root `Cargo.toml` `[workspace.package]`.
2. Update `CHANGELOG.md`.
3. Run full CI locally.
4. Create release tag.
5. Build release artifacts.

Note: Internal crate versions in `[workspace.dependencies]` are still at `"0.1.6"` while
the workspace version is `"0.1.9"`. This mismatch needs reconciliation in a future release.
