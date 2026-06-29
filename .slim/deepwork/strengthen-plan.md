# OpenDev Desktop — Strengthen, Complete, and Repair Plan

**Based on:** 10-phase, 20+ specialist-agent deep comparison vs Claude Code.
**Principle:** Systematic, comprehensive, deep. Ignore priority ordering; organize by subsystem.
**Key:** ✅ = working, ⚠️ = partial/buggy, ❌ = missing, 🔧 = needs fix

---

## 1. Core Agent Loop & Orchestration

### 1.1 ⚠️ Wire ParallelPolicy Into Main Agent Loop
- **Current state:** `ParallelPolicy::partition_with_tools()` + `execute_batched` in `phases/tool_dispatch.rs` exist, but the main agent loop serializes all non-subagent tool calls. Only `execute_parallel` (all-subagent case) uses concurrency.
- **Gap:** CC's `StreamingToolExecutor` + `toolOrchestration.ts` execute read-only tools concurrently in the main turn. OD has the infrastructure (`ParallelPolicy`, trait-based `is_concurrent_safe(args)`) but never wires it into `tool_dispatch.rs` for general tools.
- **Action:** In `react_loop/phases/tool_dispatch.rs`, after `process_response`, add a pre-dispatch step that calls `ParallelPolicy::partition_with_tools()` on all non-subagent tool calls, and for batches >1 element, execute via `futures::join_all` with `Semaphore(10)`. Single-element batches fall through to existing `execute_sequential`.
- **Files:** `crates/opendev-agents/src/react_loop/phases/tool_dispatch.rs`, `crates/opendev-tools-core/src/parallel.rs`

### 1.2 ❌ Fork Subagent Cache Trick
- **Current state:** No content-addressable API prefix sharing. Each subagent call sends full system prompt + tools from scratch.
- **Gap:** CC's `buildForkedMessages()` shares parent's system prompt + tools + message prefix for byte-identical cache hits, reducing token cost and latency on forked subagents.
- **Action:** Implement `ForkedMessageBuilder` in `crates/opendev-agents/src/react_loop/`. When subagent `isolation` is `None` and `useExactTools` is true, clone parent messages, replace last assistant's tool_use blocks with `FORK_PLACEHOLDER_RESULT = "Fork started"`, build directive. Add `AgentDefinition.use_forked_cache: bool`.
- **Files:** New: `crates/opendev-agents/src/react_loop/forked_messages.rs`

### 1.3 ❌ Transition Tag for Loop Debuggability
- **Current state:** `TurnResult` enum encodes terminal conditions but not *why* loop continued.
- **Gap:** CC's `transition` field in its state provides `'collapse_drain_retry'`, `'max_output_tokens_recovery'` etc. — invaluable for debugging and observability.
- **Action:** Add `transition: Option<LoopTransition>` to `LoopState`. Define `enum LoopTransition { NextTurn, CollapseDrainRetry, MaxTokensEscalate, MaxTokensRecovery, StopHookBlocking, TokenBudgetContinue }`. Set at each `continue` site in `execution.rs`. Log in `iteration_metrics` via tracing.
- **Files:** `crates/opendev-agents/src/react_loop/loop_state.rs`, `types.rs`

### 1.4 🔧 Completion Blocking on Background Tasks
- **Current state:** `phases/completion.rs` counts `bg_tasks_spawned` vs `get_background_result` messages. CC has more robust check.
- **Gap:** CC's `handleStopHooks` runs `TaskCompleted` → `TeammateIdle` chains, ensuring no orphaned tasks.
- **Action:** Verify that `handle_completion` correctly handles: (a) subagent started but not finished, (b) bash backgrounded but still running, (c) MCP monitor tasks still polling. Add `TaskCompleted` hook chain.
- **Files:** `crates/opendev-agents/src/react_loop/phases/completion.rs`

### 1.5 ❌ ToolSearch Deferral Reducing Initial Schema
- **Current state:** `should_defer()` + `ToolSearchTool` exist. `mark_as_core()` marks core tools.
- **Gap:** CC's deferred-tool listing sends a compact `(name, description)` summary to the model so it knows what's available before calling `ToolSearch`. OD's `deferred_tools` prompt section exists but needs verification.
- **Action:** Verify `get_deferred_summaries()` in `ToolRegistry` outputs a proper categorized markdown list. Add integration test showing the LLM can discover and activate a deferred tool. Ensure `activated_tools` set is thread-safe and cleared per-turn.
- **Files:** `crates/opendev-tools-core/src/registry/mod.rs`

### 1.6 ❌ Reference-Based Error Watermarking
- **Current state:** No pattern. Error log tracking uses indices.
- **Gap:** CC uses reference-based watermarking (`errorLogWatermark`) to track which errors have been surfaced, avoiding index-shift bugs in ring buffers.
- **Action:** In `LoopState`, add `last_surfaced_error_id: Option<String>` (UUID). Surface only errors with IDs > this reference.
- **Files:** `crates/opendev-agents/src/react_loop/loop_state.rs`

---

## 2. Tool System

### 2.1 ❌ Per-Tool `maxResultSizeChars` Budget
- **Current state:** `TruncationRule` exists (`Head`, `Tail`, `HeadTail`) via `truncation_rule()` trait method. But no declarative per-tool max result size — truncation is after-the-fact.
- **Gap:** CC's `maxResultSizeChars: number` on every `Tool` — explicit declaration. Infinity for Read prevents circular Read→file→Read loops.
- **Action:** Add `max_result_size_chars: Option<usize>` to `BaseTool` trait (None = unlimited). Add to `ToolTruncationRule`. In `sanitizer.rs`, check this *before* applying truncation rule, spill to overflow file if exceeded, return `[truncated: N chars, use read_file with offset]`.
- **Files:** `crates/opendev-tools-core/src/traits.rs`, `sanitizer.rs`, `truncation.rs`

### 2.2 ❌ Same-File Read Content Dedup
- **Current state:** Only same-turn tool-level dedup via `dedup_cache` (tool_name + args JSON).
- **Gap:** CC's `read_dedup_killswitch` skips re-read if file content unchanged (mtime + content hash). Saves ~18% cache tokens.
- **Action:** In `FileReadTool`, before full read, compute quick hash of cached content. If mtime unchanged and hash matches, return `{ file_unchanged: true, previous_offset: N }`. Gate behind `feature("read_dedup")`.
- **Files:** `crates/opendev-tools-impl/src/file_read/mod.rs`

### 2.3 🔧 File Edit Atomicity & Concurrency
- **Current state:** Per-file `std::sync::Mutex` + `spawn_blocking` in `file_edit.rs`. Good but verify edge cases.
- **Gap:** CC's atomic write discipline: `mkdir(parent)` before no-yield critical section. OD does this but should be verified.
- **Action:** Audit `file_edit.rs` and `file_write.rs` for: (a) parent dir creation before lock acquisition, (b) tmp file with 0o600 mode, (c) rename atomicity on all platforms, (d) cleanup on error.
- **Files:** `crates/opendev-tools-impl/src/file_edit.rs`

### 2.4 🔧 LSP Post-Edit Diagnostics — Edge Case Hardening
- **Current state:** `diagnostics_helper::collect_post_edit_diagnostics` works. But edge cases.
- **Gap:** Need to handle: (a) LSP not initialized for this language, (b) diagnostics timeout, (c) diagnostics for multi-edit (one file, many edits — collect once after all), (d) large diagnostic output truncation.
- **Action:** Add timeout (5s), fallback when LSP unavailable, dedup diagnostics for multi-edit, cap output at 2000 chars.
- **Files:** `crates/opendev-tools-lsp/src/diagnostics.rs`

