use serde::{Deserialize, Serialize};

/// A note with all metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Note {
    /// ULID (sortable, globally unique)
    pub id: String,
    /// Note content (plain text/markdown)
    pub content: String,
    /// Tags as array
    pub tags: Vec<String>,
    /// Optional date in ISO 8601 format (YYYY-MM-DD)
    pub date: Option<String>,
    /// Unix timestamp in milliseconds
    pub created_at: i64,
    /// Unix timestamp in milliseconds
    pub updated_at: i64,
    /// Unix timestamp in milliseconds (None = active, Some = deleted)
    pub deleted_at: Option<i64>,
}

/// Search query parameters
#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    /// Full-text search term
    pub text: Option<String>,
    /// Filter by tags (must have all specified tags)
    pub tags: Vec<String>,
    /// Filter by date range (inclusive start)
    pub date_from: Option<String>,
    /// Filter by date range (inclusive end)
    pub date_to: Option<String>,
    /// Include soft-deleted notes
    pub include_deleted: bool,
    /// Limit number of results
    pub limit: Option<usize>,
}

/// Sync request from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRequest {
    /// Notes changed on client since last sync
    pub notes: Vec<Note>,
    /// Client's last sync timestamp (milliseconds)
    pub last_sync: i64,
}

/// Sync response from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResponse {
    /// Notes from server that client needs
    pub notes: Vec<Note>,
}

/// Conflict information (for future use)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub note_id: String,
    pub client_version: Note,
    pub server_version: Note,
}
