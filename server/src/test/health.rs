use axum_test::TestServer;
use serde_json::json;

use crate::{
    model::auth::LoginResponse,
    router::setup_router,
    test::{setup_server, JWT_SECRET},
};

#[sqlx::test]
async fn health_ok(db: sqlx::Pool<sqlx::Sqlite>) {
    let router = setup_router(db, JWT_SECRET);

    let server = TestServer::new(router).unwrap();

    let response = server.get("/health/ping").await;

    response.assert_status_ok();
}

#[sqlx::test]
async fn health_auth_forbidden(db: sqlx::Pool<sqlx::Sqlite>) {
    let server = setup_server(db);

    let response = server.get("/health/auth").await;

    response.assert_status_forbidden();
}

#[sqlx::test(fixtures("user"))]
async fn health_auth_ok(db: sqlx::Pool<sqlx::Sqlite>) {
    let server = setup_server(db);

    let login_response = server
        .post("/auth/login")
        .json(&json!({
                "username": "alice@email.com",
                "password": "pass",
        }))
        .await;

    login_response.assert_status_ok();

    let json = login_response.json::<LoginResponse>();

    let run_response = server
        .get("/health/auth")
        .authorization_bearer(json.token)
        .await;

    run_response.assert_status_ok();
}
