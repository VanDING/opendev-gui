use crate::key::SecretKey;

/// Maps LLM provider names to their corresponding SecretKeys.
pub struct SecretProvider;

impl SecretProvider {
    /// Get the SecretKey for a given LLM provider.
    pub fn get_key(provider: &str) -> SecretKey {
        SecretKey::llm(provider)
    }

    /// Get the environment variable name for an LLM provider.
    pub fn env_var(provider: &str) -> String {
        format!("{}_API_KEY", provider.to_uppercase())
    }

    /// Well-known provider API key environment variables.
    pub fn known_env_vars() -> &'static [(&'static str, &'static str)] {
        &[
            ("openai", "OPENAI_API_KEY"),
            ("anthropic", "ANTHROPIC_API_KEY"),
            ("azure", "AZURE_OPENAI_API_KEY"),
            ("groq", "GROQ_API_KEY"),
            ("mistral", "MISTRAL_API_KEY"),
            ("deepinfra", "DEEPINFRA_API_KEY"),
            ("openrouter", "OPENROUTER_API_KEY"),
            ("fireworks", "FIREWORKS_API_KEY"),
            ("google", "GOOGLE_API_KEY"),
            ("deepseek", "DEEPSEEK_API_KEY"),
            ("cohere", "COHERE_API_KEY"),
            ("together", "TOGETHER_API_KEY"),
            ("perplexity", "PERPLEXITY_API_KEY"),
            ("xai", "XAI_API_KEY"),
        ]
    }
}
