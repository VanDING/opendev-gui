//! Files DTOs.

use serde::{Deserialize, Serialize};

/// Browse directory request.
#[derive(Debug, Deserialize)]
pub struct BrowseDirectoryRequest {
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub show_hidden: bool,
}

/// Verify path request.
#[derive(Debug, Deserialize)]
pub struct VerifyPathRequest {
    pub path: String,
}

/// List files query.
#[derive(Debug, Deserialize)]
pub struct ListFilesQuery {
    #[serde(default)]
    pub query: Option<String>,
}

/// Browse directory response.
#[derive(Debug, Serialize)]
pub struct BrowseDirectoryResponse {
    pub current_path: String,
    pub parent_path: Option<String>,
    pub directories: Vec<DirEntryData>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DirEntryData {
    pub name: String,
    pub path: String,
}
