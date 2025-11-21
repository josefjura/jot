use aide::{
    axum::{routing::get_with, ApiRouter, IntoApiResponse},
    transform::TransformOperation,
};
use axum::{http::StatusCode, response::IntoResponse, Extension};

use crate::{
    errors::{AuthError, RestError},
    model::user::User,
    state::AppState,
};

fn health_routes_public() -> ApiRouter<AppState> {
    ApiRouter::new().api_route("/health/ping", get_with(ping, ping_docs))
}

fn health_routes_private(_app_state: AppState) -> ApiRouter<AppState> {
    ApiRouter::new().api_route("/health/auth", get_with(auth_ping, auth_ping_docs))
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

pub async fn auth_ping(user_opt: Option<Extension<User>>) -> impl IntoApiResponse {
    // Check authentication
    match user_opt {
        Some(_) => StatusCode::OK.into_response(),
        None => RestError::Authorization(AuthError::TokenNotFound).into_response(),
    }
}

pub fn auth_ping_docs(op: TransformOperation) -> TransformOperation {
    op.summary("Auth health check")
        .description("Health check endpoint requiring authentication")
        .tag("Health")
        .response::<200, ()>() // Simple 200 OK response with no body
        .response_with::<401, (), _>(|res| res.description("Not authenticated"))
}