### 2.5 ❌ Bash Auto-Background Detection
- **Current state:** `patterns::is_server_command` detects flask/uvicorn/npm serve. Good but limited.
- **Gap:** CC detects `sleep`, `tail -f`, `watch`, `yes`, `top`, `htop`, and has `ASSISTANT_BLOCKING_BUDGET_MS = 15000` for auto-background.
- **Action:** Extend `is_server_command` → `is_auto_background_command(command)`. Add patterns for: `sleep N`, `tail -f`, `inotifywait`, `yes`, `top`, `htop`, `watch`, `ping`, `tcpdump`, `ngrok`, `python -m http.server`. Add `max_foreground_ms = 15000` timeout that auto-promotes to background.
- **Files:** `crates/opendev-tools-impl/src/bash/patterns.rs`, `bash/foreground.rs`

### 2.6 ❌ Bash AST-Based Security (Tree-Sitter)
- **Current state:** Regex-based `is_dangerous` + `BashToolPolicy` + OS sandbox.
- **Gap:** CC's `tree-sitter` + `ast.ts` (2679 lines) produces trustworthy `argv[]` from AST, fail-closed (unknown node → prompt user). OD has no AST analysis.
- **Action:** Integrate `tree-sitter-bash` grammar. Build `BashAstParser` that: (a) parses command, (b) extracts real argv (handles quotes, escapes, expansions deterministically), (c) fail-closed (parse error/timeout → mark as 'untrusted' → deny or ask). Use AST to improve permission pattern matching (currently regex-only).
- **Files:** New crate or module: `crates/opendev-exec/src/ast_parser.rs`, `bash_ast.rs`

### 2.7 ❌ Additional Bash Security Checks
- **Gap from CC:** 24 pre-parse security checks in `bashSecurity.ts`. OD has none of these.
- **Action:** Port from CC's `BASH_SECURITY_CHECK_IDS`:
  - Block command substitution variations: `<\(`, `>\(`, `=\(` (zsh process substitution)
  - Block `zmodload`, `emulate -c` (zsh eval-equivalent), `sysopen/sysread/syswrite/sysseek`
  - Block `=cmd` pattern (zsh equals expansion bypass)
  - Block heredoc in substitution (can smuggle command body)
  - Block obfuscated flags, IFS injection, git commit substitution
  - Block control characters and Unicode whitespace in commands
  - Block jq SYSTEM function and jq file arguments
- **Files:** New: `crates/opendev-exec/src/bash_security.rs`

### 2.8 ❌ Bash Permission Classifier
- **Current state:** No bash-specific classification beyond regex patterns.
- **Gap:** CC's `BASH_CLASSIFIER` allows users to define `Bash(prompt: <description>)` rules. A separate classifier side-query evaluates whether the user's description matches the actual command intent.
- **Action:** When a `Bash(prompt: *)` rule triggers, send a small synchronous side-query to a cheap model (Haiku equivalent) with: the prompt description, the actual command, and a yes/no prompt asking "Does this command match the described intent?" Fail-closed (unknown → deny).
- **Files:** New: `crates/opendev-runtime/src/bash_classifier.rs`

### 2.9 ❌ Path Validation — UNC, Tilde, Shell Expansion
- **Current state:** `normalizer::resolve_file_path` handles `~` and `$HOME`. No UNC blocking.
- **Gap:** CC blocks: (a) UNC paths (`\\server\share` — NTLM credential leak), (b) tilde expansion variants (`~root`, `~+`, `~-`), (c) shell expansion syntax (`$VAR`, `${VAR}`, `%TEMP%`, `=cmd`), (d) globs in write operations.
- **Action:** Add to `normalizer.rs`: `is_vulnerable_unc_path()`, `contains_tilde_expansion()`, `contains_shell_expansion()`. Return `ToolError::DangerousPath(reason)` for write operations on dangerous paths.
- **Files:** `crates/opendev-tools-core/src/normalizer.rs`, `path.rs`

### 2.10 ❌ Sed Validation
- **Current state:** Bash `sed` commands pass through regex `is_dangerous` check only.
- **Gap:** CC's `sedValidation.ts` rejects `e` flag, GNU extensions that allow code execution.
- **Action:** Parse `sed` expressions in bash commands. Reject `s/pat/repl/e` (execute flag), `s/pat/repl/ge`, and GNU-specific code-exec extensions.
- **Files:** `crates/opendev-exec/src/sed_validation.rs`

### 2.11 ❌ Read-Only Bash Command Validation — Expanded Allowlist
- **Current state:** `bash/mod.rs` has `is_likely_read_only_command()` with a static list of safe commands + git/cargo subcommand lists.
- **Gap:** CC's `readOnlyValidation.ts` (68KB) has per-command flag allowlists: `xargs` (excludes `-i`/`-e`), `fd` (excludes `-x`/`--exec`/`--exec-batch`), `git diff` (validates `-S`/`-G`/`-O` flag args), `gh`, `docker`, `pyright`, `ripgrep`.
- **Action:** Extend `is_likely_read_only_command()` with per-command flag-level validation. Key additions:
  - `xargs`: only safe with `-I '{}'`, `-E 'EOF'` (POSIX mandatory-separate-arg). Explicitly deny `-i`, `-e` (GNU optional-attached-arg semantics are unsafe).
  - `fd`/`fdfind`: deny `-x`/`--exec`, `-X`/`--exec-batch`, `-l`/`--list-details`.
  - `git`: expand subcommand list with flag validation (e.g., `git diff -S <string>` is read-only but `git diff -O <output-file>` is not).
- **Files:** `crates/opendev-tools-impl/src/bash/mod.rs`, New: `bash/readonly_validation.rs`

### 2.12 🔧 WebSearch — Replace DuckDuckGo Scraping
- **Current state:** `web_search/mod.rs` scrapes `https://html.duckduckgo.com/html/?q=...` — fragile, likely against ToS.
- **Action:** Replace with Brave Search API (or SerpAPI). Add `web_search.api_key` config to secrets store. Fallback chain: Brave API → SerpAPI → DDG (degraded, logged warning). Add `allowed_domains` / `blocked_domains` filter (already in schema, verify working).
- **Files:** `crates/opendev-tools-impl/src/web_search/mod.rs`

### 2.13 🔧 Browser — Add CDP/Playwright Bridge
- **Current state:** `browser.rs` 13 actions, most return "requires a real browser session" error. HTTP-only fallback for navigate/get_text/screenshot.
- **Action:** Implement `BrowserBackend` trait with two impls: `HttpBrowserBackend` (current) and `CdpBrowserBackend` (using headless Chrome via CDP or Playwright). Add `browser.backend: "http" | "cdp"` config. `CdpBrowserBackend` supports all 13 actions. Default to http, show "install playwright for full browser" hint.
- **Files:** `crates/opendev-tools-impl/src/browser.rs`, New: `browser/cdp_backend.rs`

