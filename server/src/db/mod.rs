use rusqlite::Connection;
use std::path::Path;
use tracing::info;

pub mod auth;

/// Auth database schema
const AUTH_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS device_auth (
    device_code TEXT PRIMARY KEY NOT NULL,
    user_code TEXT UNIQUE NOT NULL,
    user_id TEXT,
    expires_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

PRAGMA user_version = 1;
"#;

/// Open or create auth database
pub fn open_auth_db(path: &Path) -> Result<Connection, rusqlite::Error> {
    info!("Setting up auth database at {:?}", path);
    let conn = Connection::open(path)?;

    // Run migrations
    let version: i32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;

    if version == 0 {
        info!("Initializing auth database schema");
        conn.execute_batch(AUTH_SCHEMA)?;
    }

    info!("Auth database ready");
    Ok(conn)
}
