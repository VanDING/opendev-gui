pub const CREATE_MEMORIES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS long_term_memory (
    id              TEXT PRIMARY KEY,
    content         TEXT NOT NULL,
    category        TEXT NOT NULL,
    confidence      REAL NOT NULL DEFAULT 0.7,
    source          TEXT NOT NULL,
    project_path    TEXT,
    importance      REAL NOT NULL DEFAULT 0.5,
    access_count    INTEGER NOT NULL DEFAULT 0,
    last_accessed_at TEXT,
    expires_at      TEXT,
    created_at      TEXT NOT NULL,
    tags_json       TEXT NOT NULL DEFAULT '[]'
)
"#;

pub const CREATE_SYMBOL_LINKS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS memory_symbol_links (
    memory_id   TEXT NOT NULL,
    symbol_id   TEXT NOT NULL,
    symbol_name TEXT NOT NULL,
    project_path TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    PRIMARY KEY (memory_id, symbol_id),
    FOREIGN KEY (memory_id) REFERENCES long_term_memory(id) ON DELETE CASCADE
)
"#;

pub const MIGRATIONS: &[&str] = &[CREATE_MEMORIES_TABLE, CREATE_SYMBOL_LINKS_TABLE];
