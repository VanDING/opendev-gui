//! Config commands — DTO mapping only.

use tauri::State;
use crate::application::AppServices;
use crate::application::config_service::{UpdateConfigInput, OperationMode};
use crate::interface::desktop::contract::config::*;

/// Get current application configuration.
#[tauri::command]
pub async fn get_app_config(
    services: State<'_, AppServices>,
) -> Result<ConfigResponse, String> {
    let cfg = services.config.get_config().await;
    let mode = services.config.get_mode().await;
    let autonomy_level = services.config.get_autonomy_level().await;
    let git_branch = services.system.git_branch();
    let masked_key = services.config.masked_api_key().await;

    // Resolve compact agent role
    let (compact_model, compact_provider) = cfg.resolve_agent_role("compact");
    let compact_model_opt = if compact_model == cfg.model && compact_provider == cfg.model_provider {
        None
    } else {
        Some(compact_model)
    };
    let compact_provider_opt = if compact_model_opt.is_none() { None } else { Some(compact_provider) };

    Ok(ConfigResponse {
        model_provider: cfg.model_provider,
        model: cfg.model,
        model_vlm_provider: cfg.model_vlm_provider,
        model_vlm: cfg.model_vlm,
        model_compact_provider: compact_provider_opt,
        model_compact: compact_model_opt,
        api_key: masked_key,
        api_base_url: cfg.api_base_url,
        temperature: cfg.temperature,
        max_tokens: cfg.max_tokens,
        enable_bash: cfg.enable_bash,
        mode: mode.as_str().to_string(),
        autonomy_level,
        working_dir: services.system.working_dir().to_string(),
        git_branch,
    })
}

/// Update application configuration.
#[tauri::command]
pub async fn update_app_config(
    services: State<'_, AppServices>,
    req: UpdateConfigRequest,
) -> Result<(), String> {
    let input = UpdateConfigInput {
        model_provider: req.model_provider,
        model: req.model,
        model_vlm_provider: req.model_vlm_provider,
        model_vlm: req.model_vlm,
        temperature: req.temperature,
        max_tokens: req.max_tokens,
        enable_bash: req.enable_bash,
        api_key: req.api_key,
        api_base_url: req.api_base_url,
    };
    services.config.update_config(input).await
}

/// Set operation mode.
#[tauri::command]
pub async fn set_operation_mode(
    services: State<'_, AppServices>,
    req: ModeUpdateRequest,
) -> Result<(), String> {
    let mode = OperationMode::from_str(&req.mode)
        .ok_or_else(|| format!("Invalid mode: {}", req.mode))?;
    services.config.set_mode(mode).await;
    Ok(())
}

/// Set autonomy level.
#[tauri::command]
pub async fn set_autonomy_level(
    services: State<'_, AppServices>,
    req: AutonomyUpdateRequest,
) -> Result<(), String> {
    services.config.set_autonomy_level(&req.level).await
}

/// List available model providers.
#[tauri::command]
pub async fn list_model_providers(
    services: State<'_, AppServices>,
) -> Result<Vec<serde_json::Value>, String> {
    let providers = services.config.list_providers().await;
    Ok(providers.into_iter().map(|p| {
        serde_json::json!({
            "id": p.id,
            "name": p.name,
            "description": p.description,
            "models": p.models.into_iter().map(|m| {
                serde_json::json!({
                    "id": m.id,
                    "name": m.name,
                    "description": m.description,
                })
            }).collect::<Vec<_>>(),
        })
    }).collect())
}

/// Verify model availability.
#[tauri::command]
pub async fn verify_model(
    services: State<'_, AppServices>,
    req: VerifyModelRequest,
) -> Result<VerifyModelResponse, String> {
    let result = services.config.verify_model(&req.provider, &req.model).await;
    Ok(VerifyModelResponse {
        valid: result.valid,
        error: result.error,
    })
}
