#![deny(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

pub mod db;
pub mod models;
pub mod schema;
pub mod sync;

// Re-export commonly used types
pub use db::{
    create_note, get_note_by_id, get_notes_since, get_sync_state, open_db, search_notes,
    set_sync_state, soft_delete_note, update_note, upsert_note,
};
pub use models::{Note, SearchQuery, SyncRequest, SyncResponse};
pub use sync::{merge_notes, process_sync_request};
