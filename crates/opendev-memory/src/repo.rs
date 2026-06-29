use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

use crate::types::{MemoryCategory, MemoryEntry, MemorySource};

pub struct MemoryRepo {
    pool: SqlitePool,
}

impl MemoryRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn run_migrations(&self) -> Result<(), String> {
        for sql in crate::migration::MIGRATIONS {
            let result = sqlx::query(sql).execute(&self.pool).await;
            if let Err(ref e) = result {
                // FTS5 may not be available in all SQLite builds; log and continue.
                if sql.contains("fts5") {
                    tracing::warn!(error = %e, "FTS5 not available (needs SQLite with FTS5 enabled)");
                    continue;
                }
                return Err(format!("Migration failed: {}", e));
            }
        }
        Ok(())
    }

    pub async fn insert(&self, entry: &MemoryEntry) -> Result<(), String> {
        let mut tx = self.pool.begin().await.map_err(|e| format!("tx begin: {}", e))?;

        sqlx::query(
            "INSERT OR REPLACE INTO long_term_memory \
             (id, content, category, confidence, source, project_path, importance, \
              access_count, last_accessed_at, expires_at, created_at, tags_json) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        )
        .bind(&entry.id)
        .bind(&entry.content)
        .bind(entry.category.as_str())
        .bind(entry.confidence)
        .bind(entry.source.as_str())
        .bind(entry.project_path.as_ref().map(|p| p.to_string_lossy().to_string()))
        .bind(entry.importance)
        .bind(entry.access_count as i64)
        .bind(entry.last_accessed_at.map(|t| t.to_rfc3339()))
        .bind(entry.expires_at.map(|t| t.to_rfc3339()))
        .bind(entry.created_at.to_rfc3339())
        .bind(serde_json::to_string(&entry.tags).unwrap_or_default())
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("insert: {}", e))?;

        tx.commit().await.map_err(|e| format!("tx commit: {}", e))?;
        Ok(())
    }

    pub async fn get(&self, id: &str) -> Result<Option<MemoryEntry>, String> {
        let row = sqlx::query_as::<_, MemoryRow>(
            "SELECT id, content, category, confidence, source, project_path, importance, \
             access_count, last_accessed_at, expires_at, created_at, tags_json \
             FROM long_term_memory WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("get: {}", e))?;

        row.map(|r| r.try_into()).transpose()
    }

    pub async fn update(&self, entry: &MemoryEntry) -> Result<(), String> {
        let mut tx = self.pool.begin().await.map_err(|e| format!("tx begin: {}", e))?;

        sqlx::query(
            "UPDATE long_term_memory SET \
             content = ?2, category = ?3, confidence = ?4, source = ?5, \
             project_path = ?6, importance = ?7, access_count = ?8, \
             last_accessed_at = ?9, expires_at = ?10, tags_json = ?11 \
             WHERE id = ?1",
        )
        .bind(&entry.id)
        .bind(&entry.content)
        .bind(entry.category.as_str())
        .bind(entry.confidence)
        .bind(entry.source.as_str())
        .bind(entry.project_path.as_ref().map(|p| p.to_string_lossy().to_string()))
        .bind(entry.importance)
        .bind(entry.access_count as i64)
        .bind(entry.last_accessed_at.map(|t| t.to_rfc3339()))
        .bind(entry.expires_at.map(|t| t.to_rfc3339()))
        .bind(serde_json::to_string(&entry.tags).unwrap_or_default())
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("update: {}", e))?;

        tx.commit().await.map_err(|e| format!("tx commit: {}", e))?;
        Ok(())
    }

    pub async fn delete(&self, id: &str) -> Result<(), String> {
        let mut tx = self.pool.begin().await.map_err(|e| format!("tx begin: {}", e))?;

        sqlx::query("DELETE FROM long_term_memory WHERE id = ?1")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("delete: {}", e))?;

        tx.commit().await.map_err(|e| format!("tx commit: {}", e))?;
        Ok(())
    }

    /// Search memory using FTS5 with LIKE fallback.
    ///
    /// First attempts an FTS5 MATCH query (requires the `long_term_memory_fts`
    /// virtual table). If FTS5 is not available or the query fails, falls back
    /// to the standard `LIKE` query.
    pub async fn search_fts(
        &self,
        query: &str,
        project: Option<&Path>,
        limit: usize,
    ) -> Result<Vec<MemoryEntry>, String> {
        // Try FTS5 first
        let project_str = project.map(|p| p.to_string_lossy().to_string());
        let fts_result = self.search_fts5(query, project_str.as_deref(), limit).await;
        match fts_result {
            Ok(results) if !results.is_empty() => return Ok(results),
            Ok(_) => {}  // FTS returned empty, fall through to LIKE
            Err(_) => {} // FTS failed, fall through to LIKE
        }

        // Fallback: LIKE query
        self.search_like(query, project, limit).await
    }

    /// FTS5-based search using MATCH syntax.
    async fn search_fts5(
        &self,
        query: &str,
        project: Option<&str>,
        limit: usize,
    ) -> Result<Vec<MemoryEntry>, String> {
        // Clean the query for FTS5 — remove special chars
        let fts_query = query
            .split_whitespace()
            .filter(|w| !w.is_empty())
            .map(|w| {
                // Escape FTS5 special characters and wrap in quotes if needed
                if w.contains('"') || w.contains('\'') {
                    format!("\"{}\"", w.replace('"', "\"\""))
                } else {
                    format!("\"{w}\"")
                }
            })
            .collect::<Vec<_>>()
            .join(" AND ");

        if fts_query.is_empty() {
            return Err("empty query".to_string());
        }

        if let Some(p) = project {
            sqlx::query_as::<_, MemoryRow>(
                "SELECT id, content, category, confidence, source, project_path, importance, \
                 access_count, last_accessed_at, expires_at, created_at, tags_json \
                 FROM long_term_memory_fts \
                 WHERE long_term_memory_fts MATCH ?1 AND project_path = ?2 \
                 ORDER BY rank LIMIT ?3",
            )
            .bind(&fts_query)
            .bind(p)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("fts5 search: {}", e))?
            .into_iter()
            .map(|r| r.try_into())
            .collect::<Result<Vec<_>, String>>()
        } else {
            sqlx::query_as::<_, MemoryRow>(
                "SELECT id, content, category, confidence, source, project_path, importance, \
                 access_count, last_accessed_at, expires_at, created_at, tags_json \
                 FROM long_term_memory_fts \
                 WHERE long_term_memory_fts MATCH ?1 \
                 ORDER BY rank LIMIT ?2",
            )
            .bind(&fts_query)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("fts5 search: {}", e))?
            .into_iter()
            .map(|r| r.try_into())
            .collect::<Result<Vec<_>, String>>()
        }
    }

    /// LIKE-based fallback search.
    async fn search_like(
        &self,
        query: &str,
        project: Option<&Path>,
        limit: usize,
    ) -> Result<Vec<MemoryEntry>, String> {
        let pattern = format!("%{}%", escape_like(query));
        let project_str = project.map(|p| p.to_string_lossy().to_string());

        if let Some(ref p) = project_str {
            sqlx::query_as::<_, MemoryRow>(
                "SELECT id, content, category, confidence, source, project_path, importance, \
                 access_count, last_accessed_at, expires_at, created_at, tags_json \
                 FROM long_term_memory \
                  WHERE content LIKE ?1 ESCAPE '\\' AND project_path = ?2 \
                 ORDER BY importance DESC LIMIT ?3",
            )
            .bind(&pattern)
            .bind(p)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("search: {}", e))?
            .into_iter()
            .map(|r| r.try_into())
            .collect::<Result<Vec<_>, String>>()
        } else {
            sqlx::query_as::<_, MemoryRow>(
                "SELECT id, content, category, confidence, source, project_path, importance, \
                 access_count, last_accessed_at, expires_at, created_at, tags_json \
                 FROM long_term_memory \
                  WHERE content LIKE ?1 ESCAPE '\\' \
                 ORDER BY importance DESC LIMIT ?2",
            )
            .bind(&pattern)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("search: {}", e))?
            .into_iter()
            .map(|r| r.try_into())
            .collect::<Result<Vec<_>, String>>()
        }
    }

    pub async fn list_by_project(
        &self,
        project: Option<&Path>,
        limit: usize,
    ) -> Result<Vec<MemoryEntry>, String> {
        if let Some(p) = project {
            let p_str = p.to_string_lossy().to_string();
            sqlx::query_as::<_, MemoryRow>(
                "SELECT id, content, category, confidence, source, project_path, importance, \
                 access_count, last_accessed_at, expires_at, created_at, tags_json \
                 FROM long_term_memory WHERE project_path = ?1 \
                 ORDER BY created_at DESC LIMIT ?2",
            )
            .bind(&p_str)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("list: {}", e))?
            .into_iter()
            .map(|r| r.try_into())
            .collect()
        } else {
            sqlx::query_as::<_, MemoryRow>(
                "SELECT id, content, category, confidence, source, project_path, importance, \
                 access_count, last_accessed_at, expires_at, created_at, tags_json \
                 FROM long_term_memory ORDER BY created_at DESC LIMIT ?1",
            )
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("list: {}", e))?
            .into_iter()
            .map(|r| r.try_into())
            .collect()
        }
    }

    pub async fn list_for_decay(
        &self,
        before: DateTime<Utc>,
        limit: usize,
    ) -> Result<Vec<MemoryEntry>, String> {
        sqlx::query_as::<_, MemoryRow>(
            "SELECT id, content, category, confidence, source, project_path, importance, \
             access_count, last_accessed_at, expires_at, created_at, tags_json \
             FROM long_term_memory \
             WHERE last_accessed_at < ?1 OR (last_accessed_at IS NULL AND created_at < ?1) \
             ORDER BY created_at ASC LIMIT ?2",
        )
        .bind(before.to_rfc3339())
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("list_for_decay: {}", e))?
        .into_iter()
        .map(|r| r.try_into())
        .collect()
    }

    pub async fn link_symbol(
        &self,
        memory_id: &str,
        symbol_id: &str,
        symbol_name: &str,
        project_path: &Path,
    ) -> Result<(), String> {
        let p = project_path.to_string_lossy().to_string();
        sqlx::query(
            "INSERT INTO memory_symbol_links (memory_id, symbol_id, symbol_name, project_path) \
             VALUES (?1, ?2, ?3, ?4) \
             ON CONFLICT(memory_id, symbol_id) DO UPDATE SET symbol_name = excluded.symbol_name",
        )
        .bind(memory_id)
        .bind(symbol_id)
        .bind(symbol_name)
        .bind(&p)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("link_symbol: {}", e))?;
        Ok(())
    }

    pub async fn get_by_symbol(
        &self,
        symbol_name: &str,
        project: Option<&Path>,
        limit: usize,
    ) -> Result<Vec<MemoryEntry>, String> {
        if let Some(p) = project {
            let p_str = p.to_string_lossy().to_string();
            sqlx::query_as::<_, MemoryRow>(
                "SELECT m.id, m.content, m.category, m.confidence, m.source, m.project_path, \
                 m.importance, m.access_count, m.last_accessed_at, m.expires_at, m.created_at, m.tags_json \
                 FROM long_term_memory m \
                 JOIN memory_symbol_links s ON m.id = s.memory_id \
                 WHERE s.symbol_name = ?1 AND m.project_path = ?2 \
                 ORDER BY m.created_at DESC LIMIT ?3",
            )
            .bind(symbol_name)
            .bind(&p_str)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("get_by_symbol: {}", e))?
            .into_iter()
            .map(|r| r.try_into())
            .collect()
        } else {
            sqlx::query_as::<_, MemoryRow>(
                "SELECT m.id, m.content, m.category, m.confidence, m.source, m.project_path, \
                 m.importance, m.access_count, m.last_accessed_at, m.expires_at, m.created_at, m.tags_json \
                 FROM long_term_memory m \
                 JOIN memory_symbol_links s ON m.id = s.memory_id \
                 WHERE s.symbol_name = ?1 \
                 ORDER BY m.created_at DESC LIMIT ?2",
            )
            .bind(symbol_name)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("get_by_symbol: {}", e))?
            .into_iter()
            .map(|r| r.try_into())
            .collect()
        }
    }

    pub async fn touch(&self, id: &str, at: DateTime<Utc>) -> Result<(), String> {
        sqlx::query(
            "UPDATE long_term_memory SET access_count = access_count + 1, last_accessed_at = ?1 WHERE id = ?2",
        )
        .bind(at.to_rfc3339())
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("touch: {}", e))?;
        Ok(())
    }
}

