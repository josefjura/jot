use crate::models::{Note, SearchQuery};
use crate::schema;
use rusqlite::{params, Connection, Result};
use std::path::Path;

/// Open or create a notes database at the specified path
pub fn open_db(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    schema::migrate(&conn)?;
    Ok(conn)
}

/// Create a new note
pub fn create_note(
    conn: &Connection,
    content: &str,
    tags: Vec<String>,
    date: Option<String>,
) -> Result<Note> {
    let id = ulid::Ulid::new().to_string();
    let now = chrono::Utc::now().timestamp_millis();
    let tags_json = serde_json::to_string(&tags)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

    conn.execute(
        "INSERT INTO notes (id, content, tags, date, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, content, tags_json, date, now, now],
    )?;

    Ok(Note {
        id,
        content: content.to_string(),
        tags,
        date,
        created_at: now,
        updated_at: now,
        deleted_at: None,
    })
}

/// Get a note by ID
pub fn get_note_by_id(conn: &Connection, id: &str) -> Result<Option<Note>> {
    let mut stmt = conn.prepare(
        "SELECT id, content, tags, date, created_at, updated_at, deleted_at FROM notes WHERE id = ?1"
    )?;

    let note = stmt.query_row(params![id], |row| {
        let tags_json: String = row.get(2)?;
        let tags: Vec<String> = serde_json::from_str(&tags_json).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(e))
        })?;

        Ok(Note {
            id: row.get(0)?,
            content: row.get(1)?,
            tags,
            date: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
            deleted_at: row.get(6)?,
        })
    });

    match note {
        Ok(n) => Ok(Some(n)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Search notes with various filters
pub fn search_notes(conn: &Connection, query: &SearchQuery) -> Result<Vec<Note>> {
    let mut sql = String::from(
        "SELECT id, content, tags, date, created_at, updated_at, deleted_at FROM notes WHERE 1=1",
    );
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    // Filter by deleted status
    if !query.include_deleted {
        sql.push_str(" AND deleted_at IS NULL");
    }

    // Full-text search
    if let Some(ref text) = query.text {
        sql.push_str(" AND content LIKE ?");
        params.push(Box::new(format!("%{}%", text)));
    }

    // Date range filters
    if let Some(ref date_from) = query.date_from {
        sql.push_str(" AND date >= ?");
        params.push(Box::new(date_from.clone()));
    }

    if let Some(ref date_to) = query.date_to {
        sql.push_str(" AND date <= ?");
        params.push(Box::new(date_to.clone()));
    }

    // Tag filters
    for tag in &query.tags {
        sql.push_str(" AND tags LIKE ?");
        params.push(Box::new(format!("%\"{}%", tag)));
    }

    // Order by updated_at descending (most recent first)
    sql.push_str(" ORDER BY updated_at DESC");

    // Limit
    if let Some(limit) = query.limit {
        sql.push_str(" LIMIT ?");
        params.push(Box::new(limit as i64));
    }

    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_refs.as_slice(), |row| {
        let tags_json: String = row.get(2)?;
        let tags: Vec<String> = serde_json::from_str(&tags_json).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(e))
        })?;

        Ok(Note {
            id: row.get(0)?,
            content: row.get(1)?,
            tags,
            date: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
            deleted_at: row.get(6)?,
        })
    })?;

    let mut notes = Vec::new();
    for note in rows {
        notes.push(note?);
    }

    Ok(notes)
}

/// Update note content and/or tags
pub fn update_note(
    conn: &Connection,
    id: &str,
    content: &str,
    tags: Vec<String>,
    date: Option<String>,
) -> Result<()> {
    let now = chrono::Utc::now().timestamp_millis();
    let tags_json = serde_json::to_string(&tags)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

    conn.execute(
        "UPDATE notes SET content = ?1, tags = ?2, date = ?3, updated_at = ?4 WHERE id = ?5",
        params![content, tags_json, date, now, id],
    )?;

    Ok(())
}