### 2.14 ❌ Sensitive File Write Protection — Expand List
- **Current state:** `is_sensitive_file()` covers `.env*`, `.pem`, `.key`, `id_rsa`, `id_ed25519`, `credentials`, `service-account.json`, `.npmrc`, `.pypirc`, `.netrc`, `.htpasswd`, `*secret*.{json,yaml,yml}`.
- **Gap:** CC's `DANGEROUS_FILES` also includes `.gitconfig`, `.gitmodules`, `.bashrc`, `.zshrc`, `.profile`, `.ripgreprc`, `.mcp.json`, `.claude.json`.
- **Action:** Expand sensitive file list to match CC's. Also add: `.mcp.json`, `Cargo.toml` (dependency injection risk), `Makefile` (arbitrary command execution in auto-mode), `.github/workflows/*.yml` (CI injection).
- **Files:** `crates/opendev-runtime/src/permissions/mod.rs`

### 2.15 ❌ Tool-Level MCP Token Budget
- **Current state:** No MCP tool result size enforcement beyond the general truncation rules.
- **Gap:** CC's `MAX_MCP_OUTPUT_TOKENS = 25000`, `mcpContentNeedsTruncation()`, `truncateMcpContent()`.
- **Action:** Add `max_mcp_output_chars: usize` to `McpToolSchema` (default 25000). In `mcp_tool.rs`, after receiving MCP tool result, check total text content length. If exceeded, truncate with `[OUTPUT TRUNCATED - exceeded N token limit]` marker, offer `read_file` with overflow path.
- **Files:** `crates/opendev-tools-impl/src/mcp_tool.rs`, `crates/opendev-mcp/src/manager/tools.rs`

### 2.16 🔧 ToolResult.llm_suffix — Expand Usage
- **Current state:** `llm_suffix` field exists on `ToolResult`. Bash timeout uses it.
- **Action:** Systematically add `llm_suffix` to: (a) truncated outputs ("use read_file with offset to see more"), (b) backgrounded processes ("check later with get_background_result"), (c) permission denials in auto mode ("ask your user to approve"), (d) LSP errors discovered ("fix the error before proceeding"). Each suffix is distinct from user-visible output.
- **Files:** Various tool files in `crates/opendev-tools-impl/src/`

---

## 3. MCP Integration

### 3.1 ❌ OAuth 2.0 + PKCE Authorization Code Flow
- **Current state:** Only `client_credentials` grant (`crates/opendev-mcp/src/manager/protocol.rs:149-215`). No `authorization_code` flow, no PKCE, no token refresh.
- **Action:** Implement in `crates/opendev-mcp/src/auth.rs` (new file):
  - `McpAuthFlow` enum: `ClientCredentials | AuthorizationCode | None`
  - `AuthorizationCodeConfig`: `authorization_url`, `token_url`, `client_id`, `client_secret`, `redirect_uri` (or auto-pick port), `scopes`
  - PKCE: generate `code_verifier` (43-128 random chars), `code_challenge = base64url(sha256(verifier))`
  - Local redirect server: `tokio::net::TcpListener` on random port, single-request handler, extract `code` from query params
  - Token exchange: POST to `token_url` with `grant_type=authorization_code&code=...&code_verifier=...&redirect_uri=...`
  - Token storage: `McpTokenCache { access_token, refresh_token, expires_at }` in-memory + encrypted file fallback
  - Token refresh: on 401, POST `grant_type=refresh_token&refresh_token=...`, retry original request
- **Files:** New: `crates/opendev-mcp/src/auth.rs`, `crates/opendev-mcp/src/auth/token_cache.rs`

### 3.2 ❌ MCP WebSocket Transport
- **Current state:** `transport/mod.rs` supports `Http`, `Sse`, `Stdio` only.
- **Action:** Add `McpWebSocketTransport` to `transport/`:
  - Connect via `tokio_tungstenite::connect_async(url)`
  - Send JSON-RPC requests, receive responses
  - Handle reconnection with exponential backoff
  - Handle ping/pong for keepalive
  - Handle close frames gracefully
- **Files:** New: `crates/opendev-mcp/src/transport/websocket.rs`

### 3.3 ❌ MCP In-Process (SDK) Transport
- **Current state:** No in-process transport.
- **Gap:** CC's `SdkControlTransport` and `InProcessTransport` enable MCP servers running in the same process.
- **Action:** Add `McpInProcessTransport`:
  - `mpsc::unbounded_channel` for request/response
  - Direct Rust function calls instead of process spawning
  - Useful for built-in tools exposed as MCP (e.g., Git MCP server)
- **Files:** New: `crates/opendev-mcp/src/transport/inprocess.rs`

### 3.4 ❌ MCP Elicitation (Form + URL Modes)
- **Current state:** No elicitation. MCP servers that ask follow-up questions are unusable.
- **Action:** Implement `ElicitationHandler`:
  - `ElicitRequest { mode: Form | Url, schema: Value, elicitation_id: String }`
  - Form mode: render JSON schema as a form UI component (similar to `AskUserDialog`), collect answers, send `elicitation_result`
  - URL mode: open browser via `open::that(url)`, wait for `elicitation_complete` notification
  - Create `AskUserDialog` equivalent for MCP elicitation forms
  - Register handler in `McpManager`
- **Files:** New: `crates/opendev-mcp/src/elicitation.rs`, React component: `src/components/Chat/McpFormDialog.tsx`

### 3.5 ❌ MCP Image & Resource Content Extraction
- **Current state:** `mcp_tool.rs` only extracts `Text` content blocks. `Image` and `Resource` variants are dropped.
- **Action:** Add handling for:
  - Image: extract base64 data, resize if needed, return as image content block in the tool result
  - Resource: extract text or persist binary to disk, return path reference
  - Add `IMAGE_MAX_DIMENSIONS = (1920, 1080)`, `IMAGE_MAX_BYTES = 5_000_000`
- **Files:** `crates/opendev-tools-impl/src/mcp_tool.rs`

