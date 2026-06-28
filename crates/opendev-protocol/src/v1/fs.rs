use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// ── fs/browse ──
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FsBrowseParams {
    pub path: String,
    pub show_hidden: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FsBrowseResponse {
    pub path: String,
    pub entries: Vec<DirectoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DirectoryEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: i64,
}

/// ── fs/verify-path ──
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FsVerifyPathParams {
    pub path: String,
    pub mode: Option<String>, // "file" | "directory" | "any"
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FsVerifyPathResponse {
    pub exists: bool,
    pub is_file: bool,
    pub is_dir: bool,
    pub canonical_path: Option<String>,
}

/// ── fs/list-workspace ──
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FsListWorkspaceParams {
    pub query: Option<String>,
    pub max_results: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FsListWorkspaceResponse {
    pub files: Vec<String>,
}
