use aide::{
    axum::{routing::post_with, ApiRouter, IntoApiResponse},
    transform::TransformOperation,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use serde::{Deserialize, Serialize};

use crate::{
    errors::{RestError, RestResult},
    model::user::User,
    state::AppState,
};

/// Sync request from client
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SyncRequestDto {
    pub notes: Vec<NoteDto>,
    pub last_sync: i64,
}

/// Sync response to client
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SyncResponseDto {
    pub notes: Vec<NoteDto>,
}

/// Note DTO for API
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct NoteDto {
    pub id: String,
    pub content: String,
    pub tags: Vec<String>,
    pub date: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
}

impl From<jot_core::Note> for NoteDto {
    fn from(note: jot_core::Note) -> Self {
        NoteDto {
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

impl From<NoteDto> for jot_core::Note {
    fn from(dto: NoteDto) -> Self {
        jot_core::Note {
            id: dto.id,
            content: dto.content,
            tags: dto.tags,
            date: dto.date,
            created_at: dto.created_at,
            updated_at: dto.updated_at,
            deleted_at: dto.deleted_at,
        }
    }
}

/// Sync notes endpoint - implements incremental sync protocol
async fn sync_notes(
    State(state): State<AppState>,
    user_opt: Option<Extension<User>>,
    Json(request): Json<SyncRequestDto>,
) -> impl IntoApiResponse {
    // Check authentication
    let user = match user_opt {
        Some(Extension(user)) => user,
        None => return RestError::Authorization(crate::errors::AuthError::TokenNotFound).into_response(),
    };

    let result = perform_sync(&state, &user, request).await;

    match result {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn perform_sync(
    state: &AppState,
    user: &User,
    request: SyncRequestDto,
) -> RestResult<SyncResponseDto> {
    // Get user's database path
    let user_db_path = state.user_db_path(&user.id.to_string());

    // Open user's database
    let conn = jot_core::open_db(&user_db_path).map_err(|e| {
        RestError::Internal(format!("Failed to open user database: {}", e))
    })?;

    // Convert DTOs to core Note types
    let client_notes: Vec<jot_core::Note> = request.notes.into_iter().map(|n| n.into()).collect();

    // Process sync using core library
    let sync_request = jot_core::SyncRequest {
        notes: client_notes,
        last_sync: request.last_sync,
    };

    let sync_response = jot_core::process_sync_request(&conn, sync_request).map_err(|e| {
        RestError::Internal(format!("Failed to process sync: {}", e))
    })?;

    // Convert back to DTOs
    let response_notes: Vec<NoteDto> = sync_response.notes.into_iter().map(|n| n.into()).collect();

    Ok(SyncResponseDto {
        notes: response_notes,
    })
}

fn sync_notes_docs(op: TransformOperation) -> TransformOperation {
    op.description("Sync notes with server")
        .tag("sync")
        .response_with::<200, Json<SyncResponseDto>, _>(|res| {
            res.example(SyncResponseDto {
                notes: vec![],
            })
        })
}

pub fn sync_routes(_app_state: AppState) -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/sync", post_with(sync_notes, sync_notes_docs))
}
