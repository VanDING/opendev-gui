use serde::{Serialize, Deserialize};

/// All v1 RPC methods. Names use `<domain>/<verb>` convention.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum Method {
    // ── Session (5) ──
    /// List past sessions
    #[serde(rename = "session/list")]
    SessionList,
    /// Create new session
    #[serde(rename = "session/start")]
    SessionStart,
    /// Resume existing session
    #[serde(rename = "session/resume")]
    SessionResume,
    /// Delete session
    #[serde(rename = "session/delete")]
    SessionDelete,
    /// List turns in a session
    #[serde(rename = "session/turns")]
    SessionTurns,

    // ── Turn (3) ──
    /// Send user input, kicks off agent loop
    #[serde(rename = "turn/start")]
    TurnStart,
    /// Cancel running turn
    #[serde(rename = "turn/interrupt")]
    TurnInterrupt,
    /// Inject mid-turn steering
    #[serde(rename = "turn/steer")]
    TurnSteer,

    // ── Tool (3) ──
    /// Get available tool schemas
    #[serde(rename = "tool/list")]
    ToolList,
    /// Activate deferred tools
    #[serde(rename = "tool/search")]
    ToolSearch,
    /// Respond to approval request
    #[serde(rename = "tool/approve")]
    ToolApprove,

    // ── Approval (2) ──
    /// List pending approvals
    #[serde(rename = "approval/list")]
    ApprovalList,
    /// Respond to an approval request
    #[serde(rename = "approval/respond")]
    ApprovalRespond,

    // ── MCP (7) ──
    /// List configured MCP servers
    #[serde(rename = "mcp/server/list")]
    McpServerList,
    /// Get one server's config
    #[serde(rename = "mcp/server/get")]
    McpServerGet,
    /// Add new MCP server
    #[serde(rename = "mcp/server/create")]
    McpServerCreate,
    /// Update MCP server config
    #[serde(rename = "mcp/server/update")]
    McpServerUpdate,
    /// Remove MCP server
    #[serde(rename = "mcp/server/delete")]
    McpServerDelete,
    /// Start MCP server connection
    #[serde(rename = "mcp/server/connect")]
    McpServerConnect,
    /// Stop MCP server connection
    #[serde(rename = "mcp/server/disconnect")]
    McpServerDisconnect,

    // ── Skill (2) ──
    /// List available skills
    #[serde(rename = "skill/list")]
    SkillList,
    /// Toggle skill pin
    #[serde(rename = "skill/pin")]
    SkillPin,

    // ── Config (5) ──
    /// Get current configuration
    #[serde(rename = "config/get")]
    ConfigGet,
    /// Update configuration
    #[serde(rename = "config/update")]
    ConfigUpdate,
    /// Set operation mode (normal/plan)
    #[serde(rename = "config/mode/set")]
    ConfigModeSet,
    /// Set autonomy level
    #[serde(rename = "config/autonomy/set")]
    ConfigAutonomySet,
    /// Verify model/provider connection
    #[serde(rename = "config/model/verify")]
    ConfigModelVerify,

    // ── File System (3) ──
    /// Browse directory
    #[serde(rename = "fs/browse")]
    FsBrowse,
    /// Verify path exists/accessible
    #[serde(rename = "fs/verify-path")]
    FsVerifyPath,
    /// List workspace files
    #[serde(rename = "fs/list-workspace")]
    FsListWorkspace,

    // ── Workspace (2) ──
    /// List workspaces
    #[serde(rename = "workspace/list")]
    WorkspaceList,
    /// Get workspace details
    #[serde(rename = "workspace/get")]
    WorkspaceGet,
}

impl Method {
    /// Returns the wire string for this method.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SessionList => "session/list",
            Self::SessionStart => "session/start",
            Self::SessionResume => "session/resume",
            Self::SessionDelete => "session/delete",
            Self::SessionTurns => "session/turns",
            Self::TurnStart => "turn/start",
            Self::TurnInterrupt => "turn/interrupt",
            Self::TurnSteer => "turn/steer",
            Self::ToolList => "tool/list",
            Self::ToolSearch => "tool/search",
            Self::ToolApprove => "tool/approve",
            Self::ApprovalList => "approval/list",
            Self::ApprovalRespond => "approval/respond",
            Self::McpServerList => "mcp/server/list",
            Self::McpServerGet => "mcp/server/get",
            Self::McpServerCreate => "mcp/server/create",
            Self::McpServerUpdate => "mcp/server/update",
            Self::McpServerDelete => "mcp/server/delete",
            Self::McpServerConnect => "mcp/server/connect",
            Self::McpServerDisconnect => "mcp/server/disconnect",
            Self::SkillList => "skill/list",
            Self::SkillPin => "skill/pin",
            Self::ConfigGet => "config/get",
            Self::ConfigUpdate => "config/update",
            Self::ConfigModeSet => "config/mode/set",
            Self::ConfigAutonomySet => "config/autonomy/set",
            Self::ConfigModelVerify => "config/model/verify",
            Self::FsBrowse => "fs/browse",
            Self::FsVerifyPath => "fs/verify-path",
            Self::FsListWorkspace => "fs/list-workspace",
            Self::WorkspaceList => "workspace/list",
            Self::WorkspaceGet => "workspace/get",
        }
    }
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
