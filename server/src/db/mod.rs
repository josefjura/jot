use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};
use tracing::info;

pub mod auth;
pub mod notes;
pub mod user;

pub async fn create_db_pool(path: &str) -> Result<SqlitePool, sqlx::Error> {
    info!("Setting up database at {}", path);
    let opts = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true);

    let db = SqlitePoolOptions::new().connect_with(opts).await?;
    info!("Connected to database");
    sqlx::migrate!().run(&db).await?;
    info!("Migrated database");
    Ok(db)
}
