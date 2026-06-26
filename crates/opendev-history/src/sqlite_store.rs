use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use opendev_models::{ChatMessage, Role, Session};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Executor, Row, SqlitePool};
use tokio::runtime::Runtime;

fn block_on<F: std::future::Future + Send>(f: F) -> F::Output
where
    F::Output: Send + 'static,
{
    static RT: OnceLock<Runtime> = OnceLock::new();
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => std::thread::scope(|s| {
            let (tx, rx) = std::sync::mpsc::sync_channel(1);
            s.spawn(move || {
                let result = handle.block_on(f);
                let _ = tx.send(result);
            });
            rx.recv().expect("block_on thread panic")
        }),
        Err(_) => {
            let rt = RT.get_or_init(|| Runtime::new().expect("Failed to create tokio runtime"));
            rt.block_on(f)
        }
    }
}

pub const CREATE_SESSIONS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS sessions (
    id              TEXT PRIMARY KEY,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    title           TEXT,
    working_directory TEXT,
    parent_id       TEXT,
    channel         TEXT NOT NULL DEFAULT 'cli',
    channel_user_id TEXT NOT NULL DEFAULT '',
    time_archived   TEXT,
    metadata_json   TEXT NOT NULL DEFAULT '{}'
)
"#;

pub const CREATE_MESSAGES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS messages (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id      TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    seq             INTEGER NOT NULL,
    role            TEXT NOT NULL,
    content         TEXT NOT NULL,
    timestamp       TEXT NOT NULL,
    metadata_json   TEXT NOT NULL DEFAULT '{}',
    tool_calls_json TEXT NOT NULL DEFAULT '[]',
    tokens          INTEGER,
    thinking_trace  TEXT,
    reasoning_content TEXT,
    UNIQUE(session_id, seq)
)
"#;

pub const CREATE_COST_RECORDS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS cost_records (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id      TEXT NOT NULL DEFAULT '',
    model           TEXT NOT NULL,
    provider        TEXT NOT NULL,
    prompt_tokens   INTEGER NOT NULL DEFAULT 0,
    completion_tokens INTEGER NOT NULL DEFAULT 0,
    cache_read_tokens  INTEGER NOT NULL DEFAULT 0,
    cache_write_tokens INTEGER NOT NULL DEFAULT 0,
    thinking_tokens INTEGER,
    cost_rmb        REAL NOT NULL DEFAULT 0.0,
    created_at      TEXT NOT NULL
)
"#;

pub const CREATE_INDEXES: &[&str] = &[
    "CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id)",
    "CREATE INDEX IF NOT EXISTS idx_sessions_updated ON sessions(updated_at DESC)",
    "CREATE INDEX IF NOT EXISTS idx_sessions_channel ON sessions(channel)",
    "CREATE INDEX IF NOT EXISTS idx_cost_records_session ON cost_records(session_id)",
    "CREATE INDEX IF NOT EXISTS idx_cost_records_created ON cost_records(created_at DESC)",
];

#[derive(Debug)]
pub struct SqliteSessionStore {
    db_path: PathBuf,
    pool: SqlitePool,
}

impl SqliteSessionStore {
    pub fn open(db_path: impl AsRef<Path>) -> Result<Self, String> {
        let db_path = db_path.as_ref().to_path_buf();

        let options = SqliteConnectOptions::new()
            .filename(&db_path)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .foreign_keys(true)
            .create_if_missing(true);

        let pool = block_on(async {
            let pool = SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(options)
                .await
                .map_err(|e| format!("Failed to open SQLite database: {}", e))?;

            pool.execute(CREATE_SESSIONS_TABLE)
                .await
                .map_err(|e| format!("Failed to create sessions table: {}", e))?;
            pool.execute(CREATE_MESSAGES_TABLE)
                .await
                .map_err(|e| format!("Failed to create messages table: {}", e))?;
            pool.execute(CREATE_COST_RECORDS_TABLE)
                .await
                .map_err(|e| format!("Failed to create cost_records table: {}", e))?;
            for idx_sql in CREATE_INDEXES {
                pool.execute(*idx_sql)
                    .await
                    .map_err(|e| format!("Failed to create index: {}", e))?;
            }

            Ok::<_, String>(pool)
        })?;

        Ok(Self { db_path, pool })
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    pub fn save_session(&self, session: &Session) -> Result<(), String> {
        let title = session.metadata.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let time_archived = session.time_archived.map(|t| t.to_rfc3339());

        let extra = session_extra_json(session);

        block_on(async {
            let mut tx =
                self.pool.begin().await.map_err(|e| format!("Transaction start failed: {}", e))?;

            sqlx::query(
                "INSERT OR REPLACE INTO sessions \
                 (id, created_at, updated_at, title, working_directory, \
                  parent_id, channel, channel_user_id, time_archived, metadata_json) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            )
            .bind(&session.id)
            .bind(session.created_at.to_rfc3339())
            .bind(session.updated_at.to_rfc3339())
            .bind(title)
            .bind(&session.working_directory)
            .bind(&session.parent_id)
            .bind(&session.channel)
            .bind(&session.channel_user_id)
            .bind(&time_archived)
            .bind(&extra)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("Failed to save session: {}", e))?;

            sqlx::query("DELETE FROM messages WHERE session_id = ?1")
                .bind(&session.id)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("Failed to clear messages: {}", e))?;

            for (seq, msg) in session.messages.iter().enumerate() {
                insert_message(&mut *tx, &session.id, seq as i64, msg)
                    .await
                    .map_err(|e| format!("Failed to insert message: {}", e))?;
            }

            tx.commit().await.map_err(|e| format!("Commit failed: {}", e))?;

            Ok::<_, String>(())
        })
    }

