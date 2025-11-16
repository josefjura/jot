/// SQL schema for notes database (used by both CLI and server per-user DBs)
pub const SCHEMA_V1: &str = r#"
CREATE TABLE IF NOT EXISTS notes (
    id TEXT PRIMARY KEY NOT NULL,
    content TEXT NOT NULL,
    tags TEXT NOT NULL,
    date TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    deleted_at INTEGER
);

CREATE INDEX IF NOT EXISTS idx_updated_at ON notes(updated_at);
CREATE INDEX IF NOT EXISTS idx_deleted_at ON notes(deleted_at);
CREATE INDEX IF NOT EXISTS idx_date ON notes(date);

CREATE TABLE IF NOT EXISTS sync_state (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

PRAGMA user_version = 1;
"#;

/// Get current schema version from database
pub fn get_schema_version(conn: &rusqlite::Connection) -> Result<i32, rusqlite::Error> {
    conn.pragma_query_value(None, "user_version", |row| row.get(0))
}

/// Set schema version in database
pub fn set_schema_version(conn: &rusqlite::Connection, version: i32) -> Result<(), rusqlite::Error> {
    conn.pragma_update(None, "user_version", version)
}

/// Run migrations to bring database to current schema version
pub fn migrate(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    let version = get_schema_version(conn)?;

    match version {
        0 => {
            // Fresh database - apply v1 schema
            conn.execute_batch(SCHEMA_V1)?;
            Ok(())
        }
        1 => {
            // Already up to date
            Ok(())
        }
        _ => {
            Err(rusqlite::Error::InvalidQuery)
        }
    }
}
