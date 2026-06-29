use super::*;
use opendev_secrets::ChainedSecretStore;

/// Map provider names to their standard environment variable names.
fn env_var_for_provider(provider: &str) -> Option<&'static str> {
    match provider {
        "openai" => Some("OPENAI_API_KEY"),
        "anthropic" => Some("ANTHROPIC_API_KEY"),
        "gemini" | "google" => Some("GOOGLE_API_KEY"),
        "groq" => Some("GROQ_API_KEY"),
        "fireworks" => Some("FIREWORKS_API_KEY"),
        "mistral" => Some("MISTRAL_API_KEY"),
        "deepinfra" => Some("DEEPINFRA_API_KEY"),
        "openrouter" => Some("OPENROUTER_API_KEY"),
        _ => None,
    }
}

#[test]
fn test_env_var_for_provider() {
    assert_eq!(env_var_for_provider("openai"), Some("OPENAI_API_KEY"));
    assert_eq!(env_var_for_provider("anthropic"), Some("ANTHROPIC_API_KEY"));
    assert_eq!(env_var_for_provider("unknown"), None);
}

#[tokio::test]
async fn test_credential_store_set_get() {
    let store = CredentialStore::new(Arc::new(ChainedSecretStore::new(None, None)));
    assert!(store.get_key("testprovider").await.is_none());
}

#[tokio::test]
async fn test_credential_store_list() {
    let store = CredentialStore::new(Arc::new(ChainedSecretStore::new(None, None)));
    let providers = store.list_providers().await;
    assert!(!providers.is_empty());
}

#[tokio::test]
async fn test_credential_store_env_var() {
    let store = CredentialStore::new(Arc::new(ChainedSecretStore::new(None, None)));
    assert!(store.get_key("testprovider_nonexistent_xyz").await.is_none());
}
