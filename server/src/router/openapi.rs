use std::sync::Arc;

use aide::{
    axum::{routing::get, ApiRouter, IntoApiResponse},
    openapi::{OpenApi, Tag},
    redoc::Redoc,
    transform::TransformOpenApi,
};
use axum::{
    response::{Html, IntoResponse},
    Extension, Json,
};

use crate::state::AppState;

pub fn api_docs(api: TransformOpenApi) -> TransformOpenApi {
    api.title("Jot server")
        .version(env!("CARGO_PKG_VERSION"))
        .summary("API documentation for Jot server")
        .description(include_str!("../../README.md"))
        .tag(Tag {
            name: "Note".into(),
            description: Some("Note management endpoints".into()),
            ..Default::default()
        })
        .tag(Tag {
            name: "Health".into(),
            description: Some("Health check endpoints".into()),
            ..Default::default()
        })
        .tag(Tag {
            name: "Authentication".into(),
            description: Some("Authentication endpoints".into()),
            ..Default::default()
        })
        .tag(Tag {
            name: "Device Authorization".into(),
            description: Some("Endponts for auth management from the CLI client".into()),
            ..Default::default()
        })
}

// Handler to serve Swagger UI
pub async fn serve_swagger() -> impl IntoApiResponse {
    let html = Redoc::new("/docs.json").html();
    Html(html).into_response()
}

// Handler to serve OpenAPI JSON
pub async fn serve_docs(Extension(api): Extension<Arc<OpenApi>>) -> impl IntoApiResponse {
    Json(api.clone())
}

pub fn docs_routes() -> ApiRouter<AppState> {
    ApiRouter::new()
        .route("/docs.json", get(serve_docs))
        .route("/docs", get(serve_swagger))
}
