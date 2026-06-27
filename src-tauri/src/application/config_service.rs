//! ConfigService — Application configuration management.
//!
//! Handles reading and updating application configuration, mode switching,
//! autonomy levels, model registry queries, and provider/model verification.

use opendev_config::ModelRegistry;
use opendev_models::AppConfig;

/// Request payload for updating config fields.
#[derive(Debug, Clone, Default)]
pub struct UpdateConfigInput {
    pub model_provider: Option<String>,
    pub model: Option<String>,
    pub model_vlm_provider: Option<String>,
    pub model_vlm: Option<String>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub enable_bash: Option<bool>,
    pub api_key: Option<String>,
    pub api_base_url: Option<String>,
}

/// Operation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationMode {
    Normal,
    Plan,
}

impl OperationMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Plan => "plan",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "normal" => Some(Self::Normal),
            "plan" => Some(Self::Plan),
            _ => None,
        }
    }
}

/// Verify model result.
#[derive(Debug, Clone)]
pub struct VerifyModelResult {
    pub valid: bool,
    pub error: Option<String>,
}

/// Provider info for frontend display.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub models: Vec<ModelInfo>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub description: String,
}

/// Mask an API key for display: "sk-****...****ab12"
fn mask_api_key(key: &str) -> String {
    if key.len() > 8 {
        format!("{}...{}", &key[..4], &key[key.len() - 4..])
    } else {
        "***".to_string()
    }
}

pub struct ConfigService {
    config: std::sync::Arc<tokio::sync::RwLock<AppConfig>>,
    model_registry: std::sync::Arc<tokio::sync::RwLock<ModelRegistry>>,
    mode: std::sync::Arc<tokio::sync::RwLock<OperationMode>>,
    autonomy_level: std::sync::Arc<tokio::sync::RwLock<String>>,
}

impl ConfigService {
    pub fn new(config: AppConfig, model_registry: ModelRegistry) -> Self {
        Self {
            config: std::sync::Arc::new(tokio::sync::RwLock::new(config)),
            model_registry: std::sync::Arc::new(tokio::sync::RwLock::new(model_registry)),
            mode: std::sync::Arc::new(tokio::sync::RwLock::new(OperationMode::Normal)),
            autonomy_level: std::sync::Arc::new(tokio::sync::RwLock::new("Manual".to_string())),
        }
    }

    /// Get a snapshot of the current config.
    pub async fn get_config(&self) -> AppConfig {
        self.config.read().await.clone()
    }

    /// Get the masked API key for display.
    pub async fn masked_api_key(&self) -> Option<String> {
        let config = self.config.read().await;
        config.api_key.as_ref().map(|k| mask_api_key(k))
    }

    /// Get current operation mode.
    pub async fn get_mode(&self) -> OperationMode {
        *self.mode.read().await
    }

    /// Set operation mode.
    pub async fn set_mode(&self, new_mode: OperationMode) {
        *self.mode.write().await = new_mode;
    }

    /// Get current autonomy level.
    pub async fn get_autonomy_level(&self) -> String {
        self.autonomy_level.read().await.clone()
    }

    /// Set autonomy level. Validates against known levels.
    pub async fn set_autonomy_level(&self, level: &str) -> Result<(), String> {
        let valid = ["Manual", "Semi-Auto", "Auto"];
        if !valid.contains(&level) {
            return Err(format!("Invalid autonomy level: {}. Must be one of {:?}", level, valid));
        }
        *self.autonomy_level.write().await = level.to_string();
        Ok(())
    }

    /// Update config fields. Only provided fields are updated.
    pub async fn update_config(&self, input: UpdateConfigInput) -> Result<(), String> {
        let mut config = self.config.write().await;
        if let Some(provider) = input.model_provider {
            config.model_provider = provider;
        }
        if let Some(model) = input.model {
            config.model = model;
        }
        if let Some(provider) = input.model_vlm_provider {
            config.model_vlm_provider = Some(provider);
        }
        if let Some(model) = input.model_vlm {
            config.model_vlm = Some(model);
        }
        if let Some(temp) = input.temperature {
            config.temperature = temp;
        }
        if let Some(max) = input.max_tokens {
            config.max_tokens = max;
        }
        if let Some(bash) = input.enable_bash {
            config.enable_bash = bash;
        }
        if let Some(key) = input.api_key {
            config.api_key = if key.is_empty() { None } else { Some(key) };
        }
        if let Some(url) = input.api_base_url {
            config.api_base_url = if url.is_empty() { None } else { Some(url) };
        }
        Ok(())
    }

    /// List all available providers with their models.
    pub async fn list_providers(&self) -> Vec<ProviderInfo> {
        let registry = self.model_registry.read().await;
        registry
            .list_providers()
            .iter()
            .map(|provider_info| {
                let models: Vec<ModelInfo> = provider_info
                    .list_models(None)
                    .iter()
                    .map(|model_info| {
                        let ctx_k = model_info.context_length / 1000;
                        let mut description = format!("{}k context", ctx_k);
                        if model_info.recommended {
                            description = format!("Recommended \u{2022} {}", description);
                        }
                        ModelInfo {
                            id: model_info.id.clone(),
                            name: model_info.name.clone(),
                            description,
                        }
                    })
                    .collect();

                ProviderInfo {
                    id: provider_info.id.clone(),
                    name: provider_info.name.clone(),
                    description: provider_info.description.clone(),
                    models,
                }
            })
            .collect()
    }

    /// Verify that a provider/model combination is valid and has credentials.
    pub async fn verify_model(&self, provider: &str, model: &str) -> VerifyModelResult {
        let registry = self.model_registry.read().await;
        let provider_info = match registry.get_provider(provider) {
            Some(p) => p,
            None => {
                return VerifyModelResult {
                    valid: false,
                    error: Some(format!("Unknown provider: {}", provider)),
                };
            }
        };

        if model.is_empty() {
            return VerifyModelResult {
                valid: false,
                error: Some("Model name cannot be empty".to_string()),
            };
        }

        let config = self.config.read().await;
        let env_var = &provider_info.api_key_env;
        let has_key = if env_var.is_empty() {
            config.api_key.is_some()
        } else {
            config.api_key.is_some() || std::env::var(env_var).is_ok()
        };

        if !has_key {
            let hint = if env_var.is_empty() {
                "No API key configured".to_string()
            } else {
                format!("No API key found. Set {} environment variable", env_var)
            };
            return VerifyModelResult { valid: false, error: Some(hint) };
        }

        VerifyModelResult { valid: true, error: None }
    }

    /// Update the model registry (e.g., from a refresh).
    pub async fn set_model_registry(&self, registry: ModelRegistry) {
        *self.model_registry.write().await = registry;
    }
}
