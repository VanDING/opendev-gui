# How to Add a New LLM Provider

This guide describes how to add a new LLM provider (e.g., DeepSeek, Cohere, etc.)
to OpenDev Desktop.

## Overview

LLM providers are integrated through the `ProviderAdapter` trait. The agent core calls
`send()` and `send_stream()` uniformly — the adapter handles the provider-specific
request/response mapping.

## Steps

### 1. Create the adapter file

Create `crates/opendev-http/src/adapters/{provider_name}.rs`:

```rust
use async_trait::async_trait;
use opendev_models::message::ChatMessage;
use crate::adapters::{ProviderAdapter, ProviderError, ProviderResponse};

pub struct ProviderNameAdapter {
    api_key: String,
    model: String,
}

#[async_trait]
impl ProviderAdapter for ProviderNameAdapter {
    fn provider_name(&self) -> &'static str {
        "provider_name"
    }

    async fn send(&self, messages: &[ChatMessage]) -> Result<ProviderResponse, ProviderError> {
        // 1. Convert ChatMessage → Provider's request format
        // 2. Send HTTP request
        // 3. Parse response → ProviderResponse
    }

    async fn send_stream(
        &self,
        messages: &[ChatMessage],
    ) -> Result<Box<dyn Stream<Item = Result<ProviderChunk, ProviderError>> + Send>, ProviderError> {
        // 1. Convert ChatMessage → Provider's request format
        // 2. Send streaming HTTP request
        // 3. Return stream of ProviderChunk
    }
}
```

### 2. Register in the adapter module

Edit `crates/opendev-http/src/adapters/mod.rs`:

```rust
mod provider_name;       // Add
pub use provider_name::ProviderNameAdapter;  // Add
```

### 3. Add to provider factory

Edit `crates/opendev-http/src/adapted_client.rs`:

```rust
"provider_name" => Box::new(ProviderNameAdapter::new(api_key, model)),
```

### 4. Add pricing config

Add model pricing to `opendev-config` or the relevant pricing configuration.

### 5. Test

```bash
cargo test -p opendev-http
cargo check --workspace
```

### 6. Add to the frontend provider list

Update the provider selection UI in `src/components/Settings/ModelSettings.tsx`.

## What You Need to Know

- The `ProviderAdapter` trait handles all provider types uniformly. Your adapter
  is responsible for converting between the internal `ChatMessage` format and
  the provider's API format.
- Streaming is important — implement `send_stream` for real-time response display.
- Error handling (auth errors, rate limits, server errors) should return appropriate
  `ProviderError` variants.
- The provider name string must match between the adapter, configuration, and
  frontend selection.

## Existing Adapters (Reference)

See `crates/opendev-http/src/adapters/` for existing implementations:
- `openai.rs` — OpenAI-compatible APIs
- `anthropic.rs` — Anthropic Claude
- `gemini.rs` — Google Gemini
- `bedrock.rs` — AWS Bedrock
- `groq.rs`, `mistral.rs`, `ollama.rs` — other providers
