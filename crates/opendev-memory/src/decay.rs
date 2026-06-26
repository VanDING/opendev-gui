use chrono::{DateTime, Utc};

use crate::types::MemoryEntry;

const NEW_MEMORY_DAYS: i64 = 7;
const HALFLIFE_DAYS: f64 = 30.0;
const ACCESS_BOOST_PER_ACCESS: f64 = 0.15;
const MAX_ACCESS_BOOST: f64 = 3.0;
const SYMBOL_LINK_BOOST: f64 = 1.2;

pub struct MemoryDecay;

impl MemoryDecay {
    pub fn score(entry: &MemoryEntry, now: DateTime<Utc>) -> f64 {
        Self::score_with_links(entry, now, false)
    }

    pub fn score_with_links(
        entry: &MemoryEntry,
        now: DateTime<Utc>,
        has_symbol_links: bool,
    ) -> f64 {
        let days = days_since_creation(entry, now);
        let recency_decay = if days <= NEW_MEMORY_DAYS as f64 {
            1.0
        } else {
            (-days / HALFLIFE_DAYS * 2.0_f64.ln()).exp()
        };
        let access_boost = 1.0 + (entry.access_count as f64) * ACCESS_BOOST_PER_ACCESS;
        let access_boost = access_boost.min(MAX_ACCESS_BOOST);
        let symbol_boost = if has_symbol_links { SYMBOL_LINK_BOOST } else { 1.0 };
        entry.importance * entry.confidence * recency_decay * access_boost * symbol_boost
    }

    pub fn is_new(entry: &MemoryEntry, now: DateTime<Utc>) -> bool {
        days_since_creation(entry, now) <= NEW_MEMORY_DAYS as f64
    }

    pub fn is_expired(entry: &MemoryEntry, now: DateTime<Utc>) -> bool {
        entry.expires_at.map(|exp| now > exp).unwrap_or(false)
    }
}

fn days_since_creation(entry: &MemoryEntry, now: DateTime<Utc>) -> f64 {
    now.signed_duration_since(entry.created_at).num_seconds() as f64 / 86_400.0
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::types::{MemoryCategory, MemorySource};

    fn entry(created: DateTime<Utc>, access_count: u32) -> MemoryEntry {
        MemoryEntry {
            id: "m1".into(),
            content: "content".into(),
            category: MemoryCategory::TechnicalNote,
            confidence: 1.0,
            source: MemorySource::Agent,
            project_path: Some(PathBuf::from("/tmp")),
            importance: 1.0,
            access_count,
            created_at: created,
            last_accessed_at: None,
            expires_at: None,
            tags: vec![],
        }
    }

    #[test]
    fn new_entry_has_high_score() {
        let now = DateTime::UNIX_EPOCH + chrono::Duration::days(1);
        let e = entry(DateTime::UNIX_EPOCH, 0);
        let score = MemoryDecay::score(&e, now);
        assert!(score > 0.9, "score = {score}");
        assert!(MemoryDecay::is_new(&e, now));
    }

    #[test]
    fn old_entry_decays() {
        let created = DateTime::UNIX_EPOCH;
        let now = created + chrono::Duration::days(60);
        let e = entry(created, 0);
        let score = MemoryDecay::score(&e, now);
        assert!(score < 0.5, "score = {score}");
    }

    #[test]
    fn access_boost_caps() {
        let now = DateTime::UNIX_EPOCH + chrono::Duration::days(1);
        let e = entry(DateTime::UNIX_EPOCH, 100);
        let score = MemoryDecay::score(&e, now);
        let expected = 1.0 * 1.0 * 1.0 * MAX_ACCESS_BOOST;
        assert!((score - expected).abs() < 0.01);
    }

    #[test]
    fn symbol_link_boost_applies() {
        let now = DateTime::UNIX_EPOCH + chrono::Duration::days(1);
        let e = entry(DateTime::UNIX_EPOCH, 0);
        let without_links = MemoryDecay::score(&e, now);
        let with_links = MemoryDecay::score_with_links(&e, now, true);
        assert!((with_links - without_links * SYMBOL_LINK_BOOST).abs() < 0.001);
    }

    #[test]
    fn expired_entry_detected() {
        let now = DateTime::UNIX_EPOCH + chrono::Duration::days(10);
        let mut e = entry(DateTime::UNIX_EPOCH, 0);
        e.expires_at = Some(DateTime::UNIX_EPOCH + chrono::Duration::days(5));
        assert!(MemoryDecay::is_expired(&e, now));
    }
}
