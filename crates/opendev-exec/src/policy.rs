use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use url::Url;

/// Identifies which tool is requesting execution.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ToolKind {
    Bash,
    Custom,
    Hook,
    Mcp,
    Git,
    Lsp,
    Formatter,
    Search,
    Browser,
    Screenshot,
    Schedule,
    Other(String),
}

impl ToolKind {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Bash => "bash",
            Self::Custom => "custom",
            Self::Hook => "hook",
            Self::Mcp => "mcp",
            Self::Git => "git",
            Self::Lsp => "lsp",
            Self::Formatter => "formatter",
            Self::Search => "search",
            Self::Browser => "browser",
            Self::Screenshot => "screenshot",
            Self::Schedule => "schedule",
            Self::Other(s) => s.as_str(),
        }
    }
}

/// Capabilities the command requests.
#[derive(Debug, Clone)]
pub struct RequiredCapabilities {
    pub read: Vec<PathBuf>,
    pub write: Vec<PathBuf>,
    pub network: bool,
    pub subprocess: bool,
    pub max_memory_mb: Option<u64>,
    pub max_cpu_secs: Option<u64>,
    pub max_open_fds: Option<u64>,
    pub max_file_size_mb: Option<u64>,
}

impl Default for RequiredCapabilities {
    fn default() -> Self {
        Self {
            read: vec![],
            write: vec![],
            network: false,
            subprocess: false,
            max_memory_mb: None,
            max_cpu_secs: None,
            max_open_fds: None,
            max_file_size_mb: None,
        }
    }
}

/// A request to execute a command.
#[derive(Debug, Clone)]
pub struct ExecRequest {
    pub tool: ToolKind,
    pub command: String,
    pub argv: Vec<String>,
    pub cwd: PathBuf,
    pub env: HashMap<String, String>,
    pub requested_paths: Vec<PathBuf>,
    pub requested_net: Option<Url>,
    pub capabilities: RequiredCapabilities,
    /// Domain names the command is allowed to access (network allowlist).
    /// Empty = no domain-level restrictions beyond the network bool.
    pub allowed_domains: Vec<String>,
    /// Domain names the command is explicitly denied from accessing.
    pub denied_domains: Vec<String>,
}

/// The policy decision for an exec request.
#[derive(Debug, Clone)]
pub enum Decision {
    /// Allow execution directly.
    Allow,
    /// Allow but record the verdict.
    AllowWith(PolicyVerdict),
    /// Deny execution. MUST NOT spawn child.
    Deny { reason: String },
    /// Require user approval before execution.
    Prompt { reason: String, ttl: Duration },
}

/// Policy verdict metadata (for AllowWith).
#[derive(Debug, Clone)]
pub struct PolicyVerdict {
    pub policy_name: String,
    pub reason: String,
}

/// Errors from policy evaluation.
#[derive(Debug, thiserror::Error)]
pub enum PolicyError {
    #[error("Policy evaluation error: {0}")]
    Evaluation(String),
    #[error("Backend error: {0}")]
    Backend(String),
}

/// The core ExecPolicy trait.
pub trait ExecPolicy: Send + Sync {
    /// Evaluate a command description, returning a Decision.
    fn evaluate(&self, request: &ExecRequest) -> Result<Decision, PolicyError>;

    /// Name of this policy (for tracing/debugging).
    fn name(&self) -> &'static str;

    /// Whether this policy provides OS-level isolation (vs. env_filter only).
    fn has_os_isolation(&self) -> bool;
}

// ── Built-in policies ──

/// Strict policy — deny all by default, only hardcoded read commands pass.
pub struct StrictPolicy;

impl ExecPolicy for StrictPolicy {
    fn evaluate(&self, request: &ExecRequest) -> Result<Decision, PolicyError> {
        // Allow only explicit read-only commands
        let cmd = &request.command;
        let base = cmd.split_whitespace().next().unwrap_or("");
        match base {
            "ls" | "cat" | "head" | "tail" | "grep" | "rg" | "find" | "wc" | "sort" | "uniq"
            | "diff" | "echo" | "pwd" | "which" | "env" | "true" | "false" | "git" => {
                Ok(Decision::Allow)
            }
            _ => Ok(Decision::Deny {
                reason: format!("Command '{}' is not in strict allowlist", base),
            }),
        }
    }

