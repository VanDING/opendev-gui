# ADR-003: Provider Adapter Pattern for LLMs

## Status

Accepted 2026-06-24

## Context

The agent must support multiple LLM providers (OpenAI, Anthropic, Google Gemini,
AWS Bedrock, Groq, Mistral, Ollama). Each provider has different:
- API shapes (endpoints, auth, streaming format)
- Message schema (roles, content types, system message placement)
- Response format (tool calls, content blocks, refusal)
- Pricing models

Embedding provider-specific logic into the agent core would make the core fragile
and hard to extend.

## Decision

LLM providers are integrated through the `ProviderAdapter` trait in `opendev-http`.
Each provider implements request/response mapping in its own adapter module
under `crates/opendev-http/src/adapters/`.

Key design:
- `ProviderAdapter` trait defines `send()` and `send_stream()` — the agent core
  calls these uniformly.
- Each adapter converts the agent's internal message format to the provider's
  API format and back.
- Provider credentials are managed by `CredentialStore` (with env var fallback).
- Circuit breaker and retry logic live in the adapter layer, not the agent core.
- The active provider is selected by configuration, not auto-detected.

Current implemented adapters: OpenAI, Anthropic, Google Gemini, AWS Bedrock,
Groq, Mistral, Ollama.

## Alternatives

- **Single provider lock-in** — simpler but unacceptable for a multi-provider tool.
- **Conditional branching in agent core** — `if provider == "openai"` scattered
  throughout the codebase; violates Open-Closed Principle.
- **FFI to provider SDKs** — adds build complexity and doesn't simplify the
  adapter logic.

## Consequences

- Adding a new provider means writing one new adapter file — no changes to agent core.
- The trait interface must be general enough to express all providers' capabilities
  (streaming, thinking blocks, tool choice).
- Providers with unique features (Anthropic's thinking, Gemini's context caching)
  need trait extensions or optional fields.
- The adapter layer duplicates some serialization code across providers.

## References

- `crates/opendev-http/src/adapters/` — adapter implementations
- `crates/opendev-http/src/adapted_client.rs` — `AdaptedClient` that uses adapters
- `crates/opendev-http/src/lib.rs` — re-exports
