//! Metrics helper functions (lightweight wrappers).
//!
//! V1 metrics:
//! - opendev.llm.calls.total (counter, labels: provider, model, status)
//! - opendev.llm.tokens.input (counter, labels: provider)
//! - opendev.llm.tokens.output (counter, labels: provider)
//! - opendev.llm.duration.seconds (histogram, labels: provider)
//! - opendev.tools.executed.total (counter, labels: tool, result)
//! - opendev.tools.duration.seconds (histogram, labels: tool)
//! - opendev.sessions.active (gauge)
//! - opendev.sessions.created.total (counter)
//! - opendev.cost.usd.total (counter, labels: provider)
//! - opendev.errors.total (counter, labels: code, surface)
//! - opendev.sandbox.executions.total (counter, labels: tool, decision)
//! - opendev.secrets.lookups.total (counter, labels: namespace, result)
//! - opendev.approvals.requested.total (counter, labels: tool)
//! - opendev.approvals.granted.total (counter, labels: tool, decision)
//! - opendev.frontend.errors.total (counter, labels: component)

/// Record an LLM call metric.
pub fn record_llm_call(provider: &str, model: &str, status: &str) {
    tracing::debug!(
        target: "metrics",
        metric = "opendev.llm.calls.total",
        provider = %provider,
        model = %model,
        status = %status,
        "LLM call recorded"
    );
}

/// Record LLM token usage.
pub fn record_llm_tokens(provider: &str, input: u64, output: u64) {
    tracing::debug!(
        target: "metrics",
        metric = "opendev.llm.tokens",
        provider = %provider,
        input = input,
        output = output,
        "LLM token usage recorded"
    );
}

/// Record a tool execution.
pub fn record_tool_call(tool: &str, result: &str) {
    tracing::debug!(
        target: "metrics",
        metric = "opendev.tools.executed.total",
        tool = %tool,
        result = %result,
        "Tool call recorded"
    );
}

/// Record session created.
pub fn record_session_created() {
    tracing::debug!(
        target: "metrics",
        metric = "opendev.sessions.created.total",
        "Session created"
    );
}

/// Record session active count.
pub fn record_sessions_active(count: u64) {
    tracing::debug!(
        target: "metrics",
        metric = "opendev.sessions.active",
        count = count,
        "Sessions active"
    );
}

/// Record an error event.
pub fn record_error(code: &str, surface: &str) {
    tracing::debug!(
        target: "metrics",
        metric = "opendev.errors.total",
        code = %code,
        surface = %surface,
        "Error recorded"
    );
}

/// Record a sandbox execution.
pub fn record_sandbox_execution(tool: &str, decision: &str) {
    tracing::debug!(
        target: "metrics",
        metric = "opendev.sandbox.executions.total",
        tool = %tool,
        decision = %decision,
        "Sandbox execution recorded"
    );
}

/// Record a secret lookup.
pub fn record_secret_lookup(namespace: &str, result: &str) {
    tracing::debug!(
        target: "metrics",
        metric = "opendev.secrets.lookups.total",
        namespace = %namespace,
        result = %result,
        "Secret lookup recorded"
    );
}

/// Record an approval request.
pub fn record_approval_requested(tool: &str) {
    tracing::debug!(
        target: "metrics",
        metric = "opendev.approvals.requested.total",
        tool = %tool,
        "Approval requested"
    );
}

/// Record an approval grant.
pub fn record_approval_granted(tool: &str, decision: &str) {
    tracing::debug!(
        target: "metrics",
        metric = "opendev.approvals.granted.total",
        tool = %tool,
        decision = %decision,
        "Approval granted"
    );
}

/// Record an LLM duration.
pub fn record_llm_duration(provider: &str, duration_secs: f64) {
    tracing::debug!(
        target: "metrics",
        metric = "opendev.llm.duration.seconds",
        provider = %provider,
        duration = duration_secs,
        "LLM duration recorded"
    );
}

/// Record a tool duration.
pub fn record_tool_duration(tool: &str, duration_secs: f64) {
    tracing::debug!(
        target: "metrics",
        metric = "opendev.tools.duration.seconds",
        tool = %tool,
        duration = duration_secs,
        "Tool duration recorded"
    );
}

/// Record cost.
pub fn record_cost(provider: &str, usd: f64) {
    tracing::debug!(
        target: "metrics",
        metric = "opendev.cost.usd.total",
        provider = %provider,
        usd = usd,
        "Cost recorded"
    );
}

/// Record frontend error.
pub fn record_frontend_error(component: &str) {
    tracing::debug!(
        target: "metrics",
        metric = "opendev.frontend.errors.total",
        component = %component,
        "Frontend error recorded"
    );
}
