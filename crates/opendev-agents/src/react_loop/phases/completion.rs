//! TurnResult::Complete handling: truncation, todo nudges, completion nudge,
//! background task blocking, and TaskCompleted hook chain.
//!
//! # TaskCompleted Hook Chain
//!
//! When a subagent or background task finishes, the `TaskCompleted` hook chain
//! fires to ensure no orphaned tasks remain. Each hook in the chain inspects
//! the current state and may inject nudges or directives:
//!
//! 1. Check if all spawned background tasks have completed (by comparing
//!    bg_tasks_spawned vs get_background_result messages in history).
//! 2. Check if bash background processes have been collected.
//! 3. Check if MCP monitor tasks are still polling.
//! 4. Verify no orphaned subagent sessions remain.

use std::sync::Mutex;

use serde_json::Value;
use tracing::{debug, info, warn};

use crate::prompts::reminders::{append_directive, append_nudge, get_reminder};
use crate::traits::{AgentEventCallback, AgentResult, LlmResponse, TaskMonitor};
use opendev_runtime::{TodoManager, TodoStatus, play_finish_sound};

use super::super::ReactLoop;
use super::super::loop_state::LoopState;
use super::super::types::{IterationMetrics, LoopAction};

/// Count completed background tasks from message history.
///
/// Looks for get_background_result tool results (from subagent/bg completions)
/// and for bash background results with "background_id" metadata tracking.
fn count_completed_background_tasks(messages: &[Value]) -> usize {
    messages
        .iter()
        .filter(|m| {
            let name = m.get("name").and_then(|n| n.as_str()).unwrap_or("");
            if name == "get_background_result" {
                return true;
            }
            // Bash background results: when a background command finishes
            if name == "Bash" {
                if let Some(content) = m.get("content").and_then(|c| c.as_str()) {
                    if content.contains("Background process") && content.contains("finished")
                        || content.contains("background-done")
                        || content.contains("[background completed]")
                    {
                        return true;
                    }
                }
            }
            false
        })
        .count()
}

/// Hook: block completion when spawned background tasks haven't reported back.
fn background_tasks_hook(
    state: &mut LoopState,
    messages: &mut Vec<Value>,
) -> Option<LoopAction> {
    if state.bg_tasks_spawned == 0 {
        return None;
    }

    let bg_completed_msgs = count_completed_background_tasks(messages);
    let pending = state.bg_tasks_spawned.saturating_sub(bg_completed_msgs);

    if pending == 0 {
        debug!(
            spawned = state.bg_tasks_spawned,
            completed = bg_completed_msgs,
            "All background tasks have completed"
        );
        return None;
    }

    if state.bg_wait_nudge_count >= 10 {
        warn!(
            spawned = state.bg_tasks_spawned,
            completed = bg_completed_msgs,
            pending,
            nudge_count = state.bg_wait_nudge_count,
            "Background task nudge limit reached — allowing completion"
        );
        return None;
    }

    state.bg_wait_nudge_count += 1;
    info!(
        spawned = state.bg_tasks_spawned,
        completed = bg_completed_msgs,
        pending,
        nudge_count = state.bg_wait_nudge_count,
        "Blocking completion — background tasks still running"
    );

    let nudge = format!(
        "You have {pending} background task(s) still running. \
         Do NOT duplicate their work or call TeamDelete. \
         Do NOT call get_background_result — results arrive automatically. \
         Wait for the background completion notifications before finishing."
    );
    append_nudge(messages, &nudge);
    Some(LoopAction::Continue)
}

/// Hook: block completion when there are incomplete todos with work in progress.
fn incomplete_todos_hook(
    react_loop: &ReactLoop,
    state: &mut LoopState,
    todo_manager: Option<&Mutex<TodoManager>>,
    messages: &mut Vec<Value>,
) -> Option<LoopAction> {
    if let Some(mgr) = todo_manager
        && let Ok(mgr) = mgr.lock()
        && mgr.has_incomplete_todos()
        && mgr.has_work_in_progress()
        && state.todo_nudge_count < react_loop.config.max_todo_nudges
    {
        state.todo_nudge_count += 1;
        let count = mgr.total() - mgr.completed_count();
        let titles: Vec<_> = mgr
            .all()
            .iter()
            .filter(|t| t.status != TodoStatus::Completed)
            .take(3)
            .map(|t| format!("  - {}", t.title))
            .collect();
        let nudge = get_reminder(
            "incomplete_todos_nudge",
            &[("count", &count.to_string()), ("todo_list", &titles.join("\n"))],
        );
        append_nudge(messages, &nudge);
        return Some(LoopAction::Continue);
    }
    None
}

