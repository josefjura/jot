#![deny(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

pub mod db;
pub mod models;
pub mod schema;
pub mod sync;

// Re-export commonly used types
pub use models::{Note, SearchQuery, SyncRequest, SyncResponse};
pub use db::{
    open_db, create_note, get_note_by_id, search_notes, update_note,
    soft_delete_note, get_notes_since, upsert_note, get_sync_state, set_sync_state,
};
pub use sync::{merge_notes, process_sync_request};
