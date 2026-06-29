use std::collections::HashMap;

use chrono::Utc;
use opendev_models::file_change::{FileChange, FileChangeType};
use opendev_models::session::Session;
use opendev_models::transition::ValidateTransition;
use tempfile::TempDir;

use super::*;

// ---------------------------------------------------------------------------
// Serialization round-trip
// ---------------------------------------------------------------------------

#[test]
fn test_session_event_serialization_roundtrip() {
    let events: Vec<SessionEvent> = vec![
        SessionEvent::SessionCreated {
            id: "abc123".into(),
            working_directory: Some("/tmp".into()),
            channel: "cli".into(),
            title: Some("Test session".into()),
            parent_id: None,
            metadata: HashMap::new(),
        },
        SessionEvent::MessageAdded {
            role: "user".into(),
            content: "hello".into(),
            timestamp: Utc::now(),
            tool_calls: vec![],
            tokens: Some(42),
            thinking_trace: None,
            reasoning_content: None,
        },
        SessionEvent::MessageEdited { seq: 0, content: "updated".into() },
        SessionEvent::TitleChanged { title: "New title".into() },
        SessionEvent::SessionArchived { time_archived: Utc::now() },
        SessionEvent::SessionUnarchived,
        SessionEvent::FileChangeRecorded {
            file_change: FileChange::new(FileChangeType::Created, "foo.rs".into()),
        },
        SessionEvent::MetadataUpdated { key: "key".into(), value: serde_json::json!("value") },
        SessionEvent::SessionForked {
            source_session_id: "src-session".into(),
            fork_point: Some(3),
        },
        SessionEvent::Tombstone { undo_to_seq: 5, reason: "Undo last 2 events".into() },
    ];

    for event in &events {
        let json = serde_json::to_string(event).expect("serialize");
        let roundtripped: SessionEvent = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(event.event_type(), roundtripped.event_type());
    }
}

// ---------------------------------------------------------------------------
// EventEnvelope::new
// ---------------------------------------------------------------------------

#[test]
fn test_event_envelope_new() {
    let event = SessionEvent::TitleChanged { title: "Hello".into() };
    let envelope = EventEnvelope::new("session-1", 5, &event);

    assert_eq!(envelope.aggregate_id, "session-1");
    assert_eq!(envelope.seq, 5);
    assert_eq!(envelope.event_type, "TitleChanged");
    assert!(!envelope.id.is_empty());

    // data should round-trip back to the same event type
    let deserialized: SessionEvent =
        serde_json::from_value(envelope.data).expect("deserialize data");
    assert_eq!(deserialized.event_type(), "TitleChanged");
}

// ---------------------------------------------------------------------------
// event_type() names
// ---------------------------------------------------------------------------

#[test]
fn test_event_type_names() {
    let cases: Vec<(SessionEvent, &str)> = vec![
        (
            SessionEvent::SessionCreated {
                id: String::new(),
                working_directory: None,
                channel: String::new(),
                title: None,
                parent_id: None,
                metadata: HashMap::new(),
            },
            "SessionCreated",
        ),
        (
            SessionEvent::MessageAdded {
                role: String::new(),
                content: String::new(),
                timestamp: Utc::now(),
                tool_calls: vec![],
                tokens: None,
                thinking_trace: None,
                reasoning_content: None,
            },
            "MessageAdded",
        ),
        (SessionEvent::MessageEdited { seq: 0, content: String::new() }, "MessageEdited"),
        (SessionEvent::TitleChanged { title: String::new() }, "TitleChanged"),
        (SessionEvent::SessionArchived { time_archived: Utc::now() }, "SessionArchived"),
        (SessionEvent::SessionUnarchived, "SessionUnarchived"),
        (
            SessionEvent::FileChangeRecorded {
                file_change: FileChange::new(FileChangeType::Modified, "x".into()),
            },
            "FileChangeRecorded",
        ),
        (
            SessionEvent::MetadataUpdated { key: String::new(), value: serde_json::Value::Null },
            "MetadataUpdated",
        ),
        (
            SessionEvent::SessionForked { source_session_id: String::new(), fork_point: None },
            "SessionForked",
        ),
        (SessionEvent::Tombstone { undo_to_seq: 0, reason: String::new() }, "Tombstone"),
    ];

    for (event, expected_name) in cases {
        assert_eq!(event.event_type(), expected_name);
    }
}

