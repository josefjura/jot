use axum_test::TestServer;
use serde_json::json;

use crate::{model::auth::LoginResponse, router::setup_router};

mod auth;
mod health;
mod note;

const JWT_SECRET: &str = "BrHTysKWKIhwOTyqYvqEUOf3rhTH06Q3k2ZBf3Zbcew=";

pub async fn login(server: &TestServer) -> String {
    let login_response = server
        .post("/auth/login")
        .json(&json!({
                        "username": "alice@email.com",
                        "password": "pass",
        }))
        .await;

    login_response.assert_status_ok();
    let json = login_response.json::<LoginResponse>();

    json.token
}

pub fn setup_server(db: sqlx::Pool<sqlx::Sqlite>) -> TestServer {
    let router = setup_router(db, JWT_SECRET);

    TestServer::builder().build(router).unwrap()
}
