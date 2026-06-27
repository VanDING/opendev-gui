//! Skills commands — DTO mapping only.

use crate::application::AppServices;
use crate::interface::desktop::contract::skills::*;
use tauri::State;

/// List all discovered skills.
#[tauri::command]
pub async fn list_skills(services: State<'_, AppServices>) -> Result<Vec<SkillResponse>, String> {
    let skills = services.skill.list_skills();
    Ok(skills
        .into_iter()
        .map(|s| SkillResponse {
            name: s.name,
            description: s.description,
            namespace: s.namespace,
            source: s.source,
            pinned: s.pinned,
            status: s.status,
            usage_count: s.usage_count,
            tags: s.tags,
        })
        .collect())
}

/// Toggle skill pin status.
#[tauri::command]
pub async fn toggle_skill_pin(
    services: State<'_, AppServices>,
    name: String,
) -> Result<TogglePinResponse, String> {
    match services.skill.toggle_pin(&name) {
        Ok(pinned) => Ok(TogglePinResponse {
            status: "success".to_string(),
            pinned: Some(pinned),
            message: None,
        }),
        Err(msg) => {
            Ok(TogglePinResponse { status: "error".to_string(), pinned: None, message: Some(msg) })
        }
    }
}
