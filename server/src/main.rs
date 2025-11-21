#![deny(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
#![warn(clippy::expect_used)]

use db::open_auth_db;
use dotenvy::dotenv;
use errors::ApplicationError;
use router::setup_router;
use std::env;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod db;
mod errors;
mod jwt;
mod model;
mod router;
mod state;
mod util;

#[tokio::main]
async fn main() -> Result<(), ApplicationError> {
    if let Err(e) = run().await {
        // Print the error using Display
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

async fn run() -> Result<(), ApplicationError> {
    setup_tracing();

    let (host, port, jwt_secret, data_dir) = setup_env()?;

    // Ensure data directories exist
    std::fs::create_dir_all(&data_dir).map_err(|e| {
        ApplicationError::Internal(format!("Failed to create data directory: {}", e))
    })?;

    let users_dir = data_dir.join("users");
    std::fs::create_dir_all(&users_dir).map_err(|e| {
        ApplicationError::Internal(format!("Failed to create users directory: {}", e))
    })?;

    // Open auth database
    let auth_db_path = data_dir.join("auth.db");
    let auth_db = open_auth_db(&auth_db_path)
        .map_err(|e| ApplicationError::Internal(format!("Failed to open auth database: {}", e)))?;

    let app = setup_router(auth_db, &jwt_secret, data_dir);

    let address = format!("{}:{}", host, port);
    info!("Starting server on {}", address);

    let listener = TcpListener::bind(address)
        .await
        .map_err(ApplicationError::from)?;

    info!(
        "Listening on: {}",
        listener.local_addr().map_err(ApplicationError::from)?
    );

    axum::serve(listener, app.into_make_service())
        .await
        .map_err(ApplicationError::CannotServe)?;
    Ok(())
}

fn setup_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!(
                    "{crate_name}=debug,tower_http=debug",
                    crate_name = env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

fn setup_env() -> Result<(String, String, String, std::path::PathBuf), ApplicationError> {
    dotenv().ok();

    let host = std::env::var("JOT_HOST")
        .map_err(|e| ApplicationError::EnvError(e, "JOT_HOST".to_string()))?;
    let port = std::env::var("JOT_PORT")
        .map_err(|e| ApplicationError::EnvError(e, "JOT_PORT".to_string()))?;
    let jwt_secret = std::env::var("JOT_JWT_SECRET")
        .map_err(|e| ApplicationError::EnvError(e, "JOT_JWT_SECRET".to_string()))?;
    let data_dir = env::var("JOT_DATA_DIR").unwrap_or_else(|_| "./data".to_string());

    Ok((host, port, jwt_secret, std::path::PathBuf::from(data_dir)))
}
