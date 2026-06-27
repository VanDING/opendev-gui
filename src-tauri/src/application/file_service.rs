//! FileService — Workspace file operations.
//!
//! Handles file listing, path verification, and directory browsing.

use std::path::Path;

/// Directory entry for browse results.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DirEntry {
    pub name: String,
    pub path: String,
}

/// Browse directory response.
#[derive(Debug, Clone, serde::Serialize)]
pub struct BrowseDirResult {
    pub current_path: String,
    pub parent_path: Option<String>,
    pub directories: Vec<DirEntry>,
    pub error: Option<String>,
}

/// Path verification result.
#[derive(Debug, Clone, serde::Serialize)]
pub struct VerifyPathResult {
    pub exists: bool,
    pub is_directory: bool,
    pub path: Option<String>,
    pub error: Option<String>,
}

/// File entry for file listing.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FileEntry {
    pub path: String,
    pub name: String,
    pub is_file: bool,
}

/// File listing response.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FileListResult {
    pub files: Vec<FileEntry>,
}

pub struct FileService {
    working_dir: String,
}

impl FileService {
    pub fn new(working_dir: String) -> Self {
        Self { working_dir }
    }

    /// Browse a directory, returning subdirectories.
    pub fn browse_directory(&self, path: &str, show_hidden: bool) -> BrowseDirResult {
        let current_path = if path.is_empty() {
            self.working_dir.clone()
        } else {
            let p = Path::new(&self.working_dir).join(path);
            std::fs::canonicalize(&p)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| p.to_string_lossy().to_string())
        };

        let dir = Path::new(&current_path);
        if !dir.exists() || !dir.is_dir() {
            return BrowseDirResult {
                current_path,
                parent_path: None,
                directories: vec![],
                error: Some("Directory not found".to_string()),
            };
        }

        let parent = dir.parent().map(|p| p.to_string_lossy().to_string());

        let mut directories: Vec<DirEntry> = match dir.read_dir() {
            Ok(entries) => entries
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                .filter(|e| show_hidden || !e.file_name().to_string_lossy().starts_with('.'))
                .map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    let path = dir.join(&name).to_string_lossy().to_string();
                    DirEntry { name, path }
                })
                .collect(),
            Err(_) => vec![],
        };
        directories.sort_by(|a, b| a.name.cmp(&b.name));

        BrowseDirResult { current_path, parent_path: parent, directories, error: None }
    }

    /// Verify that a path exists within the workspace.
    pub fn verify_path(&self, path: &str) -> VerifyPathResult {
        let full_path = Path::new(&self.working_dir).join(path);
        let canonical = std::fs::canonicalize(&full_path);

        match canonical {
            Ok(p) => VerifyPathResult {
                exists: true,
                is_directory: p.is_dir(),
                path: Some(p.to_string_lossy().to_string()),
                error: None,
            },
            Err(e) => VerifyPathResult {
                exists: false,
                is_directory: false,
                path: None,
                error: Some(format!("{}", e)),
            },
        }
    }

    /// List files in the workspace, optionally filtered by query.
    pub fn list_files(&self, query: Option<&str>) -> FileListResult {
        let dir = Path::new(&self.working_dir);

        let mut files: Vec<FileEntry> = if let Some(q) = query {
            // Filter by query string against relative paths.
            self.collect_files(dir, dir, q, 0)
        } else {
            // Return top-level files and directories.
            match dir.read_dir() {
                Ok(entries) => entries
                    .filter_map(|e| e.ok())
                    .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
                    .map(|e| {
                        let name = e.file_name().to_string_lossy().to_string();
                        let path = dir.join(&name).to_string_lossy().to_string();
                        let is_file = e.file_type().map(|t| t.is_file()).unwrap_or(false);
                        FileEntry { path, name, is_file }
                    })
                    .collect(),
                Err(_) => vec![],
            }
        };

        files.sort_by(|a, b| {
            if a.is_file != b.is_file {
                a.is_file.cmp(&b.is_file) // directories first
            } else {
                a.name.cmp(&b.name)
            }
        });

        FileListResult { files }
    }

    fn collect_files(&self, base: &Path, dir: &Path, query: &str, depth: usize) -> Vec<FileEntry> {
        if depth > 3 {
            return vec![];
        }

        let mut results = Vec::new();
        let Ok(entries) = dir.read_dir() else {
            return results;
        };

        let query_lower = query.to_lowercase();

        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || name == "node_modules" || name == "target" {
                continue;
            }

            let full_path = entry.path();
            let is_file = entry.file_type().map(|t| t.is_file()).unwrap_or(false);
            let relative =
                full_path.strip_prefix(base).unwrap_or(&full_path).to_string_lossy().to_string();

            if relative.to_lowercase().contains(&query_lower) {
                results.push(FileEntry {
                    path: full_path.to_string_lossy().to_string(),
                    name: relative.clone(),
                    is_file,
                });
            }

            if !is_file {
                results.extend(self.collect_files(base, &full_path, query, depth + 1));
            }
        }

        results
    }
}
