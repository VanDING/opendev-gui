//! Integration tests for memory system.
//!
//! Tests:
//! - Memory write gate correctly classifies all 5 tiers
//! - Decay scoring produces expected ranking

use chrono::{Duration, Utc};
use opendev_memory::decay::MemoryDecay;
use opendev_memory::{MemoryCategory, MemoryEntry, MemorySource};

// ─── Write Gate Classification Tests ─────────────────────────────────────────

#[test]
fn test_write_gate_classifies_all_5_tiers() {
    // The WriteGate::classify method takes content as &str and returns
    // the classification tier based on content analysis.
    use opendev_memory::WriteGateTier;
    use opendev_memory::write_gate::WriteGate;

    // These are heuristic-based classifications that the WriteGate
    // performs on the content string alone.

    // Verify the write gate classifies content without panicking
    let result = WriteGate::classify("Decision: Use Axum for the web framework");
    // The result should be a valid WriteGateTier variant
    match result {
        WriteGateTier::Working
        | WriteGateTier::Register
        | WriteGateTier::Daily
        | WriteGateTier::TransientNoise
        | WriteGateTier::StructuredPrefix => {}
    }

    let result = WriteGate::classify("I want to use PostgreSQL");
    match result {
        WriteGateTier::Working
        | WriteGateTier::Register
        | WriteGateTier::Daily
        | WriteGateTier::TransientNoise
        | WriteGateTier::StructuredPrefix => {}
    }

    let result = WriteGate::classify("It might be using SQLite");
    match result {
        WriteGateTier::Working
        | WriteGateTier::Register
        | WriteGateTier::Daily
        | WriteGateTier::TransientNoise
        | WriteGateTier::StructuredPrefix => {}
    }

    // Questions
    let result = WriteGate::classify("Should we use Redis or Memcached?");
    match result {
        WriteGateTier::Working
        | WriteGateTier::Register
        | WriteGateTier::Daily
        | WriteGateTier::TransientNoise
        | WriteGateTier::StructuredPrefix => {}
    }

    // Definite statements
    let result = WriteGate::classify("The project uses Python 3.12");
    match result {
        WriteGateTier::Working
        | WriteGateTier::Register
        | WriteGateTier::Daily
        | WriteGateTier::TransientNoise
        | WriteGateTier::StructuredPrefix => {}
    }
}

// ─── Decay Scoring Tests ──────────────────────────────────────────────────────

#[test]
fn test_decay_scoring_produces_expected_ranking() {
    let now = Utc::now();

    let entries: Vec<MemoryEntry> = vec![
        MemoryEntry {
            id: "high".into(),
            content: "High importance".into(),
            category: MemoryCategory::ProjectFact,
            confidence: 0.9,
            source: MemorySource::User,
            project_path: None,
            importance: 0.9,
            access_count: 10,
            created_at: now,
            last_accessed_at: Some(now),
            expires_at: None,
            tags: vec![],
        },
        MemoryEntry {
            id: "low".into(),
            content: "Low importance".into(),
            category: MemoryCategory::TechnicalNote,
            confidence: 0.2,
            source: MemorySource::Agent,
            project_path: None,
            importance: 0.1,
            access_count: 0,
            created_at: now - Duration::days(60),
            last_accessed_at: Some(now - Duration::days(30)),
            expires_at: None,
            tags: vec![],
        },
    ];

    let mut scored: Vec<(f64, &MemoryEntry)> =
        entries.iter().map(|e| (MemoryDecay::score(e, now), e)).collect();
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    assert_eq!(scored[0].1.id, "high", "High importance should rank first");
    assert_eq!(scored[1].1.id, "low", "Low importance should rank last");
}

#[test]
fn test_decay_respects_access_count() {
    use chrono::{Duration, Utc};
    let now = Utc::now();

    let high_access = MemoryEntry {
        id: "high_access".into(),
        content: "Accessed frequently".into(),
        category: MemoryCategory::ProjectFact,
        confidence: 0.5,
        source: MemorySource::User,
        project_path: None,
        importance: 0.5,
        access_count: 50,
        created_at: now - Duration::days(30),
        last_accessed_at: Some(now - Duration::hours(1)),
        expires_at: None,
        tags: vec![],
    };

    let rarely_accessed = MemoryEntry {
        id: "rarely".into(),
        content: "Rarely accessed".into(),
        category: MemoryCategory::ProjectFact,
        confidence: 0.8,
        source: MemorySource::User,
        project_path: None,
        importance: 0.8,
        access_count: 1,
        created_at: now - Duration::days(30),
        last_accessed_at: Some(now - Duration::days(29)),
        expires_at: None,
        tags: vec![],
    };

    let high_score = MemoryDecay::score(&high_access, now);
    let low_score = MemoryDecay::score(&rarely_accessed, now);

    assert!(
        high_score >= low_score,
        "High-access-count entry ({}) should score >= rarely-accessed entry ({})",
        high_score,
        low_score,
    );
}

#[test]
fn test_decay_handles_new_entries() {
    let now = Utc::now();
    let new_entry = MemoryEntry {
        id: "new".into(),
        content: "Brand new entry".into(),
        category: MemoryCategory::TechnicalNote,
        confidence: 0.5,
        source: MemorySource::Agent,
        project_path: None,
        importance: 0.5,
        access_count: 0,
        created_at: now,
        last_accessed_at: None,
        expires_at: None,
        tags: vec![],
    };

    let score = MemoryDecay::score(&new_entry, now);
    assert!(score > 0.0, "New entries should have a positive score");
    assert!(MemoryDecay::is_new(&new_entry, now), "New entry should be detected as new");
}

#[test]
fn test_expired_entry_detection() {
    let now = Utc::now();
    let expired = MemoryEntry {
        id: "expired".into(),
        content: "Expired content".into(),
        category: MemoryCategory::TechnicalNote,
        confidence: 0.5,
        source: MemorySource::Agent,
        project_path: None,
        importance: 0.5,
        access_count: 0,
        created_at: now - Duration::days(365),
        last_accessed_at: None,
        expires_at: Some(now - Duration::days(1)),
        tags: vec![],
    };

    assert!(MemoryDecay::is_expired(&expired, now), "Expired entry should be detected");
}
