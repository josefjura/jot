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
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create database directory at {:?}", parent))?;
        }

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
        jot_core::create_note(&self.conn, &content, tags, date).context("Failed to create note")
    }

    /// Search for notes
    pub fn search_notes(&self, query: &SearchQuery) -> Result<Vec<Note>> {
        jot_core::search_notes(&self.conn, query).context("Failed to search notes")
    }

    /// Get a note by ID (supports partial IDs - finds notes starting with the given prefix)
    pub fn get_note_by_id(&self, id: &str) -> Result<Option<Note>> {
        // First try exact match
        if let Some(note) =
            jot_core::get_note_by_id(&self.conn, id).context("Failed to get note by ID")?
        {
            return Ok(Some(note));
        }

        // If not found, try partial match (ID starts with the given prefix)
        let query = SearchQuery {
            text: None,
            tags: vec![],
            date_from: None,
            date_to: None,
            include_deleted: false,
            limit: None,
        };
        let all_notes =
            jot_core::search_notes(&self.conn, &query).context("Failed to search notes")?;

        let matches: Vec<Note> = all_notes
            .into_iter()
            .filter(|note| note.id.starts_with(id))
            .collect();

        match matches.len() {
            0 => Ok(None),
            1 => Ok(matches.into_iter().next()),
            _ => Err(anyhow::anyhow!(
                "Ambiguous ID '{}': matches {} notes. Please provide more characters.",
                id,
                matches.len()
            )),
        }
    }

    /// Update an existing note
    pub fn update_note(
        &self,
        id: &str,
        content: String,
        tags: Vec<String>,
        date: Option<String>,
    ) -> Result<()> {
        jot_core::update_note(&self.conn, id, &content, tags, date).context("Failed to update note")
    }

    /// Soft delete a note
    pub fn soft_delete_note(&self, id: &str) -> Result<()> {
        jot_core::soft_delete_note(&self.conn, id).context("Failed to soft delete note")
    }

    /// Get all notes modified since a timestamp (for sync)
    #[allow(dead_code)]
    pub fn get_notes_since(&self, timestamp: i64) -> Result<Vec<Note>> {
        jot_core::get_notes_since(&self.conn, timestamp)
            .context("Failed to get notes since timestamp")
    }

    /// Update or insert a note (for sync)
    #[allow(dead_code)]
    pub fn upsert_note(&self, note: &Note) -> Result<()> {
        jot_core::upsert_note(&self.conn, note).context("Failed to upsert note")
    }

    /// Get the last sync timestamp
    #[allow(dead_code)]
    pub fn get_last_sync(&self) -> Result<i64> {
        match jot_core::get_sync_state(&self.conn, "last_sync") {
            Ok(Some(s)) => Ok(s.parse::<i64>().unwrap_or(0)),
            Ok(None) => Ok(0),
            Err(e) => Err(anyhow::Error::new(e).context("Failed to get last sync timestamp")),
        }
    }

    /// Set the last sync timestamp
    #[allow(dead_code)]
    pub fn set_last_sync(&self, timestamp: i64) -> Result<()> {
        jot_core::set_sync_state(&self.conn, "last_sync", &timestamp.to_string())
            .context("Failed to set last sync timestamp")
    }
}
