use aide::{axum::ApiRouter, openapi::OpenApi};
use auth::auth_routes;
use axum::{Extension, Router};
use health::health_routes;
use openapi::{api_docs, docs_routes};
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tower_sessions::{MemoryStore, SessionManagerLayer};

use crate::state::AppState;

pub mod auth;
pub mod health;
pub mod openapi;
pub mod sync;

pub fn setup_router(auth_db: Connection, jwt_secret: &str, data_dir: PathBuf) -> Router {
    aide::gen::on_error(|error| {
        println!("{error}");
    });

    aide::gen::extract_schemas(true);
    let mut api = OpenApi::default();

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store);
    let app_state = AppState::new(auth_db, jwt_secret, data_dir);
    aide::gen::infer_responses(true);

    aide::gen::infer_responses(false);

    // Note: Authentication is handled at the endpoint level by checking for Extension<User>
    // No global auth middleware is applied - public endpoints simply don't require the extension
    ApiRouter::new()
        .merge(health_routes(app_state.clone()))
        .merge(auth_routes(app_state.clone()))
        .merge(sync::sync_routes(app_state.clone()))
        .merge(docs_routes())
        .finish_api_with(&mut api, api_docs)
        .layer(Extension(Arc::new(api)))
        .layer(session_layer)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state)
}
