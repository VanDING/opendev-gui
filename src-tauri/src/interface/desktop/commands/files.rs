//! Files commands — DTO mapping only.

use tauri::State;
use crate::application::AppServices;
use crate::interface::desktop::contract::files::*;

/// Browse directory contents.
#[tauri::command]
pub async fn browse_directory(
    services: State<'_, AppServices>,
    req: BrowseDirectoryRequest,
) -> Result<BrowseDirectoryResponse, String> {
    let result = services.file.browse_directory(&req.path, req.show_hidden);
    Ok(BrowseDirectoryResponse {
        current_path: result.current_path,
        parent_path: result.parent_path,
        directories: result.directories.into_iter().map(|d| DirEntryData {
            name: d.name,
            path: d.path,
        }).collect(),
        error: result.error,
    })
}

/// Verify a path exists within the workspace.
#[tauri::command]
pub async fn verify_path(
    services: State<'_, AppServices>,
    req: VerifyPathRequest,
) -> Result<serde_json::Value, String> {
    let result = services.file.verify_path(&req.path);
    Ok(serde_json::json!({
        "exists": result.exists,
        "is_directory": result.is_directory,
        "path": result.path,
        "error": result.error,
    }))
}

/// List workspace files with optional query filter.
#[tauri::command]
pub async fn list_workspace_files(
    services: State<'_, AppServices>,
    query: Option<String>,
) -> Result<serde_json::Value, String> {
    let result = services.file.list_files(query.as_deref());
    Ok(serde_json::json!({
        "files": result.files,
    }))
}
