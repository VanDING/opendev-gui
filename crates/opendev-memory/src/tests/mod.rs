#[cfg(test)]
mod facade_tests {
    use std::collections::HashSet;
    use std::path::Path;

    use sqlx::SqlitePool;
    use tempfile::TempDir;

    use crate::repo::MemoryRepo;
    use crate::{
        MemoryCategory, MemoryConfig, MemoryEntry, MemoryFacade, MemorySessionContext, MemorySource,
    };

    async fn test_facade() -> (TempDir, MemoryFacade) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.db");
        let opts = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(&path)
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);
        let pool = SqlitePool::connect_with(opts).await.unwrap();
        let config = MemoryConfig { db_path: path, ..Default::default() };
        (dir, MemoryFacade::new_with_config(pool, config).await)
    }

    #[tokio::test]
    async fn save_and_search_long_term() {
        let (_dir, facade) = test_facade().await;
        let id = facade
            .save(
                "decision: use cargo fmt before committing",
                MemoryCategory::TechnicalNote,
                MemorySource::Agent,
                None,
                0.8,
                0.9,
                &MemorySessionContext::root(),
            )
            .await
            .unwrap()
            .unwrap();
        assert!(!id.is_empty());

        // Wait for background write worker to flush
        facade.flush().await.unwrap();

        let results = facade.search("cargo fmt", None, 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entry.content, "decision: use cargo fmt before committing");
    }

    #[tokio::test]
    async fn transient_noise_dropped() {
        let (_dir, facade) = test_facade().await;
        let id = facade
            .save(
                "timeout while connecting",
                MemoryCategory::TechnicalNote,
                MemorySource::Agent,
                None,
                0.5,
                0.5,
                &MemorySessionContext::root(),
            )
            .await
            .unwrap();
        assert!(id.is_none());
    }

    #[tokio::test]
    async fn cascade_flushes_after_min_count() {
        let (_dir, facade) = test_facade().await;
        for i in 0..2 {
            facade
                .save(
                    &format!("working note {i}"),
                    MemoryCategory::TechnicalNote,
                    MemorySource::Agent,
                    None,
                    0.5,
                    0.9,
                    &MemorySessionContext::root(),
                )
                .await
                .unwrap();
        }
        let ids = facade.flush_cascade().await.unwrap();
        facade.flush().await.unwrap();
        assert_eq!(ids.len(), 2);
        let results = facade.list(None, 10).await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn auto_nudge_triggers() {
        let (_dir, facade) = test_facade().await;
        assert!(facade.auto_nudge(25, false, false).is_some());
        assert!(facade.auto_nudge(15, true, false).is_some());
        assert!(facade.auto_nudge(5, false, true).is_some());
        assert!(facade.auto_nudge(5, false, false).is_none());
    }

    #[tokio::test]
    async fn symbol_link_and_recall() {
        let (_dir, facade) = test_facade().await;
        let id = facade
            .save(
                "decision: main function handles CLI args",
                MemoryCategory::ProjectFact,
                MemorySource::Agent,
                Some(Path::new("/proj")),
                0.8,
                0.9,
                &MemorySessionContext::root(),
            )
            .await
            .unwrap()
            .unwrap();

        // Wait for background write before linking
        facade.flush().await.unwrap();

        facade.link_symbol(&id, "sym-1", "main", Path::new("/proj")).await.unwrap();
        let results = facade.recall_by_symbol("main", Some(Path::new("/proj")), 10).await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn confidence_threshold_filters_save_and_search() {
        let (_dir, facade) = test_facade().await;
        let high = facade
            .save(
                "decision: use 4 spaces",
                MemoryCategory::UserPreference,
                MemorySource::Agent,
                None,
                0.9,
                0.9,
                &MemorySessionContext::root(),
            )
            .await
            .unwrap()
            .unwrap();
        let low = facade
            .save(
                "decision: maybe 2 spaces",
                MemoryCategory::UserPreference,
                MemorySource::Agent,
                None,
                0.9,
                0.5,
                &MemorySessionContext::root(),
            )
            .await
            .unwrap();
        assert!(low.is_none());

        facade.flush().await.unwrap();

        let results = facade.search("spaces", None, 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entry.id, high);
    }

    #[tokio::test]
    async fn proactive_pruning_respects_capacity_and_symbol_links() {
        let (_dir, base) = test_facade().await;
        let facade = base.with_capacity(2);
        let a = facade
            .save(
                "important architectural decision",
                MemoryCategory::Decision,
                MemorySource::Agent,
                None,
                0.9,
                0.9,
                &MemorySessionContext::root(),
            )
            .await
            .unwrap()
            .unwrap();
        let b = facade
            .save(
                "decision: minor style note",
                MemoryCategory::UserPreference,
                MemorySource::Agent,
                None,
                0.2,
                0.9,
                &MemorySessionContext::root(),
            )
            .await
            .unwrap()
            .unwrap();

        // Wait for background writes before linking
        facade.flush().await.unwrap();

        facade.link_symbol(&b, "sym-b", "style", Path::new("/proj")).await.unwrap();

        // Save third item — prune_if_over_capacity is triggered automatically
        let _c = facade
            .save(
                "decision: medium importance note",
                MemoryCategory::TechnicalNote,
                MemorySource::Agent,
                None,
                0.5,
                0.9,
                &MemorySessionContext::root(),
            )
            .await
            .unwrap()
            .unwrap();

        facade.flush().await.unwrap();

        let all = facade.list(None, 10).await.unwrap();
        assert_eq!(all.len(), 2, "expected 2 after auto-prune, got {}", all.len());
        let ids: HashSet<_> = all.iter().map(|e| e.id.as_str()).collect();
        assert!(ids.contains(&a.as_str()));
        assert!(ids.contains(&b.as_str()));
    }

    #[tokio::test]
    async fn curate_prunes_old_entries() {
        let (_dir, base) = test_facade().await;
        let facade = base.with_capacity(2);
        let now = chrono::Utc::now();
        let repo = MemoryRepo::new(facade.pool().clone());
        let old1 = MemoryEntry {
            id: "old1".into(),
            content: "old low importance".into(),
            category: MemoryCategory::TechnicalNote,
            confidence: 0.9,
            source: MemorySource::Agent,
            project_path: None,
            importance: 0.1,
            access_count: 0,
            created_at: now - chrono::Duration::days(60),
            last_accessed_at: None,
            expires_at: None,
            tags: vec![],
        };
        let old2 = MemoryEntry {
            id: "old2".into(),
            content: "old high importance".into(),
            category: MemoryCategory::TechnicalNote,
            confidence: 0.9,
            source: MemorySource::Agent,
            project_path: None,
            importance: 0.9,
            access_count: 0,
            created_at: now - chrono::Duration::days(60),
            last_accessed_at: None,
            expires_at: None,
            tags: vec![],
        };
        repo.insert(&old1).await.unwrap();
        repo.insert(&old2).await.unwrap();

        let report = facade.curate().await.unwrap();
        assert_eq!(report.pruned, 0);

        let old3 = MemoryEntry {
            id: "old3".into(),
            content: "old medium importance".into(),
            category: MemoryCategory::TechnicalNote,
            confidence: 0.9,
            source: MemorySource::Agent,
            project_path: None,
            importance: 0.5,
            access_count: 0,
            created_at: now - chrono::Duration::days(60),
            last_accessed_at: None,
            expires_at: None,
            tags: vec![],
        };
        repo.insert(&old3).await.unwrap();

        let report = facade.curate().await.unwrap();
        assert_eq!(report.pruned, 1);
        let remaining: HashSet<_> =
            facade.list(None, 10).await.unwrap().iter().map(|e| e.id.clone()).collect();
        assert!(!remaining.contains("old1"));
        assert!(remaining.contains("old2"));
    }

    #[tokio::test]
    async fn recall_within_budget_respects_token_limit() {
        let (_dir, facade) = test_facade().await;
        facade
            .save(
                "decision: use async runtime for all I/O operations",
                MemoryCategory::Decision,
                MemorySource::Agent,
                None,
                0.9,
                0.9,
                &MemorySessionContext::root(),
            )
            .await
            .unwrap()
            .unwrap();
        facade.flush().await.unwrap();

        let with_small_budget =
            facade.recall_within_budget("async runtime", None, 5).await.unwrap();
        assert!(with_small_budget.is_empty());

        let with_large_budget =
            facade.recall_within_budget("async runtime", None, 20).await.unwrap();
        assert_eq!(with_large_budget.len(), 1);
    }

    #[tokio::test]
    async fn ephemeral_save_stays_in_short_term() {
        let (_dir, facade) = test_facade().await;
        let ctx = MemorySessionContext::for_subagent("parent-1".into());
        let id = facade
            .save(
                "subagent exploration note",
                MemoryCategory::TechnicalNote,
                MemorySource::Agent,
                None,
                0.8,
                0.9,
                &ctx,
            )
            .await
            .unwrap();
        assert!(id.is_none(), "ephemeral saves should not return a long-term id");
        let long_term = facade.list(None, 10).await.unwrap();
        assert!(long_term.is_empty(), "ephemeral saves should not reach long-term memory");
    }
}
