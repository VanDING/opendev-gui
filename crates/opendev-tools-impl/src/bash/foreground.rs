//! Foreground (synchronous) command execution with streaming output and dual timeout.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};

use opendev_tools_core::ToolResult;

use super::BashTool;
use super::helpers::{
    IDLE_TIMEOUT, MAX_OUTPUT_CHARS, MAX_TIMEOUT, command_failure_suffix, kill_process_group,
    prepare_command, truncate_output,
};
use super::patterns::MAX_FOREGROUND_MS;

impl BashTool {
    pub(super) async fn run_foreground(
        &self,
        command: &str,
        working_dir: &std::path::Path,
        timeout_secs: u64,
        timeout_config: Option<&opendev_tools_core::ToolTimeoutConfig>,
        cancel_token: Option<&tokio_util::sync::CancellationToken>,
    ) -> ToolResult {
        let exec_command = prepare_command(command);

        // Use context-provided timeout config or fall back to defaults
        let base_idle = timeout_config
            .map(|c| Duration::from_secs(c.idle_timeout_secs))
            .unwrap_or(IDLE_TIMEOUT);
        let base_max =
            timeout_config.map(|c| Duration::from_secs(c.max_timeout_secs)).unwrap_or(MAX_TIMEOUT);

        // Caller timeout caps both idle and absolute timeouts
        let idle_timeout = base_idle.min(Duration::from_secs(timeout_secs));
        let max_timeout = base_max.min(Duration::from_secs(timeout_secs));

        // Spawn with new process group.
        // Use filtered environment to prevent API keys/tokens from leaking
        // into child processes. Delegates to opendev-exec for centralized
        // secret stripping (handles env_clear + filtered envs + PYTHONUNBUFFERED).
        //
        // NOTE: stdout/stderr and pre_exec are configured AFTER the sandbox
        // backend apply, because the backend may replace the inner
        // std::process::Command (e.g., Seatbelt wrapping with sandbox-exec).
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(&exec_command).current_dir(working_dir);

        // Apply env filter (from opendev-exec) — env_clear + filtered envs + PYTHONUNBUFFERED
        opendev_exec::env_filter::apply(cmd.as_std_mut());

        // ── Apply sandbox backend (fail-closed) ──
        //
        // Must run before stdout/stderr + pre_exec config, since some
        // backends (e.g., Seatbelt) replace the inner std::process::Command.
        let backend = opendev_exec::backend::detect_backend()
            .expect("at least NoneBackend is always available");
        let exec_request = opendev_exec::policy::ExecRequest {
            tool: opendev_exec::policy::ToolKind::Bash,
            command: command.to_string(),
            argv: vec!["sh".into(), "-c".into(), exec_command.clone()],
            cwd: working_dir.to_path_buf(),
            env: std::collections::HashMap::new(),
            requested_paths: vec![],
            requested_net: None,
            capabilities: Default::default(),
            allowed_domains: vec![],
            denied_domains: vec![],
        };
        if let Err(e) = backend.apply(cmd.as_std_mut(), &exec_request) {
            tracing::error!(error = %e, backend = backend.name(), "sandbox apply failed; refusing to spawn");
            return ToolResult::fail(format!(
                "Sandbox backend '{}' failed to apply: {}. Command not executed (fail-closed).",
                backend.name(),
                e
            ));
        }

        // ── Configure stdout/stderr + process group (after backend apply) ──
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        // SAFETY: `pre_exec` runs in the child process after `fork()` and
        // before `exec()`. At this point only a single thread exists, so
        // there is no risk of data races. `setpgid(0, 0)` creates a new
        // process group with the child as leader — this is standard
        // practice for clean process-group termination and cannot fail
        // in ways that corrupt parent state.
        // Create new process group on Unix for clean kill
        #[cfg(unix)]
        unsafe {
            cmd.pre_exec(|| {
                libc::setpgid(0, 0);
                Ok(())
            });
        }

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => return ToolResult::fail(format!("Failed to spawn command: {e}")),
        };

        let pid = child.id().unwrap_or(0);
        let pgid = pid; // process group leader = child PID

        let stdout_pipe = child.stdout.take();
        let stderr_pipe = child.stderr.take();

        // Streaming readers
        let stdout_lines: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let stderr_lines: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let last_activity = Arc::new(Mutex::new(Instant::now()));

        // Spawn stdout reader
        let stdout_handle = {
            let lines = stdout_lines.clone();
            let activity = last_activity.clone();
            tokio::spawn(async move {
                if let Some(pipe) = stdout_pipe {
                    let mut reader = BufReader::new(pipe).lines();
                    while let Ok(Some(line)) = reader.next_line().await {
                        *activity.lock().await = Instant::now();
                        lines.lock().await.push(line);
                    }
                }
            })
        };

        // Spawn stderr reader
        let stderr_handle = {
            let lines = stderr_lines.clone();
            let activity = last_activity.clone();
            tokio::spawn(async move {
                if let Some(pipe) = stderr_pipe {
                    let mut reader = BufReader::new(pipe).lines();
                    while let Ok(Some(line)) = reader.next_line().await {
                        *activity.lock().await = Instant::now();
                        lines.lock().await.push(line);
                    }
                }
            })
        };