/// Hook: implicit completion nudge — verify original task before finishing.
fn implicit_completion_hook(
    react_loop: &ReactLoop,
    state: &mut LoopState,
    messages: &mut Vec<Value>,
) -> Option<LoopAction> {
    let has_used_tools = state.iteration > state.consecutive_no_tool_calls;
    if !state.completion_nudge_sent
        && has_used_tools
        && let Some(task) = react_loop.config.original_task.as_deref()
    {
        state.completion_nudge_sent = true;
        info!(
            iteration = state.iteration,
            "Completion nudge firing — pre-nudge check"
        );
        let nudge = get_reminder("implicit_completion_nudge", &[("original_task", task)]);
        append_nudge(messages, &nudge);
        return Some(LoopAction::Continue);
    }
    None
}

/// Handle the `TurnResult::Complete` branch of the react loop.
///
/// Returns `LoopAction::Continue` when a nudge was injected and the loop
/// should re-iterate, or `LoopAction::Return` with the final result.
#[allow(clippy::too_many_arguments)]
pub(in crate::react_loop) fn handle_completion<M>(
    react_loop: &ReactLoop,
    content: String,
    status: Option<String>,
    response: &LlmResponse,
    messages: &mut Vec<Value>,
    state: &mut LoopState,
    iter_metrics: IterationMetrics,
    task_monitor: Option<&M>,
    todo_manager: Option<&Mutex<TodoManager>>,
    event_callback: Option<&dyn AgentEventCallback>,
) -> LoopAction
where
    M: TaskMonitor + ?Sized,
{
    // Check for output truncation (finish_reason == "length")
    if response.finish_reason.as_deref() == Some("length") && state.consecutive_truncations < 3 {
        state.consecutive_truncations += 1;
        warn!(
            consecutive_truncations = state.consecutive_truncations,
            "Response truncated due to output token limit, continuing"
        );
        append_directive(messages, &get_reminder("truncation_continue_directive", &[]));
        react_loop.push_metrics(iter_metrics);
        return LoopAction::Continue;
    }
    state.consecutive_truncations = 0;

    // Run the TaskCompleted hook chain in priority order.
    // Each hook returns Some(LoopAction::Continue) to block completion
    // with a directive, or None to proceed to the next hook.
    let hooks: &[&dyn Fn(
        &ReactLoop,
        &mut LoopState,
        &mut Vec<Value>,
        Option<&Mutex<TodoManager>>,
    ) -> Option<LoopAction>] = &[
        &|_rl, s, m, _tm| background_tasks_hook(s, m),
        &|rl, s, m, tm| incomplete_todos_hook(rl, s, tm, m),
        &|rl, s, m, _tm| implicit_completion_hook(rl, s, m),
    ];

    for hook in hooks {
        if let Some(action) = hook(react_loop, state, messages, todo_manager) {
            react_loop.push_metrics(iter_metrics);
            return action;
        }
    }

    // Check for background request before accepting completion
    if task_monitor.is_some_and(|m| m.is_background_requested()) {
        info!(iteration = state.iteration, "Background requested at completion — yielding");
        react_loop.push_metrics(iter_metrics);
        return LoopAction::Return(Ok(AgentResult::backgrounded(messages.clone())));
    }

    react_loop.push_metrics(iter_metrics);

    // If content was suppressed during nudge verification, emit it now
    if state.completion_nudge_sent {
        info!(
            iteration = state.iteration,
            content_len = content.len(),
            content_preview = opendev_runtime::safe_truncate(&content, 120),
            "Post-nudge acceptance — emitting suppressed content"
        );
        if !content.is_empty()
            && let Some(cb) = event_callback
        {
            cb.on_agent_chunk(&content);
        }
    }

    // Play completion sound (respects 30s cooldown)
    play_finish_sound();
    let mut result = AgentResult::ok(content, messages.clone());
    result.completion_status = status;
    LoopAction::Return(Ok(result))
}