    pub fn load_session(&self, session_id: &str) -> Result<Session, String> {
        block_on(async {
            let row = sqlx::query("SELECT * FROM sessions WHERE id = ?1")
                .bind(session_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| format!("Failed to load session: {}", e))?
                .ok_or_else(|| format!("Session not found: {}", session_id))?;

            let mut session = row_to_session(&row)?;

            let msg_rows =
                sqlx::query("SELECT * FROM messages WHERE session_id = ?1 ORDER BY seq ASC")
                    .bind(session_id)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|e| format!("Failed to load messages: {}", e))?;

            for msg_row in &msg_rows {
                session.messages.push(row_to_message(msg_row)?);
            }

            Ok(session)
        })
    }

    pub fn delete_session(&self, session_id: &str) -> Result<(), String> {
        block_on(async {
            sqlx::query("DELETE FROM sessions WHERE id = ?1")
                .bind(session_id)
                .execute(&self.pool)
                .await
                .map_err(|e| format!("Failed to delete session: {}", e))?;
            Ok(())
        })
    }

    pub fn list_session_ids(&self) -> Result<Vec<String>, String> {
        block_on(async {
            let rows = sqlx::query("SELECT id FROM sessions ORDER BY updated_at DESC")
                .fetch_all(&self.pool)
                .await
                .map_err(|e| format!("Failed to list sessions: {}", e))?;
            Ok(rows.iter().map(|r| r.get::<String, _>(0)).collect())
        })
    }

    pub fn search_messages(&self, query: &str) -> Result<Vec<(String, Vec<usize>)>, String> {
        let pattern = format!("%{}%", escape_like(query));
        block_on(async {
            let rows = sqlx::query(
                "SELECT session_id, seq FROM messages WHERE content LIKE ?1 ESCAPE '\\' ORDER BY session_id, seq",
            )
            .bind(&pattern)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Failed to search messages: {}", e))?;

            let mut results: Vec<(String, Vec<usize>)> = Vec::new();
            for row in &rows {
                let session_id: String = row.get("session_id");
                let seq: i64 = row.get("seq");
                if let Some(last) = results.last_mut()
                    && last.0 == session_id
                {
                    last.1.push(seq as usize);
                    continue;
                }
                results.push((session_id, vec![seq as usize]));
            }
            Ok(results)
        })
    }

    /// Record a cost entry (synchronous convenience).
    #[allow(clippy::too_many_arguments)]
    pub fn record_cost(
        &self,
        session_id: &str,
        model: &str,
        provider: &str,
        prompt_tokens: u64,
        completion_tokens: u64,
        cache_read_tokens: u64,
        cache_write_tokens: u64,
        cost_rmb: f64,
    ) -> Result<(), String> {
        let pool = self.pool.clone();
        let session_id = session_id.to_string();
        let model = model.to_string();
        let provider = provider.to_string();
        let now = chrono::Utc::now().to_rfc3339();
        block_on(async {
            sqlx::query(
                "INSERT INTO cost_records \
                 (session_id, model, provider, prompt_tokens, completion_tokens, \
                  cache_read_tokens, cache_write_tokens, thinking_tokens, cost_rmb, created_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            )
            .bind(&session_id)
            .bind(&model)
            .bind(&provider)
            .bind(prompt_tokens as i64)
            .bind(completion_tokens as i64)
            .bind(cache_read_tokens as i64)
            .bind(cache_write_tokens as i64)
            .bind(None::<i64>) // thinking_tokens
            .bind(cost_rmb)
            .bind(&now)
            .execute(&pool)
            .await
            .map_err(|e| format!("failed to record cost: {}", e))?;
            Ok(())
        })
    }

    pub fn append_message(&self, session_id: &str, message: &ChatMessage) -> Result<(), String> {
        block_on(async {
            let seq: i64 = sqlx::query_scalar::<_, Option<i64>>(
                "SELECT COALESCE(MAX(seq), -1) + 1 FROM messages WHERE session_id = ?1",
            )
            .bind(session_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("Failed to get next seq: {}", e))?
            .unwrap_or(0);

            insert_message(&self.pool, session_id, seq, message)
                .await
                .map_err(|e| format!("Failed to append message: {}", e))?;

            Ok(())
        })
    }
}

