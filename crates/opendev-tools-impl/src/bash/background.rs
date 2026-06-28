//! Background (fire-and-forget) command execution with startup output capture.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};

use opendev_tools_core::ToolResult;

use super::BashTool;
use super::helpers::{BackgroundProcess, command_failure_suffix, prepare_command};

impl BashTool {
    pub(super) async fn run_background(
        &self,
        command: &str,
        working_dir: &std::path::Path,
    ) -> ToolResult {
        let exec_command = prepare_command(command);

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
            Err(e) => return ToolResult::fail(format!("Failed to spawn background command: {e}")),
        };

        let pid = child.id().unwrap_or(0);
        let pgid = pid;

        let stdout_pipe = child.stdout.take();
        let stderr_pipe = child.stderr.take();

        // Capture initial startup output (up to 20s, with 3s idle timeout)
        let stdout_buf: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let stderr_buf: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let startup_activity = Arc::new(Mutex::new(Instant::now()));

        // Spawn stdout reader
        let stdout_reader_lines = stdout_buf.clone();
        let stdout_activity = startup_activity.clone();
        let stdout_reader = tokio::spawn(async move {
            if let Some(pipe) = stdout_pipe {
                let mut reader = BufReader::new(pipe).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    *stdout_activity.lock().await = Instant::now();
                    stdout_reader_lines.lock().await.push(line);
                }
            }
        });

        // Spawn stderr reader
        let stderr_reader_lines = stderr_buf.clone();
        let stderr_activity = startup_activity.clone();
        let stderr_reader = tokio::spawn(async move {
            if let Some(pipe) = stderr_pipe {
                let mut reader = BufReader::new(pipe).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    *stderr_activity.lock().await = Instant::now();
                    stderr_reader_lines.lock().await.push(line);
                }
            }
        });

        // Wait for startup output with idle timeout
        let startup_start = Instant::now();
        let max_startup = Duration::from_secs(20);
        let startup_idle = Duration::from_secs(3);

        loop {
            tokio::time::sleep(Duration::from_millis(200)).await;

            // Check if child already exited
            match child.try_wait() {
                Ok(Some(status)) => {
                    // Process finished during startup
                    let _ = tokio::time::timeout(Duration::from_secs(1), stdout_reader).await;
                    let _ = tokio::time::timeout(Duration::from_secs(1), stderr_reader).await;

                    let stdout_text = stdout_buf.lock().await.join("\n");
                    let stderr_text = stderr_buf.lock().await.join("\n");
                    let exit_code = status.code().unwrap_or(-1);

                    let mut combined = stdout_text;
                    if !stderr_text.is_empty() {
                        if !combined.is_empty() {
                            combined.push('\n');
                        }
                        combined.push_str(&format!("[stderr]\n{stderr_text}"));
                    }

                    let mut metadata = HashMap::new();
                    metadata.insert("exit_code".into(), serde_json::json!(exit_code));

                    if status.success() {
                        return ToolResult::ok_with_metadata(combined, metadata);
                    } else {
                        let suffix = command_failure_suffix(exit_code, &combined);
                        return ToolResult {
                            success: false,
                            output: Some(combined),
                            error: Some(format!("Command exited with code {exit_code}")),
                            metadata,
                            duration_ms: None,
                            llm_suffix: Some(suffix),
                        };
                    }
                }
                Ok(None) => {} // still running
                Err(_) => {}
            }

            // Check startup capture time limits
            if startup_start.elapsed() >= max_startup {
                break;
            }
            let idle_elapsed = startup_activity.lock().await.elapsed();
            // Give at least 1s before checking idle
            if startup_start.elapsed() > Duration::from_secs(1) && idle_elapsed >= startup_idle {
                break;
            }
        }

        // Process still running — store as background
        let bg_id = self.next_id().await;
        let stdout_captured = stdout_buf.lock().await.clone();
        let stderr_captured = stderr_buf.lock().await.clone();
        let startup_output = stdout_captured.join("\n");

        let bp = BackgroundProcess {
            id: bg_id,
            command: command.to_string(),
            pid,
            pgid,
            started_at: Instant::now(),
            stdout_lines: stdout_captured,
            stderr_lines: stderr_captured,
            child,
        };
        self.background.lock().await.insert(bg_id, bp);

        // Keep reader tasks alive — they'll stop when the child's pipes close.
        tokio::spawn(async move {
            let _ = stdout_reader.await;
        });
        tokio::spawn(async move {
            let _ = stderr_reader.await;
        });

        let mut metadata = HashMap::new();
        metadata.insert("background_id".into(), serde_json::json!(bg_id));
        metadata.insert("pid".into(), serde_json::json!(pid));

        let msg = if startup_output.is_empty() {
            format!("Background process started (id={bg_id}, pid={pid})")
        } else {
            format!(
                "Background process started (id={bg_id}, pid={pid})\n\
                 Startup output:\n{startup_output}"
            )
        };

        ToolResult::ok_with_metadata(msg, metadata)
    }
}
