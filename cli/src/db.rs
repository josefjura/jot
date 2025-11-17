use anyhow::{Context, Result};
use jot_core::{Note, SearchQuery};
use rusqlite::Connection;
use std::path::Path;

/// Local database for offline note storage
pub struct LocalDb {
    conn: Connection,
}

impl LocalDb {
    /// Open or create local database at the given path
    pub fn open(path: &Path) -> Result<Self> {
        let conn = jot_core::open_db(path)
            .with_context(|| format!("Failed to open local database at {:?}", path))?;

        Ok(Self { conn })
    }

    /// Create a new note
    pub fn create_note(
        &self,
        content: String,
        tags: Vec<String>,
        date: Option<String>,
    ) -> Result<Note> {
        jot_core::create_note(&self.conn, &content, tags, date)
            .context("Failed to create note")
    }

    /// Search for notes
    pub fn search_notes(&self, query: &SearchQuery) -> Result<Vec<Note>> {
        jot_core::search_notes(&self.conn, query)
            .context("Failed to search notes")
    }

    /// Get all notes modified since a timestamp (for sync)
    pub fn get_notes_since(&self, timestamp: i64) -> Result<Vec<Note>> {
        jot_core::get_notes_since(&self.conn, timestamp)
            .context("Failed to get notes since timestamp")
    }

    /// Update or insert a note (for sync)
    pub fn upsert_note(&self, note: &Note) -> Result<()> {
        jot_core::upsert_note(&self.conn, note)
            .context("Failed to upsert note")
    }

    /// Get the last sync timestamp
    pub fn get_last_sync(&self) -> Result<i64> {
        match jot_core::get_sync_state(&self.conn, "last_sync") {
            Ok(Some(s)) => Ok(s.parse::<i64>().unwrap_or(0)),
            Ok(None) => Ok(0),
            Err(e) => Err(anyhow::Error::new(e).context("Failed to get last sync timestamp")),
        }
    }

    /// Set the last sync timestamp
    pub fn set_last_sync(&self, timestamp: i64) -> Result<()> {
        jot_core::set_sync_state(&self.conn, "last_sync", &timestamp.to_string())
            .context("Failed to set last sync timestamp")
    }
}
