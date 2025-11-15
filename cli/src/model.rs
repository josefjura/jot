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
    pub id: Option<i64>,
    pub content: String,
    pub tags: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
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