#[derive(Debug, sqlx::FromRow)]
struct MemoryRow {
    id: String,
    content: String,
    category: String,
    confidence: f64,
    source: String,
    project_path: Option<String>,
    importance: f64,
    access_count: i64,
    last_accessed_at: Option<String>,
    expires_at: Option<String>,
    created_at: String,
    tags_json: String,
}

impl TryInto<MemoryEntry> for MemoryRow {
    type Error = String;

    fn try_into(self) -> Result<MemoryEntry, String> {
        let parse_dt = |s: &str| -> Result<DateTime<Utc>, String> {
            DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| format!("invalid datetime '{}': {}", s, e))
        };

        Ok(MemoryEntry {
            id: self.id,
            content: self.content,
            category: parse_category(&self.category),
            confidence: self.confidence,
            source: parse_source(&self.source),
            project_path: self.project_path.map(PathBuf::from),
            importance: self.importance,
            access_count: self.access_count as u32,
            created_at: parse_dt(&self.created_at)?,
            last_accessed_at: self.last_accessed_at.as_deref().map(parse_dt).transpose()?,
            expires_at: self.expires_at.as_deref().map(parse_dt).transpose()?,
            tags: serde_json::from_str(&self.tags_json).unwrap_or_default(),
        })
    }
}

fn parse_category(s: &str) -> MemoryCategory {
    match s {
        "ProjectFact" => MemoryCategory::ProjectFact,
        "Decision" => MemoryCategory::Decision,
        "Pattern" => MemoryCategory::Pattern,
        "Feedback" => MemoryCategory::Feedback,
        "TechnicalNote" => MemoryCategory::TechnicalNote,
        _ => MemoryCategory::UserPreference,
    }
}

