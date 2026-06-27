//! Session commands — DTO mapping only.

use crate::application::AppServices;
use crate::application::session_service::CreateSessionInput;
use crate::interface::desktop::contract::session::*;
use tauri::State;

/// List all sessions.
#[tauri::command]
pub async fn list_sessions(
    services: State<'_, AppServices>,
) -> Result<Vec<SessionInfoResponse>, String> {
    let sessions = services.session.list_sessions().await;
    Ok(sessions
        .into_iter()
        .map(|s| SessionInfoResponse {
            id: s.id,
            created_at: s.created_at,
            updated_at: s.updated_at,
            message_count: s.message_count,
            title: s.title,
            working_directory: s.working_directory,
        })
        .collect())
}

/// Create a new session.
#[tauri::command]
pub async fn create_session(
    services: State<'_, AppServices>,
    req: CreateSessionRequest,
) -> Result<CreateSessionResponse, String> {
    let input = CreateSessionInput { working_directory: req.working_directory };
    let (id, status) = services.session.create_session(input).await?;
    Ok(CreateSessionResponse { id, status: status.to_string() })
}

/// Get a specific session.
#[tauri::command]
pub async fn get_session(
    services: State<'_, AppServices>,
    id: String,
) -> Result<serde_json::Value, String> {
    services.session.get_session(&id).await
}

/// Delete a session.
#[tauri::command]
pub async fn delete_session(services: State<'_, AppServices>, id: String) -> Result<(), String> {
    services.session.delete_session(&id).await
}

/// Resume a session.
#[tauri::command]
pub async fn resume_session(
    services: State<'_, AppServices>,
    id: String,
) -> Result<String, String> {
    services.session.resume_session(&id).await
}

/// Get messages for a session.
#[tauri::command]
pub async fn get_session_messages(
    services: State<'_, AppServices>,
    id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let messages = services.session.get_session_messages(&id).await?;
    Ok(messages
        .iter()
        .map(|msg| {
            let mut val = serde_json::json!({
                "role": msg.role,
                "content": msg.content,
                "timestamp": msg.timestamp,
            });
            if let Some(ref reasoning) = msg.reasoning_content {
                val["reasoning_content"] = serde_json::json!(reasoning);
            }
            if let Some(ref trace) = msg.thinking_trace {
                val["thinking_trace"] = serde_json::json!(trace);
            }
            if !msg.tool_calls.is_empty() {
                let tool_calls: Vec<serde_json::Value> = msg
                    .tool_calls
                    .iter()
                    .map(|tc| {
                        let mut tcv = serde_json::json!({
                            "id": tc.id,
                            "name": tc.name,
                            "parameters": tc.parameters,
                        });
                        if let Some(ref result) = tc.result {
                            tcv["result"] = serde_json::json!(result);
                        }
                        if let Some(ref summary) = tc.result_summary {
                            tcv["result_summary"] = serde_json::json!(summary);
                        }
                        if let Some(ref error) = tc.error {
                            tcv["error"] = serde_json::json!(error);
                        }
                        if !tc.nested_tool_calls.is_empty() {
                            tcv["nested_tool_calls"] = serde_json::json!(tc.nested_tool_calls);
                        }
                        tcv
                    })
                    .collect();
                val["tool_calls"] = serde_json::json!(tool_calls);
            }
            val
        })
        .collect())
}

/// Get session model overrides.
#[tauri::command]
pub async fn get_session_model(
    services: State<'_, AppServices>,
    id: String,
) -> Result<serde_json::Value, String> {
    services.session.get_session_model(&id).await
}

/// Update session model overrides.
#[tauri::command]
pub async fn update_session_model(
    services: State<'_, AppServices>,
    id: String,
    req: SessionModelUpdateRequest,
) -> Result<(), String> {
    services.session.update_session_model(&id, req.model, req.provider).await
}

/// Clear session model overrides.
#[tauri::command]
pub async fn clear_session_model(
    services: State<'_, AppServices>,
    id: String,
) -> Result<(), String> {
    services.session.clear_session_model(&id).await
}