        let start = Instant::now();

        // Poll child with dual timeout
        let exit_status = loop {
            // Check if child exited
            match child.try_wait() {
                Ok(Some(status)) => break Ok(status),
                Ok(None) => {}
                Err(e) => break Err(format!("Failed to wait on child: {e}")),
            }

            // Check auto-background promotion: if this command is recognized
            // as a long-running command and has exceeded the foreground budget,
            // log a warning suggesting the LLM re-run in background.
            if start.elapsed() >= Duration::from_millis(MAX_FOREGROUND_MS)
                && super::patterns::is_auto_background_command(command)
            {
                tracing::info!(
                    duration_ms = start.elapsed().as_millis(),
                    "Command exceeded foreground budget — consider backgrounding"
                );
            }

            // Check absolute timeout
            if start.elapsed() >= max_timeout {
                kill_process_group(pgid);
                let _ = child.wait().await;
                break Err(format!(
                    "Command timed out — exceeded maximum runtime of {}s",
                    max_timeout.as_secs()
                ));
            }

            // Check idle timeout
            let idle = {
                let la = last_activity.lock().await;
                la.elapsed()
            };
            if idle >= idle_timeout {
                kill_process_group(pgid);
                let _ = child.wait().await;
                break Err(format!(
                    "Command timed out after {}s of no output (idle timeout)",
                    idle.as_secs()
                ));
            }

            // Check cancel token for user interrupt
            if let Some(token) = cancel_token
                && token.is_cancelled()
            {
                kill_process_group(pgid);
                let _ = child.wait().await;
                break Err("Interrupted by user".to_string());
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        };

        // Wait for readers to finish draining
        let _ = tokio::time::timeout(Duration::from_secs(2), stdout_handle).await;
        let _ = tokio::time::timeout(Duration::from_secs(2), stderr_handle).await;

        let stdout_text = stdout_lines.lock().await.join("\n");
        let stderr_text = stderr_lines.lock().await.join("\n");

        match exit_status {
            Ok(status) => {
                let exit_code = status.code().unwrap_or(-1);
                let success = status.success();

                let mut combined = stdout_text;
                if !stderr_text.is_empty() {
                    if !combined.is_empty() {
                        combined.push('\n');
                    }
                    combined.push_str(&format!("[stderr]\n{stderr_text}"));
                }

                // Truncate for display
                let display_output = truncate_output(&combined, false);
                // Truncate for LLM metadata
                let llm_output = truncate_output(&combined, true);

                // If output was truncated, save full output to overflow file.
                let overflow_result = if combined.len() > MAX_OUTPUT_CHARS {
                    Some(crate::truncation::truncate_output(
                        &combined,
                        None,
                        None,
                        crate::truncation::TruncateDirection::Head,
                    ))
                } else {
                    None
                };

                let mut metadata = HashMap::new();
                metadata.insert("exit_code".into(), serde_json::json!(exit_code));
                metadata.insert("llm_output".into(), serde_json::json!(llm_output));
                if let Some(ref ovf) = overflow_result
                    && let Some(ref path) = ovf.output_path
                {
                    metadata.insert(
                        "overflow_path".into(),
                        serde_json::json!(path.display().to_string()),
                    );
                }

                if success {
                    // If overflowed, append the hint to the display output.
                    let final_output = if let Some(ref ovf) = overflow_result {
                        if let Some(ref path) = ovf.output_path {
                            format!(
                                "{display_output}\n\n[Full output saved to: {}. Use Read with offset/limit or Grep to search it.]",
                                path.display()
                            )
                        } else {
                            display_output
                        }
                    } else {
                        display_output
                    };
                    let mut result = ToolResult::ok_with_metadata(final_output, metadata);
                    if overflow_result.is_some() {
                        result = result.with_llm_suffix(
                            "use read_file with offset to see the full truncated output"
                        );
                    }
                    result
                } else {
                    let suffix = command_failure_suffix(exit_code, &combined);
                    ToolResult {
                        success: false,
                        output: Some(display_output),
                        error: Some(format!("Command exited with code {exit_code}")),
                        metadata,
                        duration_ms: None,
                        llm_suffix: Some(suffix),
                    }
                }
            }
            Err(timeout_msg) => {
                let mut combined = stdout_text;
                if !stderr_text.is_empty() {
                    if !combined.is_empty() {
                        combined.push('\n');
                    }
                    combined.push_str(&format!("[stderr]\n{stderr_text}"));
                }
                let display_output = truncate_output(&combined, false);

                let mut metadata = HashMap::new();
                metadata.insert("exit_code".into(), serde_json::json!(-1));

                ToolResult {
                    success: false,
                    output: if display_output.is_empty() { None } else { Some(display_output) },
                    error: Some(timeout_msg),
                    metadata,
                    duration_ms: None,
                    llm_suffix: Some(
                        "The command timed out. Consider breaking it into smaller steps, \
                        adding a timeout flag, or checking if the process is hanging."
                            .to_string(),
                    ),
                }
            }
        }
    }
}
