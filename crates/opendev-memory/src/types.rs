use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub content: String,
    pub category: MemoryCategory,
    pub confidence: f64,
    pub source: MemorySource,
    pub project_path: Option<PathBuf>,
    pub importance: f64,
    pub access_count: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryCategory {
    UserPreference,
    ProjectFact,
    Decision,
    Pattern,
    Feedback,
    TechnicalNote,
}

impl MemoryCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            MemoryCategory::UserPreference => "UserPreference",
            MemoryCategory::ProjectFact => "ProjectFact",
            MemoryCategory::Decision => "Decision",
            MemoryCategory::Pattern => "Pattern",
            MemoryCategory::Feedback => "Feedback",
            MemoryCategory::TechnicalNote => "TechnicalNote",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemorySource {
    User,
    Agent,
    System,
    Sideagent,
    Subagent,
}

impl MemorySource {
    pub fn as_str(&self) -> &'static str {
        match self {
            MemorySource::User => "User",
            MemorySource::Agent => "Agent",
            MemorySource::System => "System",
            MemorySource::Sideagent => "Sideagent",
            MemorySource::Subagent => "Subagent",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WriteGateTier {
    Working,
    Register,
    Daily,
    TransientNoise,
    StructuredPrefix,
}

#[derive(Debug, Clone, Default)]
pub struct MemorySessionContext {
    pub parent_id: Option<String>,
    pub is_ephemeral: bool,
}

impl MemorySessionContext {
    pub fn root() -> Self {
        Self { parent_id: None, is_ephemeral: false }
    }

    pub fn for_subagent(parent_id: String) -> Self {
        Self { parent_id: Some(parent_id), is_ephemeral: true }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RecallOptions {
    pub project_path: Option<PathBuf>,
    pub limit: usize,
    pub token_budget: Option<usize>,
}

impl RecallOptions {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedMemory {
    pub entry: MemoryEntry,
    pub relevance_score: f64,
    pub verification_timestamp: DateTime<Utc>,
}

#[async_trait::async_trait]
pub trait MemoryProvider: Send + Sync {
    fn id(&self) -> &'static str;

    async fn store(&self, entry: MemoryEntry) -> Result<(), String>;

    async fn recall(&self, query: &str, opts: RecallOptions) -> Result<Vec<MemoryEntry>, String>;

    async fn list(
        &self,
        project_path: Option<&std::path::Path>,
        limit: usize,
    ) -> Result<Vec<MemoryEntry>, String>;

    async fn delete(&self, id: &str) -> Result<(), String>;

    async fn flush(&self) -> Result<(), String>;

    async fn clear(&self) -> Result<(), String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_entry_round_trip() {
        let entry = MemoryEntry {
            id: "mem-1".into(),
            content: "use 4 spaces".into(),
            category: MemoryCategory::UserPreference,
            confidence: 0.9,
            source: MemorySource::User,
            project_path: None,
            importance: 0.7,
            access_count: 0,
            created_at: chrono::DateTime::UNIX_EPOCH,
            last_accessed_at: None,
            expires_at: None,
            tags: vec!["style".into()],
        };
        let json = serde_json::to_string(&entry).unwrap();
        let back: MemoryEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry.id, back.id);
        assert_eq!(entry.category, back.category);
    }
}
