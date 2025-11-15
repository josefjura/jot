use aide::{
    axum::{routing::get_with, ApiRouter, IntoApiResponse},
    transform::TransformOperation,
};
use axum::http::StatusCode;

use crate::state::AppState;

use super::with_auth_middleware;

fn health_routes_public() -> ApiRouter<AppState> {
    ApiRouter::new().api_route("/health/ping", get_with(ping, ping_docs))
}

fn health_routes_private(app_state: AppState) -> ApiRouter<AppState> {
    let router = ApiRouter::new().api_route("/health/auth", get_with(auth_ping, auth_ping_docs));

    with_auth_middleware(router, app_state)
}

pub fn health_routes(app_state: AppState) -> ApiRouter<AppState> {
    health_routes_public().merge(health_routes_private(app_state))
}

pub async fn ping() -> impl IntoApiResponse {
    StatusCode::OK
}

pub fn ping_docs(op: TransformOperation) -> TransformOperation {
    op.summary("Health check")
        .description("Health check endpoint")
        .tag("Health")
        .response::<200, ()>() // Simple 200 OK response with no body
}

pub async fn auth_ping() -> impl IntoApiResponse {
    StatusCode::OK
}

pub fn auth_ping_docs(op: TransformOperation) -> TransformOperation {
    op.summary("Auth health check")
        .description("Health check endpoint requiring authentication")
        .tag("Health")
        .response::<200, ()>() // Simple 200 OK response with no body
        .response_with::<401, (), _>(|res| res.description("Not authenticated"))
}
