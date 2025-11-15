use aide::{axum::ApiRouter, openapi::OpenApi};
use auth::auth_routes;
use axum::{middleware, Extension, Router};
use health::health_routes;
use note::note_routes;
use openapi::{api_docs, docs_routes};
use sqlx::SqlitePool;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tower_sessions::{MemoryStore, SessionManagerLayer};

use crate::{middleware::auth_middleware, state::AppState};

pub mod auth;
pub mod health;
pub mod note;
pub mod openapi;

pub fn setup_router(db: SqlitePool, jwt_secret: &str) -> Router {
    aide::gen::on_error(|error| {
        println!("{error}");
    });

    aide::gen::extract_schemas(true);
    let mut api = OpenApi::default();

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store);
    let app_state = AppState::new(db, jwt_secret);
    aide::gen::infer_responses(true);

    let router = ApiRouter::new()
        .merge(health_routes(app_state.clone()))
        .merge(note_routes(app_state.clone()))
        .merge(auth_routes(app_state.clone()))
        .merge(docs_routes())
        .finish_api_with(&mut api, api_docs)
        .layer(Extension(Arc::new(api)))
        .layer(session_layer)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    aide::gen::infer_responses(false);

    router
}

fn with_auth_middleware(router: ApiRouter<AppState>, app_state: AppState) -> ApiRouter<AppState> {
    router.route_layer(middleware::from_fn_with_state(app_state, auth_middleware))
}
