use super::*;

#[test]
fn test_schema_sql_is_valid_syntax() {
    assert!(CREATE_SESSIONS_TABLE.contains("CREATE TABLE"));
    assert!(CREATE_SESSIONS_TABLE.contains("sessions"));
    assert!(CREATE_SESSIONS_TABLE.contains("TEXT PRIMARY KEY"));

    assert!(CREATE_MESSAGES_TABLE.contains("CREATE TABLE"));
    assert!(CREATE_MESSAGES_TABLE.contains("messages"));
    assert!(CREATE_MESSAGES_TABLE.contains("REFERENCES sessions"));
    assert!(CREATE_MESSAGES_TABLE.contains("ON DELETE CASCADE"));
}

#[test]
fn test_create_indexes_count() {
    assert_eq!(CREATE_INDEXES.len(), 5);
    for sql in CREATE_INDEXES {
        assert!(sql.starts_with("CREATE INDEX"));
    }
}

#[test]
fn test_sqlite_store_open() {
    let tmp = tempfile::tempdir().unwrap();
    let db_path = tmp.path().join("test.db");
    let store = SqliteSessionStore::open(&db_path).unwrap();
    assert_eq!(store.db_path(), db_path);
}

#[test]
fn test_sqlite_store_save_and_load_roundtrip() {
    let tmp = tempfile::tempdir().unwrap();
    let store = SqliteSessionStore::open(tmp.path().join("test.db")).unwrap();
    let mut session = Session::new();
    session.messages.push(ChatMessage {
        role: Role::User,
        content: "hello".to_string(),
        timestamp: chrono::Utc::now(),
        metadata: std::collections::HashMap::new(),
        tool_calls: vec![],
        tokens: None,
        thinking_trace: None,
        reasoning_content: None,
        token_usage: None,
        provenance: None,
    });
    session.messages.push(ChatMessage {
        role: Role::Assistant,
        content: "world".to_string(),
        timestamp: chrono::Utc::now(),
        metadata: std::collections::HashMap::new(),
        tool_calls: vec![],
        tokens: None,
        thinking_trace: None,
        reasoning_content: None,
        token_usage: None,
        provenance: None,
    });

    store.save_session(&session).unwrap();
    let loaded = store.load_session(&session.id).unwrap();

    assert_eq!(loaded.id, session.id);
    assert_eq!(loaded.messages.len(), 2);
    assert_eq!(loaded.messages[0].content, "hello");
    assert_eq!(loaded.messages[1].content, "world");
}

#[test]
fn test_sqlite_store_load_session_not_found() {
    let tmp = tempfile::tempdir().unwrap();
    let store = SqliteSessionStore::open(tmp.path().join("test.db")).unwrap();
    let result = store.load_session("nonexistent-id");
    assert!(result.is_err());
}

#[test]
fn test_sqlite_store_delete_session() {
    let tmp = tempfile::tempdir().unwrap();
    let store = SqliteSessionStore::open(tmp.path().join("test.db")).unwrap();
    let session = Session::new();
    store.save_session(&session).unwrap();
    assert!(store.load_session(&session.id).is_ok());
    store.delete_session(&session.id).unwrap();
    assert!(store.load_session(&session.id).is_err());
}

#[test]
fn test_sqlite_store_delete_nonexistent() {
    let tmp = tempfile::tempdir().unwrap();
    let store = SqliteSessionStore::open(tmp.path().join("test.db")).unwrap();
    // Deleting a non-existent session should succeed (idempotent)
    assert!(store.delete_session("ghost-id").is_ok());
}

#[test]
fn test_sqlite_store_list_sessions() {
    let tmp = tempfile::tempdir().unwrap();
    let store = SqliteSessionStore::open(tmp.path().join("test.db")).unwrap();

    // Initially empty
    let ids = store.list_session_ids().unwrap();
    assert!(ids.is_empty());

    let s1 = Session::new();
    let mut s2 = Session::new();
    s2.id = "second-session".to_string();
    store.save_session(&s1).unwrap();
    store.save_session(&s2).unwrap();

    let ids = store.list_session_ids().unwrap();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&s1.id));
    assert!(ids.contains(&s2.id));
}

