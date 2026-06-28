use std::cmp::Ordering;
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

use sqlx::SqlitePool;
use std::sync::Mutex;
use tokio::sync::mpsc;

use crate::cascade::{CascadeBuffer, PendingMemory};
use crate::config::MemoryConfig;
use crate::decay::MemoryDecay;
use crate::error::MemoryError;
use crate::provider::SqliteMemoryProvider;
use crate::repo::MemoryRepo;
use crate::short_term::ShortTermMemory;
use crate::types::{
    MemoryCategory, MemoryEntry, MemoryProvider, MemorySessionContext, MemorySource, WriteGateTier,
};
use crate::write_gate::WriteGate;

#[derive(Debug, Clone, Default)]
pub struct DecayReport {
    pub expired_removed: usize,
    pub pruned: usize,
}

type PriorityKey = (bool, bool, f64, f64, f64);

fn priority_key(
    entry: &MemoryEntry,
    score: f64,
    is_new: bool,
    has_symbol_link: bool,
) -> PriorityKey {
    (is_new, has_symbol_link, score, entry.confidence, entry.importance)
}

pub struct WriteTask {
    pub entry: MemoryEntry,
}

pub struct MemoryFacade {
    pool: SqlitePool,
    provider: Arc<dyn MemoryProvider>,
    write_tx: Option<mpsc::UnboundedSender<WriteTask>>,
    pending_writes: Arc<AtomicUsize>,
    write_notify: Arc<tokio::sync::Notify>,
    write_error: Arc<Mutex<Option<MemoryError>>>,
    short_term: Mutex<ShortTermMemory>,
    cascade: Mutex<CascadeBuffer>,
    config: MemoryConfig,
}

impl MemoryFacade {
    pub async fn new(pool: SqlitePool) -> Self {
        let config = MemoryConfig::default();
        Self::new_with_config(pool, config).await
    }