fn session_extra_json(session: &Session) -> String {
    let mut extra = serde_json::Map::new();
    extra.insert("chat_type".to_string(), serde_json::Value::String(session.chat_type.clone()));
    if let Some(ref thread_id) = session.thread_id {
        extra.insert("thread_id".to_string(), serde_json::Value::String(thread_id.clone()));
    }
    if let Some(ref last_activity) = session.last_activity {
        extra.insert(
            "last_activity".to_string(),
            serde_json::Value::String(last_activity.to_rfc3339()),
        );
    }
    extra.insert(
        "workspace_confirmed".to_string(),
        serde_json::Value::Bool(session.workspace_confirmed),
    );
    if let Some(ref owner_id) = session.owner_id {
        extra.insert("owner_id".to_string(), serde_json::Value::String(owner_id.clone()));
    }
    if !session.subagent_sessions.is_empty() {
        extra.insert(
            "subagent_sessions".to_string(),
            serde_json::to_value(&session.subagent_sessions).unwrap_or_default(),
        );
    }
    if let Some(ref slug) = session.slug {
        extra.insert("slug".to_string(), serde_json::Value::String(slug.clone()));
    }
    if !session.context_files.is_empty() {
        extra.insert(
            "context_files".to_string(),
            serde_json::to_value(&session.context_files).unwrap_or_default(),
        );
    }
    if !session.file_changes.is_empty() {
        extra.insert(
            "file_changes".to_string(),
            serde_json::to_value(&session.file_changes).unwrap_or_default(),
        );
    }
    if !session.delivery_context.is_empty() {
        extra.insert(
            "delivery_context".to_string(),
            serde_json::to_value(&session.delivery_context).unwrap_or_default(),
        );
    }
    if !session.metadata.is_empty() {
        extra.insert(
            "metadata".to_string(),
            serde_json::to_value(&session.metadata).unwrap_or_default(),
        );
    }
    serde_json::Value::Object(extra).to_string()
}

fn row_to_session(row: &sqlx::sqlite::SqliteRow) -> Result<Session, String> {
    let id: String = row.get("id");
    let created_at: String = row.get("created_at");
    let updated_at: String = row.get("updated_at");
    let working_directory: Option<String> = row.get("working_directory");
    let parent_id: Option<String> = row.get("parent_id");
    let channel: String = row.get("channel");
    let channel_user_id: String = row.get("channel_user_id");
    let time_archived: Option<String> = row.get("time_archived");
    let metadata_json: String = row.get("metadata_json");

    let created_at = chrono::DateTime::parse_from_rfc3339(&created_at)
        .map_err(|e| format!("Invalid created_at: {}", e))?
        .with_timezone(&chrono::Utc);
    let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at)
        .map_err(|e| format!("Invalid updated_at: {}", e))?
        .with_timezone(&chrono::Utc);

    let extra: serde_json::Value = serde_json::from_str(&metadata_json).unwrap_or_default();
    let extra_obj = extra.as_object().cloned().unwrap_or_default();

    let chat_type =
        extra_obj.get("chat_type").and_then(|v| v.as_str()).unwrap_or("direct").to_string();
    let thread_id = extra_obj.get("thread_id").and_then(|v| v.as_str()).map(String::from);
    let last_activity = extra_obj
        .get("last_activity")
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));
    let workspace_confirmed =
        extra_obj.get("workspace_confirmed").and_then(|v| v.as_bool()).unwrap_or(false);
    let owner_id = extra_obj.get("owner_id").and_then(|v| v.as_str()).map(String::from);
    let slug = extra_obj.get("slug").and_then(|v| v.as_str()).map(String::from);
    let subagent_sessions = extra_obj
        .get("subagent_sessions")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    let context_files = extra_obj
        .get("context_files")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    let file_changes = extra_obj
        .get("file_changes")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    let delivery_context = extra_obj
        .get("delivery_context")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    let metadata = extra_obj
        .get("metadata")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    let time_archived = time_archived
        .as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

    Ok(Session {
        id,
        created_at,
        updated_at,
        messages: Vec::new(),
        context_files,
        working_directory,
        metadata,
        file_changes,
        channel,
        chat_type,
        channel_user_id,
        thread_id,
        delivery_context,
        last_activity,
        workspace_confirmed,
        owner_id,
        parent_id,
        subagent_sessions,
        time_archived,
        slug,
    })
}

