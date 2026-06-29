//! Health monitoring, ping checks, and health state machine for MCP servers.
//!
//! Restart orchestration with exponential backoff lives in [`restart`].

mod restart;

use std::collections::HashMap;
use std::sync::Arc;

use tracing::{debug, error, info, warn};

use crate::config::prepare_server_config;
use crate::models::JsonRpcRequest;
use crate::transport;

use super::{
    HEALTH_CHECK_FAILURE_THRESHOLD, MAX_BACKOFF_SECS, MAX_RESTART_ATTEMPTS, McpManager,
    ServerConnection,
};

/// Number of consecutive crash/close events within the window to mark permanently failed.
const CRASH_THRESHOLD: usize = 5;

/// Time window (in seconds) for tracking crash frequency.
const CRASH_WINDOW_SECS: u64 = 60;

/// Health status of a server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerHealthStatus {
    /// Server is healthy and responding to pings.
    Healthy,
    /// Server has failed some health checks but is still within threshold.
    Degraded,
    /// Server is unavailable due to authentication failure (HTTP 401 / token expiry).
    NeedsAuth,
    /// Server has failed enough health checks to be marked unhealthy.
    Unhealthy,
    /// Server has exceeded max restart attempts and is permanently failed.
    PermanentlyFailed,
}

/// Health tracking state for a server.
#[derive(Debug, Clone)]
pub struct ServerHealthState {
    /// Current health status.
    pub status: ServerHealthStatus,
    /// Number of consecutive failed health checks.
    pub consecutive_failures: u32,
    /// Number of restart attempts made.
    pub restart_attempts: u32,
    /// Whether tools have been removed from the registry due to unhealthy status.
    pub tools_removed: bool,
    /// Timestamps of recent crash/close events for rapid-failure detection.
    pub crash_timestamps: Vec<chrono::DateTime<chrono::Utc>>,
}

impl Default for ServerHealthState {
    fn default() -> Self {
        Self {
            status: ServerHealthStatus::Healthy,
            consecutive_failures: 0,
            restart_attempts: 0,
            tools_removed: false,
            crash_timestamps: Vec::new(),
        }
    }
}

impl ServerHealthState {
    /// Calculate the next backoff duration for restart attempts.
    pub fn next_backoff_secs(&self) -> u64 {
        let backoff = 1u64 << self.restart_attempts.min(6);
        backoff.min(MAX_BACKOFF_SECS)
    }

    /// Record a crash event and check if the server should be permanently failed
    /// due to too many crashes in a short window.
    ///
    /// Returns `true` if the server should be marked `PermanentlyFailed`.
    pub fn record_crash(&mut self) -> bool {
        let now = chrono::Utc::now();
        self.crash_timestamps.push(now);

        // Prune timestamps outside the window
        let cutoff = now - chrono::Duration::seconds(CRASH_WINDOW_SECS as i64);
        self.crash_timestamps.retain(|&t| t > cutoff);

        // Check if we've hit the threshold
        if self.crash_timestamps.len() >= CRASH_THRESHOLD {
            self.status = ServerHealthStatus::PermanentlyFailed;
            true
        } else {
            false
        }
    }

    /// Mark the server as needing authentication (e.g., HTTP 401).
    pub fn mark_needs_auth(&mut self) {
        self.status = ServerHealthStatus::NeedsAuth;
        self.tools_removed = true;
    }

    /// Return a short label for use in system prompts.
    pub fn status_label(&self) -> &str {
        match self.status {
            ServerHealthStatus::Healthy => "healthy",
            ServerHealthStatus::Degraded => "degraded",
            ServerHealthStatus::NeedsAuth => "needs-auth",
            ServerHealthStatus::Unhealthy => "unhealthy",
            ServerHealthStatus::PermanentlyFailed => "permanently-failed",
        }
    }
}

