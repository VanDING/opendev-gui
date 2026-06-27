//! Skills API routes.
//!
//! Provides endpoints for discovering, listing, and managing skills
//! that extend the agent's knowledge and capabilities.

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;

use crate::state::AppState;

/// Build the skills router.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/skills", get(list_skills))
        .route("/api/skills/{name}/pin", post(toggle_pin))
}

/// Serializable skill metadata for API responses.
#[derive(Debug, Clone, Serialize)]
struct SkillResponse {
    name: String,
    description: String,
    namespace: String,
    source: String,
    pinned: bool,
    status: String,
    usage_count: u32,
    tags: Vec<String>,
}

impl SkillResponse {
    fn from_meta(meta: &opendev_agents::SkillMetadata) -> Self {
        Self {
            name: meta.name.clone(),
            description: meta.description.clone(),
            namespace: meta.namespace.clone(),
            source: meta.source.to_string(),
            pinned: meta.pinned,
            status: format!("{:?}", meta.status),
            usage_count: meta.usage_count,
            tags: meta.tags.clone(),
        }
    }
}

/// List all discovered skills.
async fn list_skills(State(state): State<AppState>) -> Json<Vec<SkillResponse>> {
    let skills = state.with_skill_loader(|loader| loader.discover_skills()).await;

    match skills {
        Some(skills) => {
            let response: Vec<SkillResponse> =
                skills.iter().map(SkillResponse::from_meta).collect();
            Json(response)
        }
        None => Json(Vec::new()),
    }
}

/// Toggle the pinned status of a skill by name.
async fn toggle_pin(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Json<serde_json::Value> {
    let result = state
        .with_skill_loader(|loader| {
            // Ensure skills are discovered so the metadata cache is populated.
            let skills = loader.discover_skills();

            // Look up by full_name first, then fall back to bare name.
            let found = skills.iter().find(|s| s.full_name() == name || s.name == name);

            match found {
                Some(meta) => {
                    let new_pinned = !meta.pinned;
                    loader.set_pinned(&meta.full_name(), new_pinned);
                    Some(new_pinned)
                }
                None => None,
            }
        })
        .await;

    match result {
        Some(Some(pinned)) => Json(serde_json::json!({
            "status": "success",
            "pinned": pinned,
        })),
        Some(None) => Json(serde_json::json!({
            "status": "error",
            "message": format!("Skill '{}' not found", name),
        })),
        None => Json(serde_json::json!({
            "status": "error",
            "message": "Skills system not initialized",
        })),
    }
}