### 3.6 🔧 MCP Server Health Monitoring — Edge Cases
- **Current state:** Exponential backoff health monitoring works. 3 failures → tools removed, 5 restart attempts.
- **Action:** Add: (a) graceful 401 detection (mark as `needs-auth`, don't retry), (b) per-server health state persistence across restarts, (c) health status in MCP protocol section of system prompt, (d) `McpServerHealthState::PermanentlyFailed` detection for servers that consistently crash.
- **Files:** `crates/opendev-mcp/src/manager/health.rs`

### 3.7 🔧 MCP Config — Atomic Writes
- **Current state:** `config.rs` uses atomic write (tmp + rename). Verify it covers all save paths.
- **Action:** Ensure `save_config` in `crates/opendev-mcp/src/config.rs` uses `0o600` mode on Unix. Verify that OAuth secrets stored in MCP config use `secrecy::SecretString`.
- **Files:** `crates/opendev-mcp/src/config.rs`

---

## 4. Security & Sandbox

### 4.1 ❌ Complete Sandbox Integration
- **Current state:** `opendev-sandbox/src/sandbox.rs` is entirely stubs. `// TODO: PythonSandbox::create()`. `NoneBackend` is the default.
- **Action:** Implement in phases:
  - **Phase A — macOS Seatbelt**: Wire `exec/backends/seatbelt.rs` into bash tool execution path. Test with `sandbox-exec` profile generation. Verify it actually prevents write outside cwd.
  - **Phase B — Linux Landlock**: Wire `exec/backends/landlock.rs`. Test with `Landlock::restrict_self()`.
  - **Phase C — Linux bubblewrap**: Wire `exec/backends/bwrap.rs` as Landlock fallback.
  - **Phase D — Windows Job Objects**: Wire `exec/backends/windows.rs`.
  - **Phase E — microsandbox SDK**: Wire `opendev-sandbox` crate with actual microVM support.
- **Files:** `crates/opendev-exec/src/backends/`, `crates/opendev-sandbox/src/`

### 4.2 ❌ Sandbox Network Policies
- **Current state:** Sandbox backends don't enforce network policies.
- **Action:** Add allowed_domains/denied_domains to `ExecRequest`. For Seatbelt: add `(allow network-outbound (remote "*"))` filter. For Landlock: use `landlock::Ruleset::handle_access()`. For bwrap: add `--unshare-net` and optionally `--bind /etc/resolv.conf` for DNS.
- **Files:** `crates/opendev-exec/src/backends/`

### 4.3 ❌ Workspace Trust Dialog
- **Current state:** No trust dialog. Agent starts immediately.
- **Gap:** CC's `TrustDialog` shows project-level indicators before first session: MCP servers, hooks, bash allow rules, apiKeyHelper, dangerous env vars.
- **Action:** Create `TrustDialog` React component. Show on first session open for a project. Indicators:
  - `.mcp.json` servers found
  - Project hooks in `.opendev/settings.json`
  - Custom agents/tools defined in project
  - Any env vars set by project config
  - "Trust and Continue" / "Don't Trust" / "Exit" buttons
  - Persist trust to `~/.opendev/projects/<hash>/trusted` file
- **Files:** New: `src/components/Chat/TrustDialog.tsx`, `src/stores/trust.ts`

### 4.4 ❌ Bypass Permissions Killswitch
- **Current state:** No mechanism to disable bypass-permissions mode.
- **Action:** Add `bypass_permissions_killswitch` to config. When set, entering bypass mode is blocked. Controlled via `settings.permissions.disableBypassPermissionsMode: bool`. Check at mode transition boundaries.
- **Files:** `crates/opendev-runtime/src/permissions/mod.rs`

### 4.5 ❌ Managed Settings / Policy Settings
- **Current state:** No managed settings concept. Only user/project settings.
- **Action:** Add `policySettings` source tier (highest priority). Load from `/etc/opendev/settings.json` or env-specified path (`OPENDEV_POLICY_SETTINGS_PATH`). When `allowManagedPermissionRulesOnly` is true, only policySettings rules apply. Block UI edits to policy-derived rules. Add `ManagedSettingsSecurityDialog` for IT-admin transparency.
- **Files:** `crates/opendev-config/src/`, `crates/opendev-runtime/src/permissions/`

### 4.6 🔧 Subprocess Env Scrubbing — Expand Scrub List
- **Current state:** `exec/env_filter.rs` has `SENSITIVE_ENV_SUFFIXES` (16 suffixes) + `SENSITIVE_ENV_EXACT` (~30 names).
- **Gap:** CC's `subprocessEnv.ts` has GHA-specific scrub list for CI/CD environments.
- **Action:** Add GHA scrub list when `GITHUB_ACTIONS=true`:
  - `ACTIONS_ID_TOKEN_REQUEST_TOKEN`, `ACTIONS_ID_TOKEN_REQUEST_URL` (OIDC)
  - `ACTIONS_RUNTIME_TOKEN`, `ACTIONS_RUNTIME_URL` (artifact/cache API)
  - `ALL_INPUTS` (contains API keys as JSON)
  - `OVERRIDE_GITHUB_TOKEN`, `DEFAULT_WORKFLOW_TOKEN`
  - Strip `INPUT_<NAME>` duplicates
- **Files:** `crates/opendev-exec/src/env_filter.rs`

### 4.7 ❌ Secrets Detection in Tool Outputs
- **Current state:** `secrets.rs` has 7 regex patterns. Detection exists but no integration into the tool output pipeline.
- **Action:** In `sanitizer.rs`, after tool execution, run `detect_secrets()` on output. If any secrets found: (a) redact them, (b) log a warning with `SecretKind`, (c) add `[SECRETS REDACTED]` note to LLM suffix, (d) optionally emit `RuntimeEvent::SecretsDetected` for analytics.
- **Files:** `crates/opendev-tools-core/src/sanitizer.rs`, `crates/opendev-runtime/src/secrets.rs`

### 4.8 ❌ Auto-Approval Classifier
- **Current state:** No equivalent to CC's YOLO classifier. Every write requires user confirmation.
- **Action:** Implement `ApprovalClassifier`:
  - **Simple heuristic first** (1 week): approve if (a) tool is read-only, (b) edit is same-file + small diff (< 50 lines), (c) bash command is on the read-only allowlist, (d) write target is within workspace + not a sensitive file
  - **Advanced classifier** (2-3 weeks): Port CC's 2-stage XML approach. Stage 1 (fast): `max_tokens=64`, `Err on side of blocking. <block> immediately.` Stage 2 (thinking): `max_tokens=4096`, chain-of-thought. Use cheap model (Haiku equivalent). Cache system prompt with 1h TTL.
  - Add `auto_accept` permission mode (like CC's `acceptEdits`)
  - Add denial tracking: 3 consecutive or 20 total → fallback to manual (like CC)
  - Add `SAFE_ALLOWLISTED_TOOLS` fast path (Read, Grep, Glob, etc. — skip classifier entirely)
- **Files:** New: `crates/opendev-runtime/src/classifier.rs`, `crates/opendev-runtime/src/classifier/xml_analysis.rs`

### 4.9 ❌ Dangerous Pattern Allow Rules Detection
- **Current state:** No detection for overly-broad permission rules.
- **Action:** When user adds a new allow rule, check against dangerous patterns:
  - `Bash(*)` or `Bash` (no args) — warn "this allows any command, equivalent to disabling security"
  - `Bash(python:*)` — warn "this allows arbitrary code execution"
  - `Agent(*)` — warn "this auto-approves sub-agent spawns before evaluating their prompts"
  - `WebFetch(*)` — warn "this allows network exfiltration"
  - Show warning in UI approval dialog, require double-confirmation
- **Files:** `crates/opendev-runtime/src/permissions/dangerous_patterns.rs`

### 4.10 ❌ MCP Server Project Approval
- **Current state:** No approval flow for project-level MCP servers.
- **Action:** On session start, check for project `.mcp.json` servers not yet approved. Show `McpServerApprovalDialog` listing each server with its config (masked secrets). "Enable All" / "Enable Selected" / "Disable All".
- **Files:** New: `src/components/Chat/McpApprovalDialog.tsx`

---

## 5. Prompt Engineering

### 5.1 ❌ Global Prompt Cache (Content-Hash-Based)
- **Current state:** `PromptComposer` with `CachePolicy::Static` caches per-session only.
- **Gap:** CC's `splitSysPromptPrefix` + `cache_control: {scope: 'global'}` shares static prompt blocks org-wide. ~7 sections, 5-13K tokens saved per session.
- **Action:** Implement `GlobalPromptCache`:
  - On session start, compute `sha256` of static sections (priority 12-25 sections from `factories.rs`)
  - Store in `~/.opendev/prompt-cache/{hash}.txt`
  - On subsequent sessions, load from cache, skip composition
  - Invalidate on: (a) version change, (b) config change affecting static sections, (c) 24h TTL
  - Integrate with Anthropic adapter: add `cache_control: {type: "ephemeral", scope: "org"}` blocks
- **Files:** New: `crates/opendev-agents/src/prompts/global_cache.rs`

### 5.2 🔧 Anthropic Prompt Caching Integration
- **Current state:** `compose_two_part()` returns `(stable, dynamic)` but runtime uses single-string `compose()`. Anthropic adapter (`adapters/anthropic/mod.rs`) doesn't add `cache_control` blocks.
- **Action:** Switch runtime to use `compose_two_part()`. In Anthropic adapter, when `prompt-caching-2024-07-31` beta is active, tag stable blocks with `cache_control: {type: "ephemeral"}`. Add `cache_control` to the last user message in conversation history.
- **Files:** `crates/opendev-cli/src/runtime/mod.rs` (line ~691), `crates/opendev-http/src/adapters/anthropic/mod.rs`

### 5.3 🔧 Content-Based Prompt Cache Boundary
- **Current state:** OD uses `CachePolicy` enum per section. CC uses literal string marker `__SYSTEM_PROMPT_DYNAMIC_BOUNDARY__`.
- **Action:** Evaluate pros/cons. OD's approach is cleaner but CC's marker enables content-based splitting at the API layer (important for Anthropic's `splitSysPromptPrefix`). Consider adding both: keep `CachePolicy` for internal composition, insert a marker string between Static and Cached sections for API-level splitting.
- **Files:** `crates/opendev-agents/src/prompts/composer/mod.rs`

### 5.4 ❌ Additional Context Collectors
- **Current state:** 8 collectors: TodoState, PlanMode, DateChange, GitStatus, Compaction, Memory (15 turns), SessionMemory (50K tokens). CC has 15+.
- **Action — New collectors to add:**
  - **RecentFilesCollector** (5 turns): recently edited files in this session, from `ArtifactIndex`
  - **IdeStateCollector** (10 turns): IDE diagnostics/current file (from LSP integration)
  - **SkillListingCollector** (once per session, suppress after): available skills with descriptions (like CC's `skill_listing` attachment)
  - **AgentListingCollector** (on change): available agent definitions (like CC's `agent_listing_delta`)
  - **ChangedFilesCollector** (5 turns): git diff since session start
  - **DependencyGraphCollector** (on demand): package/crate dependency structure for architecture questions
  - **McpInstructionsCollector** (on MCP connect/disconnect): delta of MCP server instructions
- **Files:** `crates/opendev-agents/src/attachments/collectors/`

### 5.5 🔧 Skill System — Complete Lifecycle
- **Current state:** `Active →(30d)→ Stale →(90d)→ Archived`. Good but needs integration with prompt.
- **Action:** Add skill listing to system prompt (like CC's `<system-reminder>` with descriptions). Add `SkillTool` prompt contribution (`prompt_contribution() -> Option<String>`). Add skill discovery on first invocation (lazy load companion files).
- **Files:** `crates/opendev-agents/src/skills/`, `crates/opendev-agents/src/prompts/`

### 5.6 🔧 Embedded Template Count & Coverage
- **Current state:** 79 embedded templates. CC has ~45 per-tool prompt templates.
- **Action:** Verify all 26 default sections in `composer/factories.rs` have corresponding `.md` files in the embedded store. Add any missing: `provider_openai.md`, `provider_anthropic.md`, `provider_fireworks.md`, `code_references.md`, `output_awareness.md`, `no_time_estimates.md`.
- **Files:** `crates/opendev-agents/src/prompts/embedded.rs`

### 5.7 ❌ Per-Tool Prompt Templates
- **Current state:** Tools have `prompt_contribution()` returning `Option<String>`. Unclear if systematically filled.
- **Action:** For each core tool (Bash, Read, Write, Edit, Glob, Grep, WebFetch, WebSearch, AskUser, TaskComplete, TodoWrite, Agent, MCP), create a `tools/tool-<name>.md` embedded template. Each describes: tool purpose, when to use vs alternatives, parameter guidance, output interpretation, known limitations. Register via `prompt_contribution()` on each tool.
- **Files:** `crates/opendev-agents/src/prompts/embedded.rs` (add templates), each tool's `BaseTool` impl.

### 5.8 ❌ Nudge System Enhancements
- **Current state:** `ProactiveReminderScheduler` with `task_proactive_reminder` (every 10 turns). Reactive nudges from `reminders.md`: `failed_tool_nudge`, `doom_loop_redirect_nudge`, etc.
- **Action:** Add CC-style reactive nudges missing:
  - `consecutive_reads_nudge` — when last 5+ tool calls are all reads, nudge to synthesis
  - `incomplete_todos_nudge` — when `task_complete` called but todos remain
  - `verification_nudge` — when tasks completed without verification step
  - `empty_output_nudge` — when command produces no output, suggest checking
  - `stuck_loop_nudge` — when same tool called 10+ times in sequence
- **Files:** `crates/opendev-agents/src/prompts/reminders.rs`, `templates/reminders.md`

---

## 6. Memory & Context

### 6.1 🔧 Memory FTS5 Integration
- **Current state:** `repo.rs` uses `LIKE %query%` instead of FTS5 virtual table.
- **Action:** Add FTS5 virtual table:
  ```sql
  CREATE VIRTUAL TABLE memory_fts USING fts5(content, category, tags, content=long_term_memory, content_rowid=rowid);
  ```
  Create triggers to keep FTS in sync on INSERT/UPDATE/DELETE. Change `search_fts` to use `MATCH` query. Fall back to `LIKE` if FTS not available.
- **Files:** `crates/opendev-memory/src/repo.rs`, `migration.rs`

### 6.2 ❌ Memory Symbol Links — Full Integration
- **Current state:** `memory_symbol_links` table exists. `MemoryDecay::score_with_links()` uses `has_symbol_links` boost. But no automated symbol link creation.
- **Action:** When a file edit tool touches a function/struct definition, automatically create symbol links via LSP: extract symbol name, find in codebase, link to memory entries mentioning that symbol. Use `MemoryRepo::link_to_symbol()`.
- **Files:** `crates/opendev-memory/src/symbol_links.rs`

### 6.3 🔧 Session Memory — Improve Extraction
- **Current state:** `SessionMemoryCollector` fires every 50K tokens, uses cheap LLM extraction, writes to file.
- **Action:** Add structured format matching CC's `DEFAULT_SESSION_MEMORY_TEMPLATE`: `# Session Title`, `# Current State`, `# Task specification`, `# Files and Functions`, `# Workflow`, `# Errors & Corrections`, `# Learnings`. Use forked subagent with limited tool set (Read + Edit on memory path only). Compaction should read session memory summary to restore context.
- **Files:** `crates/opendev-agents/src/attachments/collectors/session_memory.rs`

### 6.4 🔧 Context Compaction — LLM Compaction
- **Current state:** `summary.rs::build_compaction_payload()` + `apply_llm_compaction()` exist. Verifying integration.
- **Action:** Verify the compaction pipeline: (a) `check_usage()` → returns `Compact` at 99%, (b) `build_compaction_payload()` constructs API payload with sanitized conversation, (c) calls LLM, (d) `apply_llm_compaction()` replaces middle messages. Add integration test. Ensure `ArtifactIndex` survives compaction (checked — `as_summary()` appended to summary block).
- **Files:** `crates/opendev-context/src/compaction/summary.rs`

### 6.5 ⚠️ Token Counting Accuracy
- **Current state:** `count_tokens()` in `tokens.rs` uses cl100k_base heuristic (whitespace split, word length >12 = ceil(len/4), 0.75 ratio). `update_from_api_usage()` calibrates from real API counts.
- **Action:** Port a proper tokenizer (e.g., `tiktoken-rs` or `tokenizers` crate) for accurate counting. Use real tokenizer for compaction decisions (>70% thresholds). Fall back to heuristic if tokenizer unavailable.
- **Files:** `crates/opendev-context/src/compaction/tokens.rs`

---

## 7. Streaming & Provider Integration

### 7.1 🔧 Anthropic Adapter — Preserve Native Optimizations
- **Current state:** Everything normalized to OpenAI Chat Completions. Anthropic-native features lost.
- **Action:** Add Anthropic-native path in adapter:
  - Preserve `system` as top-level field (not injected as message)
  - Use Anthropic tool format (not OpenAI `function` format)
  - Add `cache_control: {type: "ephemeral"}` blocks
  - Add interleaved thinking via `thinking.budget_tokens`
  - Map `temperature=1` when thinking enabled (Anthropic requirement)
  - Map `stop_reason: end_turn/tool_use/max_tokens` correctly
  - Add `anthropic-beta: prompt-caching-2024-07-31,interleaved-thinking-2025-05-14`
- **Files:** `crates/opendev-http/src/adapters/anthropic/mod.rs`

### 7.2 ❌ Streaming Fallback to Non-Streaming
- **Current state:** No fallback. Stream failure → error.
- **Gap:** CC's `executeNonStreamingRequest()` catches mid-stream errors and retries as non-streaming, capped at 64K tokens. Critical for reliability.
- **Action:** In `adapted_client.rs::post_json_streaming()`, on stream parse error or transport error after stream started: (a) call non-streaming version, (b) cap `max_tokens` at 64000, (c) convert response to same format, (d) log `tengu_streaming_fallback_to_non_streaming` event.
- **Files:** `crates/opendev-http/src/adapted_client.rs`

### 7.3 ❌ API Retry with Exponential Backoff
- **Current state:** `HttpClient::send_streaming_request()` retries on 429/503 before body consumed. But no post-connection retry strategy.
- **Action:** Implement `with_retry` wrapper (CC's `withRetry.ts` equivalent):
  - Max 10 retries
  - Exponential backoff: base 500ms, 25% jitter, cap 32s
  - `Retry-After` header overrides
  - 529 (overloaded) → retry up to 3 times, then trigger model switch
  - 401/403 → refresh auth token, retry once
  - `ECONNRESET`/`EPIPE` → `disable_keep_alive()`, retry
  - Stale connection → drop pooled socket, retry
  - Context overflow → parse available tokens from error message, adjust `max_tokens_override`
  - Log every retry decision via tracing
- **Files:** New: `crates/opendev-http/src/retry.rs`

### 7.4 ❌ Model Fallback (Opus → Sonnet on Overload)
- **Current state:** No automatic model fallback.
- **Action:** When 529 persists 3+ times for Opus-class models, automatically switch to Sonnet-class. Store original model, switch back on next user turn or `/model` command. Show "⚠️ Switched to Sonnet due to capacity" in status bar.
- **Files:** `crates/opendev-http/src/client.rs`, `crates/opendev-cli/src/runtime/mod.rs`

### 7.5 ❌ Stream Resource Cleanup (Memory Leak Prevention)
- **Current state:** No explicit stream resource cleanup.
- **Gap:** CC's `releaseStreamResources()` calls `stream.controller.abort()` + `streamResponse.body?.cancel()` in a `finally` block to release native TLS/socket buffers. Without this, native memory leaks.
- **Action:** In `adapted_client.rs`, add a `Drop` impl or explicit `release()` method on stream handle. On early exit (user interrupt, error, timeout), call `response_body.cancel()`. Add `finally`-like pattern in `post_json_streaming()`.
- **Files:** `crates/opendev-http/src/adapted_client.rs`

### 7.6 🔧 Streaming Watchdog / Stall Detection
- **Current state:** `MAX_STREAM_DURATION = 300s`, `tokio::time::timeout(120s)` for idle. Good but incomplete.
- **Action:** Add: (a) stall detection: if >30s between any chunk, log warning, (b) watchdog: if `ENABLE_STREAM_WATCHDOG=true`, 90s idle → abort stream, throw. Both behind feature flags.
- **Files:** `crates/opendev-http/src/adapted_client.rs`

### 7.7 ❌ Provider-Aware Streaming Optimizations
- **Current state:** All providers use the same `parse_stream_event` pattern.
- **Action:** Add provider-specific streaming optimizations:
  - **Anthropic**: use raw SSE bytes (not `BetaMessageStream`) to avoid O(n²) JSON reparsing (like CC)
  - **OpenAI**: handle `response.output_text.delta` and `response.function_call_arguments.delta` separately
  - **Gemini**: use `alt=sse` URL rewrite, handle `thought: true` parts
  - Each adapter's `parse_stream_event` should be optimal for that provider's wire format
- **Files:** Each adapter `response.rs` under `crates/opendev-http/src/adapters/`

---

## 8. Session & Persistence

### 8.1 🔧 File-Based Session — Atomicity Edge Cases
- **Current state:** `SessionManager` uses temp file + `rename()` for atomic writes. JSON metadata + JSONL messages.
- **Action:** Audit for edge cases: (a) concurrent writes from subagents, (b) crash during write (stale tmp files), (c) partial JSONL lines (truncation), (d) JSON metadata + JSONL out of sync, (e) very large sessions (>100MB JSONL) — consider splitting into chunks.
- **Files:** `crates/opendev-history/src/session_manager/mod.rs`

### 8.2 ❌ Session Forking & Parent ID
- **Current state:** `Session` has `parent_id: Option<String>` and `subagent_sessions: HashMap`. Schema supports forks. But no fork creation flow.
- **Action:** Implement `SessionManager::fork_session(parent_id, fork_point)`:
  - Create new session with `parent_id` set
  - Copy messages up to `fork_point`
  - Update parent's `subagent_sessions` map
  - Emit `SessionForked` event
- **Files:** `crates/opendev-history/src/session_manager/mod.rs`

### 8.3 🔧 Event Store — Completion & Testing
- **Current state:** `EventStore` (JSONL events) + `SessionProjector` exist. `Tombstone` undo mechanism.
- **Action:** Complete test coverage: (a) replay from empty, (b) replay with tombstones, (c) replay with fork events, (d) concurrent event append, (e) file lock contention, (f) corrupted event recovery. Add `EventStore::validate_integrity()` that reads all events and verifies sequence.
- **Files:** `crates/opendev-history/src/event_store.rs`

### 8.4 ❌ Snapshot Manager — Integration Into Agent Loop
- **Current state:** `SnapshotManager` creates shadow git repo for per-step undo. `track()` / `revert()` / `restore()`. But not called from agent loop.
- **Action:** In `tool_dispatch.rs`, after successful write/edit tool execution, call `snapshot_manager.track()`. Before each edit, save snapshot for undo. On `undo` request, call `snapshot_manager.restore(tree_hash)`. Add `SnapshotCollector` context collector showing file changes since last snapshot.
- **Files:** `crates/opendev-agents/src/react_loop/phases/tool_dispatch.rs`, `crates/opendev-context/src/compaction/snapshot.rs`

### 8.5 🔧 Session Index — Multi-Project Listing
- **Current state:** `SessionListing::list_sessions(owner_id)` lists by owner. `list_all_sessions(projects_dir)` merges multiple project dirs.
- **Action:** Verify cross-project session listing works correctly. Add sorting by `last_activity` (currently `updated_at`). Add search by title pattern. Add pagination limit.
- **Files:** `crates/opendev-history/src/listing.rs`

---

## 9. UI & UX

### 9.1 ❌ Plan Approval Flow — UI Completion
- **Current state:** `PlanApprovalDialog.tsx` with 3 options. Works but needs polish.
- **Action:** Add: (a) plan content markdown rendering with syntax highlighting, (b) "approve and auto-accept edits" mode (skips future edit confirmations for this plan), (c) plan diff view when revising, (d) keyboard-only navigation improvements, (e) plan archive at `~/.opendev/plans/`.
- **Files:** `src/components/Chat/PlanApprovalDialog.tsx`

### 9.2 ❌ Subagent Visualization — Interactive Tree
- **Current state:** `SubagentTree.tsx` shows hierarchy with active/completed tools. Good foundation.
- **Action:** Add: (a) click-to-expand subagent result, (b) inline tool output preview (first 200 chars), (c) subagent cost/token display per agent, (d) "stop subagent" button, (e) subagent status color coding (running=blue, success=green, error=red), (f) pulsating border for currently active subagent.
- **Files:** `src/components/Chat/SubagentTree.tsx`

### 9.3 ❌ Diff Viewer — Enhanced Features
- **Current state:** `DiffViewer.tsx` — basic line-by-line classification by prefix.
- **Action:** Add: (a) side-by-side mode, (b) word-level highlighting within lines, (c) syntax highlighting for the file language, (d) "apply/reject hunk" buttons (like `git add -p`), (e) collapsed unchanged sections with N-line skip indicators.
- **Files:** `src/components/Chat/DiffViewer.tsx`

### 9.4 ❌ Terminal Output — ANSI Escape Handling
- **Current state:** `BashPreview.tsx` strips ANSI escapes. `MarkdownContent.tsx` renders code blocks.
- **Action:** Implement proper ANSI-to-HTML converter for bash output display. Handle: colors (256), bold/dim/italic/underline, cursor movement, clear screen sequences. Render as styled HTML in output blocks. This is key for commands like `cargo test`, `npm test`, `git diff --color`.
- **Files:** New: `src/utils/ansiToHtml.ts`, `src/components/Chat/AnsiOutput.tsx`

### 9.5 ❌ File Tree / Workspace Browser
- **Current state:** `FileMentionDropdown.tsx` shows flat file list for `@` mentions.
- **Action:** Add workspace file tree panel (sidebar or popover). Features: (a) recursive directory tree view, (b) file type icons, (c) git status indicators (modified/staged/untracked), (d) click to open/read file, (e) drag to add to context, (f) search/filter.
- **Files:** New: `src/components/Chat/FileTreePanel.tsx`

### 9.6 ❌ Settings UI — Complete Panels
- **Current state:** Settings panels for Model, MCP, Theme, Skills, Privacy exist.
- **Action:** Add missing settings panels: (a) **Permissions**: view/edit allow/deny rules with pattern builder, (b) **Sandbox**: configure sandbox backends, network policies, filesystem policies, (c) **Memory**: view/manage memories, configure retention, (d) **Hooks**: configure lifecycle hooks, (e) **Cost**: cost breakdown by session/model, spending limits.
- **Files:** `src/components/Settings/`

### 9.7 ❌ Command Palette — Full Command Set
- **Current state:** `CommandPalette.tsx` with 7 hardcoded commands.
- **Action:** Add all slash commands: `/compact`, `/cost`, `/status`, `/diff`, `/clear`, `/review`, `/commit`, `/init`, `/doctor`, `/feedback`, `/share`, `/resume`, `/rename`, `/export`. Add command search/filter. Add keyboard shortcut display. Extend command registration to arbitrary user-defined commands.
- **Files:** `src/components/Chat/CommandPalette.tsx`

### 9.8 ❌ Toast / Notification System Enhancement
- **Current state:** `sonner` toasts for errors and completions.
- **Action:** Add: (a) persistent notification center (bell icon with unread count), (b) notification categories (info/warning/error/success/cost), (c) action buttons on toasts ("Show Diff", "Undo"), (d) sound alerts for completions (CC's `play_finish_sound`), (e) Do Not Disturb mode.
- **Files:** `src/components/Chat/`, `src/stores/notifications.ts`

### 9.9 ❌ Keyboard Shortcuts — Full Coverage
- **Current state:** Basic shortcuts (Ctrl+N new session, Esc close modal, etc.)
- **Action:** Implement comprehensive keyboard shortcut system:
  - `Ctrl+K` — command palette
  - `Ctrl+L` — clear chat
  - `Ctrl+Shift+C` — copy last response
  - `Ctrl+Shift+E` — export session
  - `Ctrl+.` — interrupt
  - `Ctrl+B` — background current task
  - `Ctrl+R` — history search
  - `Ctrl+Shift+M` — toggle mode
  - Global + context-specific shortcuts
  - Shortcut customization in settings
  - Visual shortcut hints
- **Files:** `src/hooks/useKeyboardShortcuts.ts`, `src/stores/keybindings.ts`

### 9.10 🔧 Zusand Store — Optimistic Update Edge Cases
- **Current state:** `chat.ts` (904 lines) handles ~20 event subscriptions. Optimistic user messages.
- **Action:** Fix edge cases: (a) optimistic message expiration on reconnect (currently 30s, verify works), (b) race condition between `user_message` event and optimistic match, (c) store cleanup on session delete (clear `sessionStates[id]`), (d) memory leak on rapid session switching (messages accumulate in cache), (e) ensure all event handlers handle null/undefined `currentSessionId`.
- **Files:** `src/stores/chat.ts`

---

## 10. CLI & Entry Points

### 10.1 ❌ Session Resume — Interactive Picker Polish
- **Current state:** `runners.rs` has `--resume` flag, interactive picker lists 20 most recent.
- **Action:** Enhance interactive picker: (a) search/filter by title, (b) keyboard navigation (j/k or ↑↓), (c) session preview on hover (first 3 messages), (d) project grouping, (e) "resume last" shortcut, (f) session deletion from picker.
- **Files:** `crates/opendev-cli/src/runners.rs`

### 10.2 ❌ `--continue` / `--resume` Consistency
- **Current state:** Both flags exist. Behavior may be inconsistent.
- **Action:** Ensure: `--continue` resumes most recent session in current project. `--resume` (no arg) shows interactive picker across all projects. `--resume <id>` resumes specific. `--resume <slug>` resumes by slug. Handle edge case where session's working directory differs from current.
- **Files:** `crates/opendev-cli/src/runners.rs`

### 10.3 ❌ REPL Enhancements
- **Current state:** `opendev-repl` with readline-based input, `/` commands.
- **Action:** Add: (a) multi-line input (paste detection), (b) syntax highlighting for code blocks in input, (c) auto-completion for file paths and slash commands, (d) UP/DOWN history navigation, (e) Ctrl+R history search, (f) line editing (Ctrl+A/E, Ctrl+W, Alt+B/F), (g) colored output for different message roles.
- **Files:** `crates/opendev-repl/src/`

### 10.4 ⚠️ Headless / Non-Interactive Mode — Completion
- **Current state:** `run_non_interactive()` sends prompt, prints result, exits. Workflow channel (approvals, ask_user) not handled.
- **Action:** In non-interactive mode: (a) auto-approve read-only tools, (b) auto-deny destructive writes (or use `--dangerously-skip-permissions` flag), (c) render ask_user as text output and wait for stdin, (d) print subagent progress, (e) exit with correct code (0=success, 1=error).
- **Files:** `crates/opendev-cli/src/runners.rs`

### 10.5 ❌ Remote Session Mode (like CC's CCR)
- **Current state:** `commands/channel.rs` has Telegram remote mode. No generic remote session support.
- **Action:** Add `opendev remote` mode (generic, not just Telegram):
  - Connect to remote OpenDev instance via WebSocket
  - Forward stdin/stdout
  - Handle reconnection
  - File synchronization (send/receive project files)
  - Session state sync
- **Files:** `crates/opendev-cli/src/commands/remote.rs`, `crates/opendev-web/src/`

---

## 11. Plugins & Extensibility

### 11.1 ⚠️ Plugin Manager — Marketplace
- **Current state:** `opendev-plugins` crate with basic manager. No marketplace.
- **Action:** Add plugin marketplace:
  - Registry URL (`https://plugins.opendev.ai/index.json`) with plugin metadata
  - `opendev plugin search <query>` — search registry
  - `opendev plugin install <name>` — download and extract to `~/.opendev/plugins/`
  - `opendev plugin update` — check for updates
  - `opendev plugin uninstall <name>`
  - Plugin manifest schema: `{name, version, description, author, tools, mcp_servers, skills, hooks, commands}`
  - Version compatibility check
  - Plugin sandboxing (each plugin runs in MCP stdio process)
- **Files:** `crates/opendev-plugins/src/`, `crates/opendev-cli/src/commands/plugin.rs`

### 11.2 ❌ Custom Tool Definition (User-Defined)
- **Current state:** `custom_tool.rs` exists. Verify functionality.
- **Action:** Allow users to define custom tools as:
  - Shell commands in `~/.opendev/tools/*.json` with `{name, description, command, parameters, output_format}`
  - Validation: command must be in allowlist or go through sandbox
  - Parameter substitution: `{param_name}` in command string
  - Output parsing: json/text
  - Security: each custom tool runs in sandbox, env filtered
- **Files:** `crates/opendev-tools-impl/src/custom_tool.rs`

### 11.3 ❌ Lifecycle Hooks — Shell-Based
- **Current state:** `opendev-hooks` crate exists. Verify current state.
- **Action:** Implement CC-compatible hook system:
  - Hook types: `PreToolUse`, `PostToolUse`, `Notification`, `Stop`, `SubagentStop`, `SessionStart`, `SessionEnd`, `PreCompact`
  - Hook definition: shell command with `{tool_name}`, `{tool_input}`, `{cwd}` env vars
  - Hook output format: JSON with `{continue, stopReason, decision, systemMessage}`
  - Timeout: 60s per hook
  - Environment: scrubbed (same as bash)
- **Files:** `crates/opendev-hooks/src/`

---

## 12. Observability & Analytics

### 12.1 ⚠️ Telemetry — Complete OpenTelemetry Integration
- **Current state:** `opendev-telemetry` with JSON logging, OTLP, Sentry. Feature-gated.
- **Action:** Ensure complete coverage: (a) LLM call spans (request, first-token, completion), (b) tool execution spans (resolve, execute, sanitize), (c) subagent lifecycle spans, (d) MCP operation spans, (e) cost metrics (per model, per session, cumulative), (f) error spans with structured error data, (g) W3C trace context propagation across subagent boundaries.
- **Files:** `crates/opendev-telemetry/src/`

### 12.2 ❌ GrowthBook/Feature Flag Integration
- **Current state:** No feature flag system.
- **Action:** Integrate GrowthBook SDK (or simple JSON feature flag file):
  - `~/.opendev/features.json` — local overrides
  - Remote feature flag endpoint for managed rollouts
  - Gate new features behind flags
  - A/B test support
  - Killswitch for problematic features
- **Files:** New: `crates/opendev-config/src/features.rs`

### 12.3 ❌ Session Debug Logger
- **Current state:** `SessionDebugLogger` exists. Logs `llm_request`, `llm_response`, `llm_error`, `full`.
- **Action:** Add: (a) per-session debug file at `~/.opendev/sessions/<id>/debug.log`, (b) tool execution logging with args and results, (c) permission decision logging, (d) compaction event logging, (e) MCP communication logging, (f) configurable log levels per category.
- **Files:** `crates/opendev-runtime/src/debug_logger.rs`

### 12.4 ❌ Cost Tracking — Per-Session Breakdown
- **Current state:** `CostTracker` with `TokenUsage` and `PricingInfo`. Good.
- **Action:** Add: (a) CostTracker UI panel showing per-session and cumulative costs, (b) cost by model breakdown, (c) cost attribution to subagents, (d) spending limits (soft/hard cap), (e) cost estimate before LLM call, (f) cost tracking persistence across sessions.
- **Files:** `crates/opendev-runtime/src/cost_tracker.rs`, new React component

---

## 13. Testing & Quality

### 13.1 ❌ Agent Loop Integration Tests
- **Action:** Create integration tests for the full agent loop: (a) simple tool call → response, (b) multi-turn conversation, (c) subagent spawn and completion, (d) background task tracking, (e) compaction triggering, (f) doom loop detection, (g) interrupt handling, (h) error recovery. Use mock HTTP server and deterministic tool registry.
- **Files:** `crates/opendev-agents/tests/`

### 13.2 ❌ Tool Safety Tests
- **Action:** For each dangerous/bash tool: (a) test readonly detection with edge cases (pipes, semicolons, env prefixes, redirects), (b) test dangerous command detection with known exploits, (c) fuzz test bash command parser, (d) test path validation with symlinks, UNC paths, tilde expansions.
- **Files:** `crates/opendev-tools-impl/tests/`

### 13.3 ❌ Compact & Memory Tests
- **Action:** Test: (a) message filtering preserves tool_call_id pairing, (b) compaction preserves conversation semantics, (c) memory search returns relevant results, (d) memory write gate correctly classifies all 5 tiers, (e) decay scoring produces expected ranking, (f) FTS5 search matches expected results.
- **Files:** `crates/opendev-context/tests/`, `crates/opendev-memory/tests/`

### 13.4 ❌ MCP Transport Tests
- **Action:** Test: (a) stdio transport with real MCP server (e.g., sqlite-mcp), (b) SSE transport with reconnect, (c) HTTP transport with auth, (d) WebSocket transport, (e) health monitoring with server crash, (f) tool discovery and re-discovery on notification.
- **Files:** `crates/opendev-mcp/tests/`

---

## Summary Counts

| Category | ⚠️ Partial/Fix | ❌ Missing | 🔧 Needs Audit |
|----------|---------------|-----------|---------------|
| Agent Loop | 2 | 4 | 1 |
| Tool System | 5 | 11 | 1 |
| MCP Integration | 2 | 5 | 1 |
| Security & Sandbox | 1 | 9 | 0 |
| Prompt Engineering | 3 | 4 | 0 |
| Memory & Context | 3 | 1 | 0 |
| Streaming & Provider | 4 | 3 | 0 |
| Session & Persistence | 3 | 1 | 0 |
| UI & UX | 2 | 8 | 0 |
| CLI & Entry Points | 1 | 4 | 0 |
| Plugins & Extensibility | 1 | 2 | 0 |
| Observability | 2 | 2 | 0 |
| Testing & Quality | 0 | 4 | 0 |
| **TOTAL** | **29** | **58** | **3** |
