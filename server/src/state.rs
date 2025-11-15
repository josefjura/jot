use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub jwt_secret: String,
}

impl AppState {
    pub fn new(db: SqlitePool, jwt_secret: &str) -> Self {
        Self {
            db,
            jwt_secret: jwt_secret.to_string(),
        }
    }
}