    fn name(&self) -> &'static str {
        "strict"
    }
    fn has_os_isolation(&self) -> bool {
        false
    }
}

/// Workspace write policy — can write cwd + tmp, read ~/Library/Application Support, network opt-in.
pub struct WorkspaceWritePolicy {
    pub cwd: PathBuf,
}

impl ExecPolicy for WorkspaceWritePolicy {
    fn evaluate(&self, request: &ExecRequest) -> Result<Decision, PolicyError> {
        // Default: allow commands targeting the workspace
        if request.requested_paths.iter().all(|p| p.starts_with(&self.cwd)) {
            return Ok(Decision::Allow);
        }
        // External paths need prompting
        Ok(Decision::Prompt {
            reason: "Command accesses paths outside workspace".into(),
            ttl: Duration::from_secs(300),
        })
    }

    fn name(&self) -> &'static str {
        "workspace_write"
    }
    fn has_os_isolation(&self) -> bool {
        false
    }
}

/// Read-only policy — completely read-only, no network.
pub struct ReadOnlyPolicy;

impl ExecPolicy for ReadOnlyPolicy {
    fn evaluate(&self, request: &ExecRequest) -> Result<Decision, PolicyError> {
        if request.capabilities.network {
            return Ok(Decision::Deny {
                reason: "Network access denied by read-only policy".into(),
            });
        }
        if !request.capabilities.write.is_empty() {
            return Ok(Decision::Deny { reason: "Write access denied by read-only policy".into() });
        }
        Ok(Decision::Allow)
    }

    fn name(&self) -> &'static str {
        "read_only"
    }
    fn has_os_isolation(&self) -> bool {
        false
    }
}

/// Danger full-access policy — explicit opt-in only.
pub struct DangerFullAccessPolicy;

impl ExecPolicy for DangerFullAccessPolicy {
    fn evaluate(&self, _request: &ExecRequest) -> Result<Decision, PolicyError> {
        Ok(Decision::Allow)
    }

    fn name(&self) -> &'static str {
        "danger_full_access"
    }
    fn has_os_isolation(&self) -> bool {
        false
    }
}

/// BashTool policy — uses dangerous pattern detection + safe command allowlist.
pub struct BashToolPolicy {
    pub safe_commands: Vec<String>,
    pub working_dir: PathBuf,
}

impl BashToolPolicy {
    pub fn new(working_dir: PathBuf) -> Self {
        // 28 safe commands from existing codebase
        let safe_commands = vec![
            "cargo".into(),
            "rustc".into(),
            "npm".into(),
            "node".into(),
            "python".into(),
            "python3".into(),
            "git".into(),
            "ls".into(),
            "cat".into(),
            "head".into(),
            "tail".into(),
            "grep".into(),
            "find".into(),
            "wc".into(),
            "sort".into(),
            "uniq".into(),
            "diff".into(),
            "echo".into(),
            "pwd".into(),
            "which".into(),
            "env".into(),
            "test".into(),
            "true".into(),
            "false".into(),
            "mkdir".into(),
            "cp".into(),
            "mv".into(),
            "touch".into(),
        ];
        Self { safe_commands, working_dir }
    }

    pub fn is_safe_command(&self, command: &str) -> bool {
        let base = command.split_whitespace().next().unwrap_or("");
        self.safe_commands.iter().any(|c| c == base)
    }
}

impl ExecPolicy for BashToolPolicy {
    fn evaluate(&self, request: &ExecRequest) -> Result<Decision, PolicyError> {
        // 1. Check dangerous patterns first
        if crate::patterns::is_dangerous(request.command.as_str()) {
            return Ok(Decision::Deny { reason: "Command matches dangerous pattern".into() });
        }

        // 2. Allow all non-dangerous commands.
        //    OS-level isolation (via SandboxBackend) handles actual confinement.
        //    The safe_commands list is available for future approval-UI integration.
        Ok(Decision::Allow)
    }

    fn name(&self) -> &'static str {
        "bash_tool"
    }
    fn has_os_isolation(&self) -> bool {
        true
    }
}
