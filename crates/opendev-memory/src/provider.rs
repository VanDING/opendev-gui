use std::path::Path;

use sqlx::SqlitePool;

use crate::repo::MemoryRepo;
use crate::types::{MemoryEntry, MemoryProvider, RecallOptions};

#[derive(Clone)]
pub struct SqliteMemoryProvider {
    pool: SqlitePool,
}

impl SqliteMemoryProvider {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl MemoryProvider for SqliteMemoryProvider {
    fn id(&self) -> &'static str {
        "sqlite"
    }

    async fn store(&self, entry: MemoryEntry) -> Result<(), String> {
        let repo = MemoryRepo::new(self.pool.clone());
        repo.insert(&entry).await
    }

    async fn recall(&self, query: &str, opts: RecallOptions) -> Result<Vec<MemoryEntry>, String> {
        let repo = MemoryRepo::new(self.pool.clone());
        let limit = if opts.limit == 0 { 50 } else { opts.limit.min(200) };
        repo.search_fts(query, opts.project_path.as_deref(), limit).await
    }

    async fn list(
        &self,
        project_path: Option<&Path>,
        limit: usize,
    ) -> Result<Vec<MemoryEntry>, String> {
        let repo = MemoryRepo::new(self.pool.clone());
        let limit = if limit == 0 { 100 } else { limit.min(1000) };
        repo.list_by_project(project_path, limit).await
    }

    async fn delete(&self, id: &str) -> Result<(), String> {
        let repo = MemoryRepo::new(self.pool.clone());
        repo.delete(id).await
    }

    async fn flush(&self) -> Result<(), String> {
        Ok(())
    }

    async fn clear(&self) -> Result<(), String> {
        sqlx::query("DELETE FROM long_term_memory")
            .execute(&self.pool)
            .await
            .map_err(|e| format!("clear failed: {}", e))?;
        Ok(())
    }
}