fn parse_source(s: &str) -> MemorySource {
    match s {
        "Agent" => MemorySource::Agent,
        "System" => MemorySource::System,
        "Sideagent" => MemorySource::Sideagent,
        "Subagent" => MemorySource::Subagent,
        _ => MemorySource::User,
    }
}

fn escape_like(pattern: &str) -> String {
    let mut escaped = String::with_capacity(pattern.len() + 8);
    for ch in pattern.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '%' => escaped.push_str("\\%"),
            '_' => escaped.push_str("\\_"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MemorySource;
    use sqlx::sqlite::SqliteConnectOptions;
    use tempfile::TempDir;

    #[tokio::test]
    async fn basic_like_query() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.db");
        let opts = SqliteConnectOptions::new().filename(&path).create_if_missing(true);
        let pool = SqlitePool::connect_with(opts).await.unwrap();
        sqlx::query("CREATE TABLE t (id TEXT, content TEXT)").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO t VALUES (?1, ?2)")
            .bind("1")
            .bind("hello world")
            .execute(&pool)
            .await
            .unwrap();
        let pattern = "%world%";
        let rows: Vec<(String,)> = sqlx::query_as("SELECT content FROM t WHERE content LIKE ?1")
            .bind(pattern)
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
    }

    fn test_pool() -> (TempDir, SqlitePool) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.db");
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let pool = runtime.block_on(async {
            let opts = sqlx::sqlite::SqliteConnectOptions::new()
                .filename(&path)
                .create_if_missing(true)
                .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);
            let pool = sqlx::SqlitePool::connect_with(opts).await.unwrap();
            for sql in crate::migration::MIGRATIONS {
                sqlx::query(sql).execute(&pool).await.unwrap();
            }
            pool
        });
        (dir, pool)
    }

    fn sample_entry(id: &str) -> MemoryEntry {
        MemoryEntry {
            id: id.into(),
            content: "use cargo fmt".into(),
            category: MemoryCategory::TechnicalNote,
            confidence: 0.9,
            source: MemorySource::Agent,
            project_path: None,
            importance: 0.5,
            access_count: 0,
            created_at: DateTime::UNIX_EPOCH,
            last_accessed_at: None,
            expires_at: None,
            tags: vec![],
        }
    }

    #[test]
    fn insert_and_get_memory() {
        let (_dir, pool) = test_pool();
        let repo = MemoryRepo::new(pool);
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let entry = sample_entry("m1");
            repo.insert(&entry).await.unwrap();
            let loaded = repo.get("m1").await.unwrap().unwrap();
            assert_eq!(loaded.content, entry.content);
            assert_eq!(loaded.category, entry.category);
        });
    }

    #[test]
    fn delete_memory() {
        let (_dir, pool) = test_pool();
        let repo = MemoryRepo::new(pool);
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let entry = sample_entry("m1");
            repo.insert(&entry).await.unwrap();
            repo.delete("m1").await.unwrap();
            assert!(repo.get("m1").await.unwrap().is_none());
        });
    }
}
