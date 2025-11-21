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

/// Migration from V1 to V2: Rename 'date' column to 'subject_date'
pub const MIGRATION_V1_TO_V2: &str = r#"
-- Rename date column to subject_date
ALTER TABLE notes RENAME COLUMN date TO subject_date;

-- Drop old index
DROP INDEX IF EXISTS idx_date;

-- Create new index
CREATE INDEX IF NOT EXISTS idx_subject_date ON notes(subject_date);

-- Create index on created_at for filtering
CREATE INDEX IF NOT EXISTS idx_created_at ON notes(created_at);

PRAGMA user_version = 2;
"#;

/// Get current schema version from database
pub fn get_schema_version(conn: &rusqlite::Connection) -> Result<i32, rusqlite::Error> {
    conn.pragma_query_value(None, "user_version", |row| row.get(0))
}

/// Set schema version in database
pub fn set_schema_version(
    conn: &rusqlite::Connection,
    version: i32,
) -> Result<(), rusqlite::Error> {
    conn.pragma_update(None, "user_version", version)
}

/// Run migrations to bring database to current schema version
pub fn migrate(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    let mut version = get_schema_version(conn)?;

    // Apply migrations sequentially
    if version == 0 {
        // Fresh database - apply v1 schema
        conn.execute_batch(SCHEMA_V1)?;
        version = 1;
    }

    if version == 1 {
        // Migrate from v1 to v2
        conn.execute_batch(MIGRATION_V1_TO_V2)?;
        version = 2;
    }

    // Version 2 is current
    if version == 2 {
        Ok(())
    } else {
        Err(rusqlite::Error::InvalidQuery)
    }
}
