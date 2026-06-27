//! SessionService — Session lifecycle management.
//!
//! Handles CRUD operations on sessions, session model overrides,
//! session message access, and bridge mode info.

use opendev_history::SessionManager;
use opendev_models::Session;

/// Create session request data.
#[derive(Debug, Clone, Default)]
pub struct CreateSessionInput {
    pub working_directory: Option<String>,
}

/// Session info for frontend display.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SessionInfo {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub message_count: usize,
    pub title: Option<String>,
    pub working_directory: Option<String>,
}

pub struct SessionService {
    session_manager: std::sync::Arc<tokio::sync::RwLock<SessionManager>>,
    working_dir: String,
}

impl SessionService {
    pub fn new(session_manager: SessionManager, working_dir: String) -> Self {
        Self {
            session_manager: std::sync::Arc::new(tokio::sync::RwLock::new(session_manager)),
            working_dir,
        }
    }

    /// Get the working directory.
    pub fn working_dir(&self) -> &str {
        &self.working_dir
    }

    /// List all sessions from the index.
    pub async fn list_sessions(&self) -> Vec<SessionInfo> {
        let mgr = self.session_manager.read().await;
        let index = mgr.index().read_index();
        match index {
            Some(idx) => idx
                .entries
                .iter()
                .map(|entry| SessionInfo {
                    id: entry.session_id.clone(),
                    created_at: entry.created.clone(),
                    updated_at: entry.modified.clone(),
                    message_count: entry.message_count as usize,
                    title: entry.title.clone(),
                    working_directory: entry.working_directory.clone(),
                })
                .collect(),
            None => Vec::new(),
        }
    }

    /// Get the current session ID.
    pub async fn current_session_id(&self) -> Option<String> {
        let mgr = self.session_manager.read().await;
        mgr.current_session().map(|s| s.id.clone())
    }

    /// Get the current session details.
    pub async fn current_session(&self) -> Option<Session> {
        let mgr = self.session_manager.read().await;
        mgr.current_session().cloned()
    }

    /// Create a new session, optionally reusing an empty session for the same workspace.
    pub async fn create_session(
        &self,
        input: CreateSessionInput,
    ) -> Result<(String, &'static str), String> {
        let mut mgr = self.session_manager.write().await;

        // Try to reuse an existing empty session for the same workspace.
        if let Some(ref wd) = input.working_directory {
            if let Some(index) = mgr.index().read_index() {
                let empty_match = index.entries.iter().find(|entry| {
                    entry.message_count == 0
                        && entry.working_directory.as_deref() == Some(wd.as_str())
                });

                if let Some(entry) = empty_match {
                    let candidate_id = entry.session_id.clone();
                    let is_stale = mgr
                        .current_session()
                        .map(|s| s.id == candidate_id && !s.messages.is_empty())
                        .unwrap_or(false);

                    if !is_stale && mgr.resume_session(&candidate_id).is_ok() {
                        return Ok((candidate_id, "reused"));
                    }
                }
            }
        }

        // No reusable session found — create a new one.
        let session = mgr.create_session();
        let session_id = session.id.clone();

        if let Some(wd) = input.working_directory {
            if let Some(s) = mgr.current_session_mut() {
                s.working_directory = Some(wd);
            }
        }

        mgr.save_current()
            .map_err(|e| format!("Failed to save session: {}", e))?;

        Ok((session_id, "created"))
    }

    /// Get session metadata by ID.
    pub async fn get_session(&self, id: &str) -> Result<serde_json::Value, String> {
        let mgr = self.session_manager.read().await;
        let session = mgr
            .load_session(id)
            .map_err(|e| format!("Session {} not found: {}", id, e))?;
        serde_json::to_value(session.get_metadata())
            .map_err(|e| format!("Failed to serialize session: {}", e))
    }

