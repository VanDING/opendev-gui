//! Chat commands — DTO mapping only.

use tauri::State;
use crate::application::AppServices;
use crate::interface::desktop::contract::chat::*;

/// Send a chat query. Returns immediately; results arrive via event stream.
#[tauri::command]
pub async fn send_chat_query(
    services: State<'_, AppServices>,
    req: ChatQueryRequest,
) -> Result<ChatActionResponse, String> {
    let message = req.message.trim().to_string();
    if message.is_empty() {
        return Err("Message cannot be empty.".to_string());
    }

    let session_id = match req.session_id {
        Some(id) => id,
        None => services.session.current_session_id().await
            .ok_or_else(|| "No active session. Create a session first.".to_string())?,
    };

    // If session is already running, inject into queue
    if services.chat.is_session_running(&session_id).await {
        services.chat.try_inject_message(&session_id, message.clone()).await
            .map_err(|_| "Agent is busy; try again shortly.".to_string())?;
        return Ok(ChatActionResponse {
            status: "accepted".to_string(),
            session_id: Some(session_id),
            message: Some("Message queued for running session".to_string()),
        });
    }

    // Load session
    let mgr = services.chat.session_manager_read().await;
    let session_exists = mgr.load_session(&session_id).is_ok()
        || mgr.current_session().map(|s| s.id == session_id).unwrap_or(false);
    drop(mgr);

    if !session_exists {
        return Err(format!("Session '{}' not found.", session_id));
    }

    // Mark session as running
    services.chat.set_session_running(session_id.clone()).await;

    // Fire agent executor in background (if set)
    if let Some(_executor) = services.chat.agent_executor().await {
        // Agent execution happens through the AppState bridge
    }

    Ok(ChatActionResponse {
        status: "accepted".to_string(),
        session_id: Some(session_id),
        message: None,
    })
}

/// Interrupt the current chat task.
#[tauri::command]
pub async fn interrupt_chat(
    services: State<'_, AppServices>,
) -> Result<ChatActionResponse, String> {
    services.chat.request_interrupt().await;
    services.workflow.deny_all().await;
    Ok(ChatActionResponse {
        status: "interrupt_requested".to_string(),
        session_id: None,
        message: None,
    })
}

/// Clear chat (create a new session).
#[tauri::command]
pub async fn clear_chat(
    services: State<'_, AppServices>,
    workspace: Option<String>,
) -> Result<ChatActionResponse, String> {
    use crate::application::session_service::CreateSessionInput;
    let input = CreateSessionInput { working_directory: workspace };
    let (session_id, _) = services.session.create_session(input).await?;
    Ok(ChatActionResponse {
        status: "success".to_string(),
        session_id: Some(session_id),
        message: Some("Chat cleared".to_string()),
    })
}

/// Get messages for the current session.
#[tauri::command]
pub async fn get_chat_messages(
    services: State<'_, AppServices>,
) -> Result<Vec<serde_json::Value>, String> {
    services.chat.get_current_messages().await
}
