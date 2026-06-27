//! ChatService — Chat query execution and lifecycle.
//!
//! Handles sending queries to the agent, managing running sessions,
//! and interrupting/cancelling active tasks.

use std::collections::HashMap;
use tokio::sync::Mutex;

use opendev_history::SessionManager;
use opendev_models::AppConfig;

/// Agent executor trait — the bridge between chat and agent execution.
#[async_trait::async_trait]
pub trait AgentExecutor: Send + Sync + 'static {
    async fn execute_query(
        &self,
        message: String,
        session_id: String,
        state: Box<dyn std::any::Any + Send>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

pub struct ChatService {
    session_manager: std::sync::Arc<tokio::sync::RwLock<SessionManager>>,
    config: std::sync::Arc<tokio::sync::RwLock<AppConfig>>,
    running_sessions: Mutex<HashMap<String, String>>,
    injection_queues: Mutex<HashMap<String, tokio::sync::mpsc::Sender<String>>>,
    agent_executor: Mutex<Option<std::sync::Arc<dyn AgentExecutor>>>,
    interrupt_requested: Mutex<bool>,
}

impl ChatService {
    pub fn new(
        session_manager: SessionManager,
        config: AppConfig,
    ) -> Self {
        Self {
            session_manager: std::sync::Arc::new(tokio::sync::RwLock::new(session_manager)),
            config: std::sync::Arc::new(tokio::sync::RwLock::new(config)),
            running_sessions: Mutex::new(HashMap::new()),
            injection_queues: Mutex::new(HashMap::new()),
            agent_executor: Mutex::new(None),
            interrupt_requested: Mutex::new(false),
        }
    }

    /// Set the agent executor implementation.
    pub fn set_agent_executor(&self, executor: std::sync::Arc<dyn AgentExecutor>) {
        let mut guard = self.agent_executor.blocking_lock();
        *guard = Some(executor);
    }

    /// Get the agent executor, if set.
    pub async fn agent_executor(&self) -> Option<std::sync::Arc<dyn AgentExecutor>> {
        self.agent_executor.lock().await.clone()
    }

    /// Get messages for the current session (filtered for display).
    pub async fn get_current_messages(
        &self,
    ) -> Result<Vec<serde_json::Value>, String> {
        let mgr = self.session_manager.read().await;
        let session = mgr
            .current_session()
            .ok_or_else(|| "No active session".to_string())?;

        let messages: Vec<serde_json::Value> = session
            .messages
            .iter()
            .filter(|msg| {
                // Skip system-injected messages
                if msg.metadata.contains_key("_msg_class") {
                    return false;
                }
                if msg.role == opendev_models::message::Role::User
                    && opendev_models::message::is_system_injected_content(&msg.content)
                {
                    return false;
                }
                if msg.role == opendev_models::message::Role::System {
                    return false;
                }
                true
            })
            .map(|msg| {
                let mut val = serde_json::json!({
                    "role": msg.role,
                    "content": msg.content,
                    "timestamp": msg.timestamp,
                    "tool_calls": msg.tool_calls.iter()
                        .filter(|tc| tc.name != "task_complete")
                        .count(),
                });
                if let Some(ref reasoning) = msg.reasoning_content {
                    val["reasoning_content"] = serde_json::json!(reasoning);
                }
                if let Some(ref trace) = msg.thinking_trace {
                    val["thinking_trace"] = serde_json::json!(trace);
                }
                val
            })
            .collect();

        Ok(messages)
    }

    /// Check if a session is currently running.
    pub async fn is_session_running(&self, session_id: &str) -> bool {
        self.running_sessions.lock().await.contains_key(session_id)
    }

    /// Get the set of running session IDs.
    pub async fn running_sessions(&self) -> Vec<String> {
        self.running_sessions.lock().await.keys().cloned().collect()
    }

    /// Mark a session as running.
    pub async fn set_session_running(&self, session_id: String) {
        self.running_sessions
            .lock()
            .await
            .insert(session_id, "running".to_string());
    }

    /// Mark a session as idle.
    pub async fn set_session_idle(&self, session_id: &str) {
        self.running_sessions.lock().await.remove(session_id);
    }

    /// Try to inject a message into a running session's queue.
    pub async fn try_inject_message(
        &self,
        session_id: &str,
        message: String,
    ) -> Result<(), String> {
        let mut queues = self.injection_queues.lock().await;
        if let Some(tx) = queues.get(session_id) {
            tx.send(message)
                .await
                .map_err(|_| "Injection queue full or closed".to_string())?;
            Ok(())
        } else {
            Err("Session not found or injection queue not available".to_string())
        }
    }

    /// Register an injection queue for a session.
    pub async fn register_injection_queue(
        &self,
        session_id: &str,
        tx: tokio::sync::mpsc::Sender<String>,
    ) {
        self.injection_queues
            .lock()
            .await
            .insert(session_id.to_string(), tx);
    }

    /// Remove an injection queue.
    pub async fn remove_injection_queue(&self, session_id: &str) {
        self.injection_queues.lock().await.remove(session_id);
    }

    /// Mark that an interrupt has been requested.
    pub async fn request_interrupt(&self) {
        *self.interrupt_requested.lock().await = true;
    }

    /// Clear the interrupt flag.
    pub async fn clear_interrupt(&self) {
        *self.interrupt_requested.lock().await = false;
    }

    /// Check if interrupt has been requested.
    pub async fn is_interrupt_requested(&self) -> bool {
        *self.interrupt_requested.lock().await
    }

    /// Get a reference to the session manager for reading.
    pub async fn session_manager_read(
        &self,
    ) -> tokio::sync::RwLockReadGuard<'_, SessionManager> {
        self.session_manager.read().await
    }

    /// Get a reference to the config for reading.
    pub async fn config_read(&self) -> tokio::sync::RwLockReadGuard<'_, AppConfig> {
        self.config.read().await
    }
}