    /// Delete a session by ID.
    pub async fn delete_session(&self, id: &str) -> Result<(), String> {
        let mut mgr = self.session_manager.write().await;

        // Verify session exists.
        mgr.load_session(id)
            .map_err(|e| format!("Session {} not found: {}", id, e))?;

        // Delete session files.
        let session_dir = mgr.session_dir().to_path_buf();
        let json_path = session_dir.join(format!("{}.json", id));
        let jsonl_path = session_dir.join(format!("{}.jsonl", id));
        let debug_path = session_dir.join(format!("{}.debug", id));

        if json_path.exists() {
            std::fs::remove_file(&json_path)
                .map_err(|e| format!("Failed to delete session file: {}", e))?;
        }
        if jsonl_path.exists() {
            std::fs::remove_file(&jsonl_path)
                .map_err(|e| format!("Failed to delete session transcript: {}", e))?;
        }
        if debug_path.exists() {
            let _ = std::fs::remove_file(&debug_path);
        }

        // Remove from index.
        mgr.index()
            .remove_entry(id)
            .map_err(|e| format!("Failed to update index: {}", e))?;

        // Clear current session if it was the deleted one.
        if mgr.current_session().map(|s| s.id == id).unwrap_or(false) {
            mgr.set_current_session(Session::new());
        }

        Ok(())
    }

    /// Resume a session.
    pub async fn resume_session(&self, id: &str) -> Result<String, String> {
        let mut mgr = self.session_manager.write().await;
        mgr.resume_session(id)
            .map_err(|e| format!("Session {} not found: {}", id, e))?;
        Ok(id.to_string())
    }

    /// Get messages for a session.
    pub async fn get_session_messages(
        &self,
        id: &str,
    ) -> Result<Vec<opendev_models::ChatMessage>, String> {
        let mgr = self.session_manager.read().await;
        let session = mgr
            .load_session(id)
            .map_err(|e| format!("Session {} not found: {}", id, e))?;
        Ok(session.messages.clone())
    }

    /// Get session model overrides from metadata.
    pub async fn get_session_model(
        &self,
        session_id: &str,
    ) -> Result<serde_json::Value, String> {
        let mgr = self.session_manager.read().await;
        let session = mgr
            .load_session(session_id)
            .map_err(|e| format!("Session {} not found: {}", session_id, e))?;
        let overlay = session.metadata.get("session_model").cloned().unwrap_or(serde_json::json!({}));
        Ok(overlay)
    }

    /// Update session model overrides stored in metadata["session_model"].
    pub async fn update_session_model(
        &self,
        session_id: &str,
        model: Option<String>,
        provider: Option<String>,
    ) -> Result<(), String> {
        let mgr = self.session_manager.read().await;
        let mut session = mgr
            .load_session(session_id)
            .map_err(|e| format!("Session {} not found: {}", session_id, e))?
            .clone();
        drop(mgr);

        let current = session.metadata.get("session_model")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();
        let mut overlay = current;
        if let Some(m) = model {
            if m.is_empty() {
                overlay.remove("model");
            } else {
                overlay.insert("model".to_string(), serde_json::json!(m));
            }
        }
        if let Some(p) = provider {
            if p.is_empty() {
                overlay.remove("model_provider");
            } else {
                overlay.insert("model_provider".to_string(), serde_json::json!(p));
            }
        }

        if overlay.is_empty() {
            session.metadata.remove("session_model");
        } else {
            session.metadata.insert("session_model".to_string(), serde_json::Value::Object(overlay));
        }

        let mut mgr = self.session_manager.write().await;
        mgr.save_session(&session)
            .map_err(|e| format!("Failed to save session: {}", e))?;
        Ok(())
    }

    /// Clear session model overrides.
    pub async fn clear_session_model(&self, session_id: &str) -> Result<(), String> {
        let mgr = self.session_manager.read().await;
        let mut session = mgr
            .load_session(session_id)
            .map_err(|e| format!("Session {} not found: {}", session_id, e))?
            .clone();
        drop(mgr);

        session.metadata.remove("session_model");

        let mut mgr = self.session_manager.write().await;
        mgr.save_session(&session)
            .map_err(|e| format!("Failed to save session: {}", e))?;
        Ok(())
    }
}