/// Soft delete a note
pub fn soft_delete_note(conn: &Connection, id: &str) -> Result<()> {
    let now = chrono::Utc::now().timestamp_millis();

    conn.execute(
        "UPDATE notes SET deleted_at = ?1, updated_at = ?2 WHERE id = ?3",
        params![now, now, id],
    )?;

    Ok(())
}

/// Get all notes updated since a specific timestamp (for sync)
pub fn get_notes_since(conn: &Connection, timestamp: i64) -> Result<Vec<Note>> {
    let mut stmt = conn.prepare(
        "SELECT id, content, tags, date, created_at, updated_at, deleted_at
         FROM notes
         WHERE updated_at > ?1
         ORDER BY updated_at ASC",
    )?;

    let rows = stmt.query_map(params![timestamp], |row| {
        let tags_json: String = row.get(2)?;
        let tags: Vec<String> = serde_json::from_str(&tags_json).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(e))
        })?;

        Ok(Note {
            id: row.get(0)?,
            content: row.get(1)?,
            tags,
            date: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
            deleted_at: row.get(6)?,
        })
    })?;

    let mut notes = Vec::new();
    for note in rows {
        notes.push(note?);
    }

    Ok(notes)
}

/// Upsert a note (insert or update based on timestamp comparison)
pub fn upsert_note(conn: &Connection, note: &Note) -> Result<()> {
    let tags_json = serde_json::to_string(&note.tags)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

    // Check if note exists
    if let Some(existing) = get_note_by_id(conn, &note.id)? {
        // Only update if incoming note is newer
        if note.updated_at > existing.updated_at {
            conn.execute(
                "UPDATE notes SET content = ?1, tags = ?2, date = ?3, created_at = ?4, updated_at = ?5, deleted_at = ?6 WHERE id = ?7",
                params![note.content, tags_json, note.date, note.created_at, note.updated_at, note.deleted_at, note.id],
            )?;
        }
    } else {
        // Insert new note
        conn.execute(
            "INSERT INTO notes (id, content, tags, date, created_at, updated_at, deleted_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![note.id, note.content, tags_json, note.date, note.created_at, note.updated_at, note.deleted_at],
        )?;
    }

    Ok(())
}

/// Get sync state value
pub fn get_sync_state(conn: &Connection, key: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM sync_state WHERE key = ?1")?;

    let value = stmt.query_row(params![key], |row| row.get(0));

    match value {
        Ok(v) => Ok(Some(v)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Set sync state value
pub fn set_sync_state(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO sync_state (key, value) VALUES (?1, ?2)",
        params![key, value],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_and_get_note() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = open_db(&db_path).unwrap();

        let note = create_note(&conn, "test content", vec!["tag1".to_string()], None).unwrap();

        assert_eq!(note.content, "test content");
        assert_eq!(note.tags, vec!["tag1"]);

        let retrieved = get_note_by_id(&conn, &note.id).unwrap().unwrap();

        assert_eq!(retrieved.id, note.id);
        assert_eq!(retrieved.content, "test content");
    }

    #[test]
    fn test_soft_delete() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = open_db(&db_path).unwrap();

        let note = create_note(&conn, "test", vec![], None).unwrap();

        soft_delete_note(&conn, &note.id).unwrap();

        let deleted = get_note_by_id(&conn, &note.id).unwrap().unwrap();

        assert!(deleted.deleted_at.is_some());
    }

    #[test]
    fn test_search_notes() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = open_db(&db_path).unwrap();

        create_note(&conn, "first note", vec!["work".to_string()], None).unwrap();
        create_note(&conn, "second note", vec!["personal".to_string()], None).unwrap();

        let query = SearchQuery {
            text: Some("first".to_string()),
            ..Default::default()
        };

        let results = search_notes(&conn, &query).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "first note");
    }
}
