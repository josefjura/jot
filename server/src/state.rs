use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
    pub auth_db: Arc<Mutex<Connection>>, // Auth database (users, device_auth)
    pub jwt_secret: String,
    pub data_dir: PathBuf, // Directory for per-user note databases
}

impl AppState {
    pub fn new(auth_db: Connection, jwt_secret: &str, data_dir: PathBuf) -> Self {
        Self {
            auth_db: Arc::new(Mutex::new(auth_db)),
            jwt_secret: jwt_secret.to_string(),
            data_dir,
        }
    }

    /// Get path to a user's notes database
    pub fn user_db_path(&self, user_id: &str) -> PathBuf {
        self.data_dir.join("users").join(format!("{}.db", user_id))
    }
}