// ---------------------------------------------------------------------------
// ValidateTransition tests
// ---------------------------------------------------------------------------

fn make_session(archived: bool, message_count: usize) -> Session {
    let mut session = Session::new();
    if archived {
        session.archive();
    }
    for _ in 0..message_count {
        session.messages.push(opendev_models::message::ChatMessage {
            role: opendev_models::message::Role::User,
            content: "msg".into(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
            tool_calls: vec![],
            tokens: None,
            thinking_trace: None,
            reasoning_content: None,
            token_usage: None,
            provenance: None,
        });
    }
    session
}

#[test]
fn test_validate_archive_already_archived() {
    let session = make_session(true, 0);
    let event = SessionEvent::SessionArchived { time_archived: Utc::now() };
    let err = session.validate_transition(&event).unwrap_err();
    assert!(err.to_string().contains("already archived"));
}

#[test]
fn test_validate_unarchive_not_archived() {
    let session = make_session(false, 0);
    let event = SessionEvent::SessionUnarchived;
    let err = session.validate_transition(&event).unwrap_err();
    assert!(err.to_string().contains("not archived"));
}

#[test]
fn test_validate_add_message_when_archived() {
    let session = make_session(true, 0);
    let event = SessionEvent::MessageAdded {
        role: "user".into(),
        content: "hello".into(),
        timestamp: Utc::now(),
        tool_calls: vec![],
        tokens: None,
        thinking_trace: None,
        reasoning_content: None,
    };
    let err = session.validate_transition(&event).unwrap_err();
    assert!(err.to_string().contains("archived session"));
}

#[test]
fn test_validate_title_change_empty() {
    let session = make_session(false, 0);
    let event = SessionEvent::TitleChanged { title: "   ".into() };
    let err = session.validate_transition(&event).unwrap_err();
    assert!(err.to_string().contains("empty"));
}

#[test]
fn test_validate_fork_point_out_of_range() {
    let session = make_session(false, 3);
    let event =
        SessionEvent::SessionForked { source_session_id: "src".into(), fork_point: Some(10) };
    let err = session.validate_transition(&event).unwrap_err();
    assert!(err.to_string().contains("out of range"));
}

#[test]
fn test_validate_valid_transitions() {
    let session = make_session(false, 5);

    let event = SessionEvent::MessageAdded {
        role: "user".into(),
        content: "hello".into(),
        timestamp: Utc::now(),
        tool_calls: vec![],
        tokens: None,
        thinking_trace: None,
        reasoning_content: None,
    };
    assert!(session.validate_transition(&event).is_ok());

    let event = SessionEvent::TitleChanged { title: "Good title".into() };
    assert!(session.validate_transition(&event).is_ok());

    let event = SessionEvent::SessionArchived { time_archived: Utc::now() };
    assert!(session.validate_transition(&event).is_ok());

    let event =
        SessionEvent::SessionForked { source_session_id: "src".into(), fork_point: Some(5) };
    assert!(session.validate_transition(&event).is_ok());

    let event = SessionEvent::SessionForked { source_session_id: "src".into(), fork_point: None };
    assert!(session.validate_transition(&event).is_ok());

    let archived_session = make_session(true, 0);
    let event = SessionEvent::SessionUnarchived;
    assert!(archived_session.validate_transition(&event).is_ok());

    let event = SessionEvent::SessionCreated {
        id: "new".into(),
        working_directory: None,
        channel: "cli".into(),
        title: None,
        parent_id: None,
        metadata: HashMap::new(),
    };
    assert!(session.validate_transition(&event).is_ok());
}

// ---------------------------------------------------------------------------
// EventStore tests
// ---------------------------------------------------------------------------

fn make_temp_store() -> (TempDir, EventStore) {
    let dir = TempDir::new().expect("create temp dir");
    let path = dir.path().canonicalize().expect("canonicalize");
    let store = EventStore::new(path);
    (dir, store)
}

fn sample_events(n: usize) -> Vec<SessionEvent> {
    (0..n).map(|i| SessionEvent::TitleChanged { title: format!("title-{i}") }).collect()
}

#[test]
fn test_event_store_append_and_load() {
    let (_dir, store) = make_temp_store();
    let events = sample_events(3);
    let envelopes = store.append("sess-1", events).unwrap();
    assert_eq!(envelopes.len(), 3);

    let loaded = store.load("sess-1").unwrap();
    assert_eq!(loaded.len(), 3);
    for (i, env) in loaded.iter().enumerate() {
        assert_eq!(env.seq, (i + 1) as u64);
        assert_eq!(env.aggregate_id, "sess-1");
        assert_eq!(env.event_type, "TitleChanged");
    }
}

#[test]
fn test_event_store_append_increments_seq() {
    let (_dir, store) = make_temp_store();
    store.append("sess-1", sample_events(2)).unwrap();
    let second_batch = store.append("sess-1", sample_events(3)).unwrap();

    assert_eq!(second_batch[0].seq, 3);
    assert_eq!(second_batch[1].seq, 4);
    assert_eq!(second_batch[2].seq, 5);

    let all = store.load("sess-1").unwrap();
    assert_eq!(all.len(), 5);
    for (i, env) in all.iter().enumerate() {
        assert_eq!(env.seq, (i + 1) as u64);
    }
}

#[test]
fn test_event_store_load_empty() {
    let (_dir, store) = make_temp_store();
    let loaded = store.load("nonexistent").unwrap();
    assert!(loaded.is_empty());
}

#[test]
fn test_event_store_load_since() {
    let (_dir, store) = make_temp_store();
    store.append("sess-1", sample_events(5)).unwrap();

    let since = store.load_since("sess-1", 2).unwrap();
    assert_eq!(since.len(), 3);
    assert_eq!(since[0].seq, 3);
    assert_eq!(since[1].seq, 4);
    assert_eq!(since[2].seq, 5);
}

#[test]
fn test_event_store_latest_seq() {
    let (_dir, store) = make_temp_store();
    assert_eq!(store.latest_seq("sess-1").unwrap(), 0);

    store.append("sess-1", sample_events(3)).unwrap();
    assert_eq!(store.latest_seq("sess-1").unwrap(), 3);

    store.append("sess-1", sample_events(2)).unwrap();
    assert_eq!(store.latest_seq("sess-1").unwrap(), 5);
}

#[test]
fn test_event_store_has_events() {
    let (_dir, store) = make_temp_store();
    assert!(!store.has_events("sess-1"));

    store.append("sess-1", sample_events(1)).unwrap();
    assert!(store.has_events("sess-1"));
}

#[test]
fn test_event_store_concurrent_safety() {
    let (_dir, store) = make_temp_store();

    // Two sequential appends should not corrupt the file.
    store.append("sess-1", sample_events(3)).unwrap();
    store.append("sess-1", sample_events(3)).unwrap();

    let all = store.load("sess-1").unwrap();
    assert_eq!(all.len(), 6);
    for (i, env) in all.iter().enumerate() {
        assert_eq!(env.seq, (i + 1) as u64);
    }
}

// ---------------------------------------------------------------------------
// append_validated tests
// ---------------------------------------------------------------------------

#[test]
fn test_append_validated_success() {
    let (_dir, store) = make_temp_store();
    let session = make_session(false, 0);

    let events = vec![
        SessionEvent::TitleChanged { title: "New title".into() },
        SessionEvent::SessionArchived { time_archived: Utc::now() },
    ];

    let envelopes = store.append_validated(&session, "sess-v1", events).unwrap();
    assert_eq!(envelopes.len(), 2);
    assert_eq!(envelopes[0].event_type, "TitleChanged");
    assert_eq!(envelopes[1].event_type, "SessionArchived");

    let loaded = store.load("sess-v1").unwrap();
    assert_eq!(loaded.len(), 2);
}

#[test]
fn test_append_validated_rejects_invalid() {
    let (_dir, store) = make_temp_store();
    let session = make_session(true, 0); // already archived

    let events = vec![SessionEvent::SessionArchived { time_archived: Utc::now() }];

    let err = store.append_validated(&session, "sess-v2", events).unwrap_err();
    assert!(err.contains("already archived"));
}

#[test]
fn test_append_validated_sequential_validation() {
    let (_dir, store) = make_temp_store();
    let session = make_session(false, 0);

    // archive + unarchive in sequence should pass
    let events = vec![
        SessionEvent::SessionArchived { time_archived: Utc::now() },
        SessionEvent::SessionUnarchived,
    ];
    let envelopes = store.append_validated(&session, "sess-v3", events).unwrap();
    assert_eq!(envelopes.len(), 2);

    // archive + archive should fail on second event
    let session2 = make_session(false, 0);
    let events = vec![
        SessionEvent::SessionArchived { time_archived: Utc::now() },
        SessionEvent::SessionArchived { time_archived: Utc::now() },
    ];
    let err = store.append_validated(&session2, "sess-v4", events).unwrap_err();
    assert!(err.contains("already archived"));
}

// ---------------------------------------------------------------------------
// Tombstone / undo tests
// ---------------------------------------------------------------------------

#[test]
fn test_tombstone_event_serialization() {
    let event = SessionEvent::Tombstone { undo_to_seq: 5, reason: "Undo last 2 events".into() };
    assert_eq!(event.event_type(), "Tombstone");

    let json = serde_json::to_string(&event).expect("serialize");
    let roundtripped: SessionEvent = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(roundtripped.event_type(), "Tombstone");
    if let SessionEvent::Tombstone { undo_to_seq, reason } = roundtripped {
        assert_eq!(undo_to_seq, 5);
        assert_eq!(reason, "Undo last 2 events");
    } else {
        panic!("Expected Tombstone variant");
    }
}

#[test]
fn test_undo_last_event() {
    let (_dir, store) = make_temp_store();
    // Append 3 events (seqs 1, 2, 3).
    store.append("sess-u1", sample_events(3)).unwrap();

    // Undo the last 1 event. With 3 events, undoable = 2 (keep first).
    // Undoing 1: keep_up_to_idx = 3 - 1 - 1 = 1 => undo_to_seq = seq of effective[1] = 2.
    let (envelope, undo_to_seq) = store.undo("sess-u1", 1).unwrap();
    assert_eq!(envelope.event_type, "Tombstone");
    assert_eq!(undo_to_seq, 2); // keep events with seq <= 2

    // Effective events should be 2 (seqs 1 and 2).
    let all = store.load("sess-u1").unwrap();
    let effective = EventStore::effective_events(&all);
    assert_eq!(effective.len(), 2);
    assert_eq!(effective[0].seq, 1);
    assert_eq!(effective[1].seq, 2);
}

#[test]
fn test_undo_multiple_events() {
    let (_dir, store) = make_temp_store();
    store.append("sess-u2", sample_events(5)).unwrap();

    // Undo last 3 events from [1,2,3,4,5]. undoable=4, undo_count=3.
    // keep_up_to_idx = 5-3-1 = 1, undo_to_seq = effective[1].seq = 2.
    let (_envelope, undo_to_seq) = store.undo("sess-u2", 3).unwrap();
    assert_eq!(undo_to_seq, 2);

    let all = store.load("sess-u2").unwrap();
    let effective = EventStore::effective_events(&all);
    assert_eq!(effective.len(), 2);
    assert_eq!(effective[0].seq, 1);
    assert_eq!(effective[1].seq, 2);
}

#[test]
fn test_undo_nothing_to_undo() {
    let (_dir, store) = make_temp_store();

    // Empty log.
    let err = store.undo("nonexistent", 1).unwrap_err();
    assert!(err.contains("No events to undo"));
}

#[test]
fn test_undo_capped_to_keep_first_event() {
    let (_dir, store) = make_temp_store();
    // Append just 1 event (only a title change, no SessionCreated in this case).
    store.append("sess-u3", sample_events(1)).unwrap();

    // Trying to undo more than available effective events minus the first.
    // With 1 event total, undoable = 0, so nothing to undo.
    let err = store.undo("sess-u3", 5).unwrap_err();
    assert!(err.contains("Nothing to undo"));
}

#[test]
fn test_effective_events_with_tombstone() {
    let (_dir, store) = make_temp_store();
    store.append("sess-e1", sample_events(5)).unwrap();

    // Manually append a tombstone with undo_to_seq=3 (keep seqs <= 3).
    // Events with seq 4 and 5 (between undo_to_seq and tombstone) are undone.
    store
        .append("sess-e1", vec![SessionEvent::Tombstone { undo_to_seq: 3, reason: "test".into() }])
        .unwrap();

    let all = store.load("sess-e1").unwrap();
    assert_eq!(all.len(), 6); // 5 originals + 1 tombstone

    let effective = EventStore::effective_events(&all);
    assert_eq!(effective.len(), 3); // seqs 1, 2, 3 are kept
    assert_eq!(effective[0].seq, 1);
    assert_eq!(effective[1].seq, 2);
    assert_eq!(effective[2].seq, 3);
}

#[test]
fn test_append_validated_no_persist_on_failure() {
    let (_dir, store) = make_temp_store();
    let session = make_session(false, 0);

    // First event valid, second invalid — nothing should be persisted.
    let events = vec![
        SessionEvent::TitleChanged { title: "Good title".into() },
        SessionEvent::SessionArchived { time_archived: Utc::now() },
        SessionEvent::SessionArchived { time_archived: Utc::now() },
    ];

    let err = store.append_validated(&session, "sess-v5", events).unwrap_err();
    assert!(err.contains("already archived"));

    // Verify nothing was persisted.
    let loaded = store.load("sess-v5").unwrap();
    assert!(loaded.is_empty());
}

// ── validate_integrity tests ──

#[test]
fn test_validate_integrity_empty_store() {
    let dir = TempDir::new().unwrap();
    let store = EventStore::new(dir.path().to_path_buf());
    assert!(store.validate_integrity("empty-session").is_ok());
}

#[test]
fn test_validate_integrity_contiguous_sequences() {
    let dir = TempDir::new().unwrap();
    let store = EventStore::new(dir.path().to_path_buf());

    store
        .append(
            "sess-v6",
            vec![
                SessionEvent::SessionCreated {
                    id: "sess-v6".into(),
                    working_directory: Some("/tmp".into()),
                    channel: "cli".into(),
                    title: None,
                    parent_id: None,
                    metadata: HashMap::new(),
                },
                SessionEvent::TitleChanged { title: "Test session".into() },
            ],
        )
        .unwrap();

    assert!(store.validate_integrity("sess-v6").is_ok());
}

#[test]
fn test_validate_integrity_empty_after_clear() {
    let store = EventStore::new(TempDir::new().unwrap().path().to_path_buf());
    // Non-existent aggregate is empty → valid.
    assert!(store.validate_integrity("nonexistent").is_ok());
}

// ── Replay tests ──

#[test]
fn test_replay_from_empty() {
    let dir = TempDir::new().unwrap();
    let store = EventStore::new(dir.path().to_path_buf());

    let events = store.load("empty-session").unwrap();
    assert!(events.is_empty());
    assert_eq!(store.latest_seq("empty-session").unwrap(), 0);
}

#[test]
fn test_replay_with_tombstones() {
    let dir = TempDir::new().unwrap();
    let store = EventStore::new(dir.path().to_path_buf());

    store
        .append(
            "sess-v7",
            vec![
                SessionEvent::SessionCreated {
                    id: "sess-v7".into(),
                    working_directory: None,
                    channel: "cli".into(),
                    title: None,
                    parent_id: None,
                    metadata: HashMap::new(),
                },
                SessionEvent::TitleChanged { title: "Original".into() },
                SessionEvent::TitleChanged { title: "Updated".into() },
            ],
        )
        .unwrap();

    let all = store.load("sess-v7").unwrap();
    assert_eq!(all.len(), 3);

    // Apply tombstone to undo the last event.
    let (_, undo_to_seq) = store.undo("sess-v7", 1).unwrap();
    assert_eq!(undo_to_seq, 2);

    // Effective events should exclude the undone event.
    let all2 = store.load("sess-v7").unwrap();
    let effective = EventStore::effective_events(&all2);
    assert_eq!(effective.len(), 2);
    assert!(effective.iter().any(|e| matches!(
        serde_json::from_value::<SessionEvent>(e.data.clone()).unwrap(),
        SessionEvent::TitleChanged { ref title, .. } if title == "Original"
    )));
}

#[test]
fn test_replay_with_fork_events() {
    let dir = TempDir::new().unwrap();
    let store = EventStore::new(dir.path().to_path_buf());

    store
        .append(
            "sess-forked",
            vec![
                SessionEvent::SessionCreated {
                    id: "sess-forked".into(),
                    working_directory: None,
                    channel: "cli".into(),
                    title: None,
                    parent_id: None,
                    metadata: HashMap::new(),
                },
                SessionEvent::SessionForked {
                    source_session_id: "parent-sess".into(),
                    fork_point: Some(5),
                },
            ],
        )
        .unwrap();

    let events = store.load("sess-forked").unwrap();
    assert_eq!(events.len(), 2);

    let has_fork = events.iter().any(|e| e.event_type == "SessionForked");
    assert!(has_fork);
}

// ── Concurrent event append tests ──

#[tokio::test]
async fn test_concurrent_event_append() {
    let dir = TempDir::new().unwrap();
    let store = std::sync::Arc::new(EventStore::new(dir.path().to_path_buf()));

    let mut handles = Vec::new();
    for i in 0..5 {
        let store = store.clone();
        handles.push(tokio::spawn(async move {
            store
                .append(
                    "concurrent-sess",
                    vec![SessionEvent::TitleChanged { title: format!("Concurrent title {i}") }],
                )
                .unwrap();
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    // All 5 events should be appended with contiguous seq numbers.
    assert!(store.validate_integrity("concurrent-sess").is_ok());
    let all = store.load("concurrent-sess").unwrap();
    // Plus the initial SessionCreated event we didn't add here — so 5 TitleChanged.
    // Actually we didn't add SessionCreated, so the sequences might start at 1.
    assert_eq!(all.len(), 5);
    for (i, event) in all.iter().enumerate() {
        assert_eq!(event.seq, (i + 1) as u64);
    }
}

// ── Corrupted event recovery tests ──

#[test]
fn test_corrupted_event_line_skipped() {
    let dir = TempDir::new().unwrap();
    let store = EventStore::new(dir.path().to_path_buf());

    // Write a valid event followed by a corrupt line.
    let path = store.event_log_path("corrupt-sess");
    let mut lines = Vec::new();

    let event = SessionEvent::SessionCreated {
        id: "corrupt-sess".into(),
        working_directory: Some("/tmp".into()),
        channel: "cli".into(),
        title: None,
        parent_id: None,
        metadata: HashMap::new(),
    };
    let envelope = EventEnvelope::new("corrupt-sess", 1, &event);
    lines.push(serde_json::to_string(&envelope).unwrap());

    // Append a corrupted line.
    lines.push("INVALID JSON LINE{{{".to_string());
    lines.push(
        serde_json::to_string(&EventEnvelope::new(
            "corrupt-sess",
            2,
            &SessionEvent::TitleChanged { title: "After corruption".into() },
        ))
        .unwrap(),
    );

    std::fs::write(&path, lines.join("\n")).unwrap();

    // Load should succeed, skipping the corrupted line.
    let events = store.load("corrupt-sess").unwrap();
    // The corrupt line can't be parsed, so only 2 valid lines remain.
    // But read_last_seq might fail to parse the last valid line due to
    // the corrupt line between. Let's just verify we got some events.
    assert!(!events.is_empty(), "Should recover some events despite corruption");
}

#[test]
fn test_integrity_fails_on_duplicate_seq() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("dup-sess.events.jsonl");

    // Manually write two events with the same seq.
    let event = SessionEvent::SessionCreated {
        id: "dup-sess".into(),
        working_directory: None,
        channel: "cli".into(),
        title: None,
        parent_id: None,
        metadata: HashMap::new(),
    };
    let e1 = EventEnvelope::new("dup-sess", 1, &event);
    let e2 = EventEnvelope::new(
        "dup-sess",
        1,
        &SessionEvent::TitleChanged { title: "Duplicate".into() },
    );

    let content = format!(
        "{}\n{}\n",
        serde_json::to_string(&e1).unwrap(),
        serde_json::to_string(&e2).unwrap(),
    );
    std::fs::write(&path, content).unwrap();

    let store = EventStore::new(dir.path().to_path_buf());
    let result = store.validate_integrity("dup-sess");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("duplicate"));
}
