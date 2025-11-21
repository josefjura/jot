use crate::db::{get_note_by_id, get_notes_since, upsert_note};
use crate::models::{Note, SyncRequest, SyncResponse};
use rusqlite::{Connection, Result};

/// Merge notes from client into server database
/// Returns notes that client needs to update
pub fn merge_notes(
    conn: &Connection,
    client_notes: Vec<Note>,
    client_last_sync: i64,
) -> Result<Vec<Note>> {
    let mut notes_to_send = Vec::new();
    let mut client_note_ids: Vec<String> = Vec::new();

    // Process each incoming note from client
    for client_note in client_notes {
        client_note_ids.push(client_note.id.clone());
        let server_note = get_note_by_id(conn, &client_note.id)?;

        match server_note {
            None => {
                // New note from client - insert it
                upsert_note(conn, &client_note)?;
            }
            Some(server_note) => {
                // Conflict resolution: Last-Write-Wins
                if client_note.updated_at > server_note.updated_at {
                    // Client version is newer
                    upsert_note(conn, &client_note)?;
                } else if server_note.updated_at > client_note.updated_at {
                    // Server version is newer - send to client
                    notes_to_send.push(server_note);
                }
                // If timestamps equal, no action needed
            }
        }
    }

    // Get all notes from server that are newer than client's last sync
    let server_new_notes = get_notes_since(conn, client_last_sync)?;

    // Filter out notes that client just sent to us and notes we've already decided to send
    for note in server_new_notes {
        if !client_note_ids.contains(&note.id) && !notes_to_send.iter().any(|n| n.id == note.id) {
            notes_to_send.push(note);
        }
    }

    Ok(notes_to_send)
}

/// Process sync request (server-side logic)
pub fn process_sync_request(conn: &Connection, request: SyncRequest) -> Result<SyncResponse> {
    let notes = merge_notes(conn, request.notes, request.last_sync)?;
    Ok(SyncResponse { notes })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::db::{create_note, open_db};
    use std::thread;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn test_merge_new_note_from_client() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = open_db(&db_path).unwrap();

        let client_note = Note {
            id: "01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string(),
            content: "client note".to_string(),
            tags: vec![],
            subject_date: None,
            created_at: 1000,
            updated_at: 1000,
            deleted_at: None,
        };

        let result = merge_notes(&conn, vec![client_note.clone()], 0).unwrap();

        // Should return empty since server has no newer notes
        assert_eq!(result.len(), 0);

        // Verify note was inserted
        let note = get_note_by_id(&conn, &client_note.id).unwrap().unwrap();

        assert_eq!(note.content, "client note");
    }

    #[test]
    fn test_merge_conflict_last_write_wins() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = open_db(&db_path).unwrap();

        // Create server note
        let note = create_note(&conn, "server version", vec![], None).unwrap();

        thread::sleep(Duration::from_millis(10));

        // Client sends newer version
        let client_note = Note {
            id: note.id.clone(),
            content: "client version (newer)".to_string(),
            tags: vec![],
            subject_date: None,
            created_at: note.created_at,
            updated_at: chrono::Utc::now().timestamp_millis(),
            deleted_at: None,
        };

        let result = merge_notes(&conn, vec![client_note.clone()], 0).unwrap();

        // Server should not send anything back (client version wins)
        assert_eq!(result.len(), 0);

        // Verify client version was saved
        let updated = get_note_by_id(&conn, &note.id).unwrap().unwrap();

        assert_eq!(updated.content, "client version (newer)");
    }
}