    pub async fn new_with_config(pool: SqlitePool, config: MemoryConfig) -> Self {
        let provider: Arc<dyn MemoryProvider> = Arc::new(SqliteMemoryProvider::new(pool.clone()));
        let pending_writes = Arc::new(AtomicUsize::new(0));
        let write_notify = Arc::new(tokio::sync::Notify::new());
        let write_error = Arc::new(Mutex::new(None));

        let repo = MemoryRepo::new(pool.clone());
        if let Err(e) = repo.run_migrations().await {
            tracing::error!(error = %e, "memory migrations failed");
        }

        let write_tx = spawn_write_worker(
            Arc::clone(&provider),
            Arc::clone(&pending_writes),
            Arc::clone(&write_notify),
            Arc::clone(&write_error),
        );

        let short_term = ShortTermMemory::new(
            config.short_term_max_kv,
            config.short_term_max_turns,
            config.short_term_ttl,
        );
        let cascade = CascadeBuffer::new(config.cascade_min_count, config.cascade_max_age);

        Self {
            pool,
            provider,
            write_tx,
            pending_writes,
            write_notify,
            write_error,
            short_term: Mutex::new(short_term),
            cascade: Mutex::new(cascade),
            config,
        }
    }

    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.config.long_term_max_entries = capacity;
        self
    }

    fn capacity(&self) -> usize {
        self.config.long_term_max_entries
    }

    pub fn provider(&self) -> Arc<dyn MemoryProvider> {
        Arc::clone(&self.provider)
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn flush(&self) -> Result<(), MemoryError> {
        while self.pending_writes.load(AtomicOrdering::SeqCst) > 0 {
            self.write_notify.notified().await;
        }
        let mut guard = self.write_error.lock().map_err(|_| MemoryError::ChannelClosed)?;
        guard.take().map_or(Ok(()), Err)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn save(
        &self,
        content: &str,
        category: MemoryCategory,
        source: MemorySource,
        project_path: Option<&Path>,
        importance: f64,
        confidence: f64,
        context: &MemorySessionContext,
    ) -> Result<Option<String>, MemoryError> {
        if context.is_ephemeral {
            let key = format!("ephemeral:{}", uuid::Uuid::new_v4());
            if let Ok(mut g) = self.short_term.lock() {
                g.set(key, content.into());
            }
            return Ok(None);
        }

        let tier = WriteGate::classify(content);

        if tier == WriteGateTier::TransientNoise {
            tracing::debug!("dropping transient noise memory");
            return Ok(None);
        }

        if confidence < self.config.confidence_threshold {
            tracing::info!(confidence = %confidence, threshold = %self.config.confidence_threshold, "dropping low-confidence memory");
            return Ok(None);
        }

        match tier {
            WriteGateTier::Working => {
                let pending = PendingMemory {
                    content: content.into(),
                    category,
                    source,
                    project_path: project_path.map(Path::to_path_buf),
                    importance,
                    confidence,
                    staged_at: std::time::Instant::now(),
                    tier,
                };
                let should_flush_now = self
                    .cascade
                    .lock()
                    .ok()
                    .map(|mut g| {
                        g.stage(pending);
                        g.should_flush()
                    })
                    .unwrap_or(false);
                if should_flush_now {
                    self.flush_cascade().await?;
                }
                Ok(None)
            }
            _ => {
                let id = self
                    .write_to_long_term(
                        content,
                        category,
                        source,
                        project_path,
                        importance,
                        confidence,
                        tier,
                    )
                    .await?;
                self.prune_if_over_capacity().await?;
                Ok(Some(id))
            }
        }
    }

    pub async fn flush_cascade(&self) -> Result<Vec<String>, MemoryError> {
        let pending = self.cascade.lock().map(|mut g| g.flush()).unwrap_or_default();
        let mut ids = Vec::with_capacity(pending.len());
        for item in pending {
            let id = self
                .write_to_long_term(
                    &item.content,
                    item.category,
                    item.source,
                    item.project_path.as_deref(),
                    item.importance,
                    item.confidence,
                    item.tier,
                )
                .await?;
            ids.push(id);
        }
        self.prune_if_over_capacity().await?;
        Ok(ids)
    }

    #[allow(clippy::too_many_arguments)]
    async fn write_to_long_term(
        &self,
        content: &str,
        category: MemoryCategory,
        source: MemorySource,
        project_path: Option<&Path>,
        importance: f64,
        confidence: f64,
        tier: WriteGateTier,
    ) -> Result<String, MemoryError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now();
        let expires_at = match tier {
            WriteGateTier::Daily => Some(now + chrono::Duration::days(30)),
            _ => None,
        };
        let entry = MemoryEntry {
            id: id.clone(),
            content: content.into(),
            category,
            confidence,
            source,
            project_path: project_path.map(Path::to_path_buf),
            importance,
            access_count: 0,
            created_at: now,
            last_accessed_at: None,
            expires_at,
            tags: vec![],
        };

        if let Some(tx) = &self.write_tx {
            self.pending_writes.fetch_add(1, AtomicOrdering::SeqCst);
            if tx.send(WriteTask { entry }).is_err() {
                self.pending_writes.fetch_sub(1, AtomicOrdering::SeqCst);
                self.write_notify.notify_waiters();
                return Err(MemoryError::ChannelClosed);
            }
            return Ok(id);
        }

        let repo = MemoryRepo::new(self.pool.clone());
        repo.insert(&entry).await.map_err(MemoryError::Storage)?;

        Ok(id)
    }

    pub async fn search(
        &self,
        query: &str,
        project_path: Option<&Path>,
        limit: usize,
    ) -> Result<Vec<crate::types::VerifiedMemory>, MemoryError> {
        let repo = MemoryRepo::new(self.pool.clone());
        let entries =
            repo.search_fts(query, project_path, limit).await.map_err(MemoryError::Storage)?;
        let now = chrono::Utc::now();
        let mut verified = Vec::with_capacity(entries.len());
        for mut entry in entries {
            if entry.confidence < self.config.confidence_threshold {
                continue;
            }
            let _ = repo.touch(&entry.id, now).await;
            entry.access_count += 1;
            entry.last_accessed_at = Some(now);
            let score = MemoryDecay::score(&entry, now);
            verified.push(crate::types::VerifiedMemory {
                entry,
                relevance_score: score,
                verification_timestamp: now,
            });
        }
        Ok(verified)
    }

    pub async fn list(
        &self,
        project_path: Option<&Path>,
        limit: usize,
    ) -> Result<Vec<MemoryEntry>, MemoryError> {
        let repo = MemoryRepo::new(self.pool.clone());
        repo.list_by_project(project_path, limit).await.map_err(MemoryError::Storage)
    }

    pub async fn delete(&self, id: &str) -> Result<(), MemoryError> {
        let repo = MemoryRepo::new(self.pool.clone());
        repo.delete(id).await.map_err(MemoryError::Storage)
    }

    pub async fn link_symbol(
        &self,
        memory_id: &str,
        symbol_id: &str,
        symbol_name: &str,
        project_path: &Path,
    ) -> Result<(), MemoryError> {
        let repo = MemoryRepo::new(self.pool.clone());
        repo.link_symbol(memory_id, symbol_id, symbol_name, project_path)
            .await
            .map_err(MemoryError::Storage)
    }

    pub async fn recall_by_symbol(
        &self,
        symbol_name: &str,
        project_path: Option<&Path>,
        limit: usize,
    ) -> Result<Vec<MemoryEntry>, MemoryError> {
        let repo = MemoryRepo::new(self.pool.clone());
        repo.get_by_symbol(symbol_name, project_path, limit).await.map_err(MemoryError::Storage)
    }

    pub fn short_term_get(&self, key: &str) -> Option<String> {
        self.short_term.lock().ok().and_then(|mut g| g.get(key).map(|s| s.to_string()))
    }

    pub fn short_term_set(&self, key: String, value: String) {
        if let Ok(mut g) = self.short_term.lock() {
            g.set(key, value);
        }
    }

    pub fn push_turn(&self, log: String) {
        if let Ok(mut g) = self.short_term.lock() {
            g.push_turn(log);
        }
    }

    pub fn recent_turns(&self, limit: usize) -> Vec<String> {
        self.short_term
            .lock()
            .map(|g| g.recent_turns(limit).into_iter().map(|s| s.to_string()).collect())
            .unwrap_or_default()
    }

    pub async fn curate(&self) -> Result<DecayReport, MemoryError> {
        let linked_ids = self.symbol_linked_ids().await?;
        let count = self.long_term_count().await?;
        let repo = MemoryRepo::new(self.pool.clone());
        let all = repo.list_by_project(None, count).await.map_err(MemoryError::Storage)?;
        let now = chrono::Utc::now();
        let mut expired = 0;
        let mut scored: Vec<(MemoryEntry, PriorityKey)> = Vec::with_capacity(all.len());
        for entry in all {
            if MemoryDecay::is_expired(&entry, now) {
                let _ = repo.delete(&entry.id).await;
                expired += 1;
                continue;
            }
            let is_new = MemoryDecay::is_new(&entry, now);
            let has_link = linked_ids.contains(&entry.id);
            let score = MemoryDecay::score_with_links(&entry, now, has_link);
            let key = priority_key(&entry, if is_new { f64::MAX } else { score }, is_new, has_link);
            scored.push((entry, key));
        }

        let mut pruned = 0;
        if scored.len() > self.capacity() {
            scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
            let to_remove = scored.split_off(self.capacity());
            for (entry, _) in to_remove {
                let _ = repo.delete(&entry.id).await;
                pruned += 1;
            }
        }

        Ok(DecayReport { expired_removed: expired, pruned })
    }

    pub async fn recall_within_budget(
        &self,
        query: &str,
        project_path: Option<&Path>,
        token_budget: usize,
    ) -> Result<Vec<crate::types::VerifiedMemory>, MemoryError> {
        let repo = MemoryRepo::new(self.pool.clone());
        let candidates =
            repo.search_fts(query, project_path, 100).await.map_err(MemoryError::Storage)?;
        Ok(self.verify_and_budget(candidates, token_budget).await)
    }

    pub async fn recall_by_symbol_within_budget(
        &self,
        symbol_name: &str,
        project_path: Option<&Path>,
        token_budget: usize,
    ) -> Result<Vec<crate::types::VerifiedMemory>, MemoryError> {
        let repo = MemoryRepo::new(self.pool.clone());
        let candidates = repo
            .get_by_symbol(symbol_name, project_path, 100)
            .await
            .map_err(MemoryError::Storage)?;
        Ok(self.verify_and_budget(candidates, token_budget).await)
    }

    pub async fn recall_for_file_within_budget(
        &self,
        file_path: &Path,
        project_path: Option<&Path>,
        token_budget: usize,
    ) -> Result<Vec<crate::types::VerifiedMemory>, MemoryError> {
        let repo = MemoryRepo::new(self.pool.clone());
        let query = file_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| file_path.to_string_lossy().to_string());
        let candidates =
            repo.search_fts(&query, project_path, 100).await.map_err(MemoryError::Storage)?;
        Ok(self.verify_and_budget(candidates, token_budget).await)
    }

    async fn verify_and_budget(
        &self,
        candidates: Vec<MemoryEntry>,
        token_budget: usize,
    ) -> Vec<crate::types::VerifiedMemory> {
        let now = chrono::Utc::now();
        let verified: Vec<crate::types::VerifiedMemory> = candidates
            .into_iter()
            .map(|entry| {
                let score = MemoryDecay::score(&entry, now);
                crate::types::VerifiedMemory {
                    entry,
                    relevance_score: score,
                    verification_timestamp: now,
                }
            })
            .collect();
        apply_token_budget(verified, token_budget)
    }

    async fn prune_if_over_capacity(&self) -> Result<(), MemoryError> {
        // Flush background writes first so we have an accurate count
        self.flush().await?;
        let capacity = self.capacity();
        let count = self.long_term_count().await?;
        if count <= capacity {
            return Ok(());
        }

        let linked_ids = self.symbol_linked_ids().await?;
        let repo = MemoryRepo::new(self.pool.clone());
        let all = repo.list_by_project(None, count).await.map_err(MemoryError::Storage)?;
        let now = chrono::Utc::now();

        let mut scored: Vec<(MemoryEntry, PriorityKey)> = Vec::with_capacity(all.len());
        for entry in all {
            let is_new = MemoryDecay::is_new(&entry, now);
            let has_link = linked_ids.contains(&entry.id);
            let score = MemoryDecay::score_with_links(&entry, now, has_link);
            let key = priority_key(&entry, if is_new { f64::MAX } else { score }, is_new, has_link);
            scored.push((entry, key));
        }

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
        let to_remove = scored.split_off(capacity);
        for (entry, _) in to_remove {
            let _ = repo.delete(&entry.id).await;
        }
        Ok(())
    }

    async fn long_term_count(&self) -> Result<usize, MemoryError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM long_term_memory")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| MemoryError::Storage(e.to_string()))?;
        Ok(count.0 as usize)
    }

    async fn symbol_linked_ids(&self) -> Result<HashSet<String>, MemoryError> {
        let rows: Vec<(String,)> =
            sqlx::query_as("SELECT DISTINCT memory_id FROM memory_symbol_links")
                .fetch_all(&self.pool)
                .await
                .map_err(|e| MemoryError::Storage(e.to_string()))?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    pub fn auto_nudge(
        &self,
        turn_count: usize,
        has_decision: bool,
        session_ending: bool,
    ) -> Option<String> {
        if session_ending {
            return Some("Session ending — would you like to save a summary?".into());
        }
        if has_decision && turn_count > 10 {
            return Some("A decision was recorded — save it to long-term memory?".into());
        }
        if turn_count > 20 {
            return Some("You've had more than 20 turns — consider saving a memory.".into());
        }
        None
    }
}

