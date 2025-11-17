use serde::{Deserialize, Serialize};

#[expect(dead_code)]
pub struct LoginResponse {
    pub token: String,
}

pub enum TokenPollResponse {
    Pending,
    Success(String),
    #[expect(dead_code)]
    Failure(String),
}
#[derive(Serialize, Deserialize)]
pub struct CreateNoteResponse {
    pub id: i64,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetNotesResponse {
    pub notes: Vec<Note>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub content: String,
    pub tags: Vec<String>,
    pub date: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
}

impl From<jot_core::Note> for Note {
    fn from(note: jot_core::Note) -> Self {
        Note {
            id: note.id,
            content: note.content,
            tags: note.tags,
            date: note.date,
            created_at: note.created_at,
            updated_at: note.updated_at,
            deleted_at: note.deleted_at,
        }
    }
}

#[derive(Serialize)]
pub struct PreviewNote {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub content: String,
}

#[derive(Serialize)]
pub struct DeviceCodeRequest {
    pub device_code: String,
}

#[derive(Serialize, Deserialize)]
pub struct Token {
    pub access_token: String,
}