#[test]
fn test_sqlite_store_search_messages() {
    let tmp = tempfile::tempdir().unwrap();
    let store = SqliteSessionStore::open(tmp.path().join("test.db")).unwrap();

    let mut session = Session::new();
    session.messages.push(ChatMessage {
        role: Role::User,
        content: "how do I write a parser in rust?".to_string(),
        timestamp: chrono::Utc::now(),
        metadata: std::collections::HashMap::new(),
        tool_calls: vec![],
        tokens: None,
        thinking_trace: None,
        reasoning_content: None,
        token_usage: None,
        provenance: None,
    });
    session.messages.push(ChatMessage {
        role: Role::Assistant,
        content: "here is a parser implementation".to_string(),
        timestamp: chrono::Utc::now(),
        metadata: std::collections::HashMap::new(),
        tool_calls: vec![],
        tokens: None,
        thinking_trace: None,
        reasoning_content: None,
        token_usage: None,
        provenance: None,
    });
    store.save_session(&session).unwrap();

    let results = store.search_messages("parser").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, session.id);
    assert_eq!(results[0].1.len(), 2); // both messages contain "parser"

    let results = store.search_messages("python").unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_sqlite_store_search_messages_like_wildcard_percent() {
    let tmp = tempfile::tempdir().unwrap();
    let store = SqliteSessionStore::open(tmp.path().join("test.db")).unwrap();

    let mut session = Session::new();
    session.messages.push(ChatMessage {
        role: Role::User,
        content: "50% discount on rust books".to_string(),
        timestamp: chrono::Utc::now(),
        metadata: std::collections::HashMap::new(),
        tool_calls: vec![],
        tokens: None,
        thinking_trace: None,
        reasoning_content: None,
        token_usage: None,
        provenance: None,
    });
    session.messages.push(ChatMessage {
        role: Role::Assistant,
        content: "the sale is over".to_string(),
        timestamp: chrono::Utc::now(),
        metadata: std::collections::HashMap::new(),
        tool_calls: vec![],
        tokens: None,
        thinking_trace: None,
        reasoning_content: None,
        token_usage: None,
        provenance: None,
    });
    store.save_session(&session).unwrap();

    // Searching for literal '%' should only match the message containing '%',
    // NOT all messages via wildcard expansion.
    let results = store.search_messages("%").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, session.id);
    assert_eq!(results[0].1.len(), 1); // only "50% discount..." contains literal %
}

#[test]
fn test_sqlite_store_search_messages_like_wildcard_underscore() {
    let tmp = tempfile::tempdir().unwrap();
    let store = SqliteSessionStore::open(tmp.path().join("test.db")).unwrap();

    let mut session = Session::new();
    session.messages.push(ChatMessage {
        role: Role::User,
        content: "the variable name is foo_bar".to_string(),
        timestamp: chrono::Utc::now(),
        metadata: std::collections::HashMap::new(),
        tool_calls: vec![],
        tokens: None,
        thinking_trace: None,
        reasoning_content: None,
        token_usage: None,
        provenance: None,
    });
    session.messages.push(ChatMessage {
        role: Role::Assistant,
        content: "refactored to fooBar".to_string(),
        timestamp: chrono::Utc::now(),
        metadata: std::collections::HashMap::new(),
        tool_calls: vec![],
        tokens: None,
        thinking_trace: None,
        reasoning_content: None,
        token_usage: None,
        provenance: None,
    });
    store.save_session(&session).unwrap();

    // Searching for literal '_' should only match the message containing '_',
    // NOT match every single-character position via wildcard.
    let results = store.search_messages("_").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, session.id);
    assert_eq!(results[0].1.len(), 1); // only "foo_bar" contains literal _
}

#[test]
fn test_escape_like_unit() {
    assert_eq!(escape_like("hello"), "hello");
    assert_eq!(escape_like("50%"), "50\\%");
    assert_eq!(escape_like("foo_bar"), "foo\\_bar");
    assert_eq!(escape_like(r"test\path"), "test\\\\path");
    assert_eq!(escape_like("%like_"), "\\%like\\_");
    assert_eq!(escape_like(""), "");
}

#[test]
fn test_sqlite_store_append_message() {
    let tmp = tempfile::tempdir().unwrap();
    let store = SqliteSessionStore::open(tmp.path().join("test.db")).unwrap();
    let session = Session::new();
    store.save_session(&session).unwrap();

    let msg = ChatMessage {
        role: Role::User,
        content: "appended message".to_string(),
        timestamp: chrono::Utc::now(),
        metadata: std::collections::HashMap::new(),
        tool_calls: vec![],
        tokens: None,
        thinking_trace: None,
        reasoning_content: None,
        token_usage: None,
        provenance: None,
    };
    store.append_message(&session.id, &msg).unwrap();

    let loaded = store.load_session(&session.id).unwrap();
    assert_eq!(loaded.messages.len(), 1);
    assert_eq!(loaded.messages[0].content, "appended message");
}

#[test]
fn test_sqlite_store_append_nonexistent_session() {
    let tmp = tempfile::tempdir().unwrap();
    let store = SqliteSessionStore::open(tmp.path().join("test.db")).unwrap();
    let msg = ChatMessage {
        role: Role::User,
        content: "hello".to_string(),
        timestamp: chrono::Utc::now(),
        metadata: std::collections::HashMap::new(),
        tool_calls: vec![],
        tokens: None,
        thinking_trace: None,
        reasoning_content: None,
        token_usage: None,
        provenance: None,
    };
    // Appending to a nonexistent session should fail (FK constraint)
    let result = store.append_message("nonexistent", &msg);
    assert!(result.is_err());
}

#[test]
fn test_schema_has_required_columns() {
    for col in &[
        "id",
        "created_at",
        "updated_at",
        "title",
        "working_directory",
        "parent_id",
        "channel",
        "time_archived",
        "metadata_json",
    ] {
        assert!(CREATE_SESSIONS_TABLE.contains(col), "sessions schema missing column: {}", col);
    }

    for col in &[
        "session_id",
        "seq",
        "role",
        "content",
        "timestamp",
        "metadata_json",
        "tool_calls_json",
        "tokens",
        "thinking_trace",
        "reasoning_content",
    ] {
        assert!(CREATE_MESSAGES_TABLE.contains(col), "messages schema missing column: {}", col);
    }
}