fn spawn_write_worker(
    provider: Arc<dyn MemoryProvider>,
    pending: Arc<AtomicUsize>,
    notify: Arc<tokio::sync::Notify>,
    error_slot: Arc<Mutex<Option<MemoryError>>>,
) -> Option<mpsc::UnboundedSender<WriteTask>> {
    let handle = match tokio::runtime::Handle::try_current() {
        Ok(h) => h,
        Err(_) => return None,
    };

    let (tx, mut rx) = mpsc::unbounded_channel::<WriteTask>();
    handle.spawn(async move {
        while let Some(task) = rx.recv().await {
            if let Err(e) = provider.store(task.entry).await {
                tracing::error!(error = %e, "long-term memory background store failed");
                if let Ok(mut guard) = error_slot.lock()
                    && guard.is_none()
                {
                    *guard = Some(MemoryError::Provider(e));
                }
            }
            if pending.fetch_sub(1, AtomicOrdering::SeqCst) == 1 {
                notify.notify_waiters();
            }
        }
    });
    Some(tx)
}

fn apply_token_budget(
    mut verified: Vec<crate::types::VerifiedMemory>,
    token_budget: usize,
) -> Vec<crate::types::VerifiedMemory> {
    verified.sort_by(|a, b| {
        b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(Ordering::Equal).then_with(
            || b.entry.confidence.partial_cmp(&a.entry.confidence).unwrap_or(Ordering::Equal),
        )
    });

    let mut result = Vec::new();
    let mut used = 0usize;
    for v in verified {
        let cost = v.entry.content.chars().count() / 4;
        if used + cost > token_budget {
            break;
        }
        used += cost;
        result.push(v);
        if used >= token_budget {
            break;
        }
    }
    result
}