fn msg_extra_json(msg: &ChatMessage) -> String {
    let mut extra = serde_json::Map::new();
    if let Some(ref provenance) = msg.provenance {
        extra
            .insert("provenance".to_string(), serde_json::to_value(provenance).unwrap_or_default());
    }
    if let Some(ref token_usage) = msg.token_usage {
        extra.insert(
            "token_usage".to_string(),
            serde_json::to_value(token_usage).unwrap_or_default(),
        );
    }
    serde_json::Value::Object(extra).to_string()
}

fn row_to_message(row: &sqlx::sqlite::SqliteRow) -> Result<ChatMessage, String> {
    let role: String = row.get("role");
    let content: String = row.get("content");
    let timestamp: String = row.get("timestamp");
    let metadata_json: String = row.get("metadata_json");
    let tool_calls_json: String = row.get("tool_calls_json");
    let tokens: Option<i64> = row.get("tokens");
    let thinking_trace: Option<String> = row.get("thinking_trace");
    let reasoning_content: Option<String> = row.get("reasoning_content");

    let role = match role.to_lowercase().as_str() {
        "user" => Role::User,
        "assistant" => Role::Assistant,
        "system" => Role::System,
        _ => return Err(format!("Unknown role: {}", role)),
    };

    let timestamp = chrono::DateTime::parse_from_rfc3339(&timestamp)
        .map_err(|e| format!("Invalid timestamp: {}", e))?
        .with_timezone(&chrono::Utc);

    let metadata: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_str(&metadata_json).unwrap_or_default();
    let tool_calls: Vec<opendev_models::ToolCall> =
        serde_json::from_str(&tool_calls_json).unwrap_or_default();

    let extra: serde_json::Value = serde_json::from_str(&metadata_json).unwrap_or_default();
    let extra_obj = extra.as_object().cloned().unwrap_or_default();

    let provenance =
        extra_obj.get("provenance").and_then(|v| serde_json::from_value(v.clone()).ok());
    let token_usage =
        extra_obj.get("token_usage").and_then(|v| serde_json::from_value(v.clone()).ok());

    Ok(ChatMessage {
        role,
        content,
        timestamp,
        metadata,
        tool_calls,
        tokens: tokens.map(|t| t as u64),
        thinking_trace,
        reasoning_content,
        token_usage,
        provenance,
    })
}

async fn insert_message(
    executor: impl Executor<'_, Database = sqlx::Sqlite>,
    session_id: &str,
    seq: i64,
    msg: &ChatMessage,
) -> Result<(), String> {
    let extra = msg_extra_json(msg);
    let tool_calls_json =
        serde_json::to_string(&msg.tool_calls).unwrap_or_else(|_| "[]".to_string());

    sqlx::query(
        "INSERT INTO messages \
         (session_id, seq, role, content, timestamp, metadata_json, tool_calls_json, \
          tokens, thinking_trace, reasoning_content) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
    )
    .bind(session_id)
    .bind(seq)
    .bind(msg.role.to_string())
    .bind(&msg.content)
    .bind(msg.timestamp.to_rfc3339())
    .bind(&extra)
    .bind(&tool_calls_json)
    .bind(msg.tokens.map(|t| t as i64))
    .bind(&msg.thinking_trace)
    .bind(&msg.reasoning_content)
    .execute(executor)
    .await
    .map_err(|e| format!("Failed to insert message: {}", e))?;

    Ok(())
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
#[path = "sqlite_store_tests.rs"]
mod tests;