impl McpManager {
    /// Perform a health check (ping) on a specific server.
    ///
    /// Sends a `ping` JSON-RPC request and checks for a response.
    /// Returns `true` if the server responded, `false` otherwise.
    pub async fn ping_server(&self, server_name: &str) -> bool {
        let connections = self.connections.read().await;
        let conn = match connections.get(server_name) {
            Some(c) => c,
            None => return false,
        };

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: self.next_request_id(),
            method: "ping".to_string(),
            params: None,
        };

        match conn.transport.send_request(&request).await {
            Ok(_) => true,
            Err(e) => {
                debug!(
                    server = server_name,
                    error = %e,
                    "Health check ping failed"
                );
                false
            }
        }
    }

    /// Record a health check result for a server.
    ///
    /// Updates the health state and takes appropriate action:
    /// - On success: resets consecutive failure count.
    /// - On failure: increments consecutive failure count.
    /// - When threshold is reached: marks unhealthy, removes tools, attempts restart.
    /// - HTTP 401 / auth errors: marks `NeedsAuth` instead of retrying.
    pub async fn record_health_check(&self, server_name: &str, success: bool) {
        let mut health_states = self.health_states.write().await;
        let state = health_states.entry(server_name.to_string()).or_default();

        if state.status == ServerHealthStatus::PermanentlyFailed
            || state.status == ServerHealthStatus::NeedsAuth
        {
            return;
        }

        if success {
            state.consecutive_failures = 0;
            if state.status == ServerHealthStatus::Degraded {
                state.status = ServerHealthStatus::Healthy;
                info!(server = server_name, "MCP server health restored");
            }
            return;
        }

        // Failure path.
        state.consecutive_failures += 1;
        debug!(
            server = server_name,
            consecutive_failures = state.consecutive_failures,
            "MCP server health check failed"
        );

        if state.consecutive_failures < HEALTH_CHECK_FAILURE_THRESHOLD {
            state.status = ServerHealthStatus::Degraded;
        } else if !state.tools_removed {
            // Threshold reached: mark unhealthy and remove tools.
            state.status = ServerHealthStatus::Unhealthy;
            state.tools_removed = true;
            warn!(
                server = server_name,
                "MCP server marked unhealthy after {} consecutive failures. \
                 Removing its tools from the active registry.",
                state.consecutive_failures
            );
            // Drop the lock before calling async methods.
            drop(health_states);
            self.remove_failed_server(server_name).await;
        }
    }

    /// Record an auth failure (HTTP 401) for a server.
    ///
    /// Instead of retrying, the server is marked `NeedsAuth` so the system
    /// can prompt the user to re-authenticate.
    pub async fn record_auth_failure(&self, server_name: &str) {
        let mut health_states = self.health_states.write().await;
        let state = health_states.entry(server_name.to_string()).or_default();
        state.mark_needs_auth();
        warn!(
            server = server_name,
            "MCP server requires authentication (HTTP 401)"
        );
        drop(health_states);
        self.remove_failed_server(server_name).await;
    }

    /// Record a crash/close event for rapid-failure detection.
    ///
    /// If the server crashes 5+ times within 60 seconds, it is permanently failed.
    pub async fn record_crash(&self, server_name: &str) {
        let mut health_states = self.health_states.write().await;
        let state = health_states.entry(server_name.to_string()).or_default();

        if state.status == ServerHealthStatus::PermanentlyFailed {
            return;
        }

        if state.record_crash() {
            error!(
                server = server_name,
                "Server permanently failed after {} crashes within {}s",
                CRASH_THRESHOLD,
                CRASH_WINDOW_SECS,
            );
        }
    }

    /// Get the health state of a specific server.
    pub async fn get_health_state(&self, server_name: &str) -> Option<ServerHealthState> {
        let health_states = self.health_states.read().await;
        health_states.get(server_name).cloned()
    }

    /// Get health states for all servers.
    pub async fn get_all_health_states(&self) -> HashMap<String, ServerHealthState> {
        let health_states = self.health_states.read().await;
        health_states.clone()
    }

    /// Build a health status summary suitable for inclusion in system prompts.
    ///
    /// Returns a list of `(server_name, status_label)` pairs for all tracked servers.
    pub async fn health_status_summary(&self) -> Vec<(String, String)> {
        let health_states = self.health_states.read().await;
        let mut summary: Vec<(String, String)> = health_states
            .iter()
            .map(|(name, state)| (name.clone(), state.status_label().to_string()))
            .collect();
        summary.sort_by(|a, b| a.0.cmp(&b.0));
        summary
    }

    /// Start background health monitoring.
    ///
    /// Spawns a task that periodically pings all connected servers.
    /// When a server becomes unhealthy, its tools are removed and a
    /// restart is attempted.
    pub async fn start_health_monitoring(&self) {
        if self.health_check_interval_secs == 0 {
            debug!("Health monitoring disabled (interval = 0)");
            return;
        }

        let interval = std::time::Duration::from_secs(self.health_check_interval_secs);
        let connections = Arc::clone(&self.connections);
        let health_states = Arc::clone(&self.health_states);
        let request_id = Arc::clone(&self.request_id);
        let config = Arc::clone(&self.config);

        let tool_cache = Arc::clone(&self.tool_schema_cache);
        let _working_dir = self.working_dir.clone();

        let handle = tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            ticker.tick().await; // First tick is immediate; skip it.

            loop {
                ticker.tick().await;

                // Collect current server names.
                let server_names: Vec<String> = {
                    let conns = connections.read().await;
                    conns.keys().cloned().collect()
                };

                // Also check servers that were recently marked unhealthy
                // (not permanently failed) for restart.
                let unhealthy_servers: Vec<String> = {
                    let states = health_states.read().await;
                    states
                        .iter()
                        .filter(|(_, s)| s.status == ServerHealthStatus::Unhealthy)
                        .map(|(name, _)| name.clone())
                        .collect()
                };

                // Ping connected servers.
                for name in &server_names {
                    let ping_ok = {
                        let conns = connections.read().await;
                        if let Some(conn) = conns.get(name.as_str()) {
                            let req = JsonRpcRequest {
                                jsonrpc: "2.0".to_string(),
                                id: request_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
                                method: "ping".to_string(),
                                params: None,
                            };
                            conn.transport.send_request(&req).await.is_ok()
                        } else {
                            false
                        }
                    };

                    // Update health state.
                    let mut states = health_states.write().await;
                    let state = states.entry(name.clone()).or_default();

                    if state.status == ServerHealthStatus::PermanentlyFailed
                        || state.status == ServerHealthStatus::NeedsAuth
                    {
                        continue;
                    }

                    if ping_ok {
                        state.consecutive_failures = 0;
                        if state.status == ServerHealthStatus::Degraded {
                            state.status = ServerHealthStatus::Healthy;
                            info!(server = %name, "MCP server health restored");
                        }
                    } else {
                        state.consecutive_failures += 1;
                        debug!(
                            server = %name,
                            failures = state.consecutive_failures,
                            "Health check ping failed"
                        );

                        if state.consecutive_failures >= HEALTH_CHECK_FAILURE_THRESHOLD
                            && !state.tools_removed
                        {
                            state.status = ServerHealthStatus::Unhealthy;
                            state.tools_removed = true;
                            warn!(
                                server = %name,
                                "MCP server marked unhealthy, removing tools"
                            );
                            // Remove the server connection.
                            drop(states);
                            let mut conns = connections.write().await;
                            if let Some(conn) = conns.remove(name.as_str())
                                && let Err(e) = conn.transport.close().await
                            {
                                debug!("Error closing unhealthy server '{}': {}", name, e);
                            }
                        } else if state.consecutive_failures < HEALTH_CHECK_FAILURE_THRESHOLD {
                            state.status = ServerHealthStatus::Degraded;
                        }
                    }
                }

                // Attempt restart for unhealthy servers.
                for name in &unhealthy_servers {
                    let should_restart = {
                        let states = health_states.read().await;
                        if let Some(state) = states.get(name.as_str()) {
                            state.status == ServerHealthStatus::Unhealthy
                                && state.status != ServerHealthStatus::NeedsAuth
                                && state.status != ServerHealthStatus::PermanentlyFailed
                                && state.restart_attempts < MAX_RESTART_ATTEMPTS
                        } else {
                            false
                        }
                    };

                    if !should_restart {
                        continue;
                    }

                    let backoff_secs;
                    {
                        let mut states = health_states.write().await;
                        let state = states.entry(name.clone()).or_default();
                        backoff_secs = state.next_backoff_secs();
                        state.restart_attempts += 1;
                        info!(
                            server = %name,
                            attempt = state.restart_attempts,
                            backoff_secs,
                            "Attempting background restart"
                        );
                    }

                    tokio::time::sleep(std::time::Duration::from_secs(backoff_secs)).await;

                    // Try to reconnect using the stored config.
                    let restart_ok = {
                        let cfg = config.read().await;
                        if let Some(mcp_config) = cfg.as_ref() {
                            if let Some(server_config) = mcp_config.mcp_servers.get(name.as_str()) {
                                let prepared = prepare_server_config(server_config);
                                match transport::create_transport(&prepared) {
                                    Ok(mut t) => {
                                        if t.connect().await.is_ok() {
                                            let tools = {
                                                let cache = tool_cache.read().await;
                                                cache.get(name.as_str()).map(|c| c.tools.clone())
                                            }
                                            .unwrap_or_default();

                                            let mut conns = connections.write().await;
                                            conns.insert(
                                                name.clone(),
                                                ServerConnection {
                                                    transport: t,
                                                    tools,
                                                    config: prepared,
                                                },
                                            );
                                            true
                                        } else {
                                            false
                                        }
                                    }
                                    Err(_) => false,
                                }
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    };

                    let mut states = health_states.write().await;
                    if let Some(state) = states.get_mut(name.as_str()) {
                        if restart_ok {
                            state.status = ServerHealthStatus::Healthy;
                            state.consecutive_failures = 0;
                            state.tools_removed = false;
                            info!(server = %name, "Background restart succeeded");
                        } else if state.restart_attempts >= MAX_RESTART_ATTEMPTS {
                            state.status = ServerHealthStatus::PermanentlyFailed;
                            error!(
                                server = %name,
                                "Server permanently failed after {} restart attempts",
                                MAX_RESTART_ATTEMPTS
                            );
                        } else {
                            // Record crash for rapid-failure detection
                            state.crash_timestamps.push(chrono::Utc::now());
                            let cutoff = chrono::Utc::now()
                                - chrono::Duration::seconds(CRASH_WINDOW_SECS as i64);
                            state.crash_timestamps.retain(|&t| t > cutoff);
                            if state.crash_timestamps.len() >= CRASH_THRESHOLD {
                                state.status = ServerHealthStatus::PermanentlyFailed;
                                error!(
                                    server = %name,
                                    "Server permanently failed after {} crashes within {}s",
                                    CRASH_THRESHOLD,
                                    CRASH_WINDOW_SECS,
                                );
                            } else {
                                warn!(server = %name, "Background restart failed");
                            }
                        }
                    }
                }
            }
        });

        let mut handle_guard = self.health_check_handle.write().await;
        // Abort any existing task.
        if let Some(old) = handle_guard.take() {
            old.abort();
        }
        *handle_guard = Some(handle);
        info!(interval_secs = self.health_check_interval_secs, "Started MCP health monitoring");
    }

    /// Stop background health monitoring.
    pub async fn stop_health_monitoring(&self) {
        let mut handle_guard = self.health_check_handle.write().await;
        if let Some(handle) = handle_guard.take() {
            handle.abort();
            debug!("Stopped MCP health monitoring");
        }
    }
}

#[cfg(test)]
mod tests;
