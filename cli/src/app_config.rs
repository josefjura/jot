use std::path::Path;

use serde::Serialize;

use crate::{args::ConfigArgs, profile::Profile};

pub const DEFAULT_DB_FILENAME: &str = "notes.db";

#[derive(Debug, Serialize)]
pub struct AppConfig {
    pub profile_path: String,
    pub db_path: String,
    pub profile_exists: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        let db_path = get_default_db_path()
            .unwrap_or_else(|| format!("./{}", DEFAULT_DB_FILENAME));

        AppConfig {
            profile_path: "./".to_string(),
            db_path,
            profile_exists: false,
        }
    }
}

/// Get default database path using XDG data directory
fn get_default_db_path() -> Option<String> {
    directories::ProjectDirs::from("com", "beardo", "jot")
        .map(|dirs| dirs.data_dir().join(DEFAULT_DB_FILENAME))
        .and_then(|path| path.to_str().map(|s| s.to_string()))
}

impl AppConfig {
    pub fn from_args(_args: ConfigArgs, profile_path: &Path, profile: Option<&Profile>) -> Self {
        let defaults = AppConfig::default();

        let db_path = profile
            .and_then(|p| p.db_path.as_ref())
            .cloned()
            .unwrap_or(defaults.db_path);

        AppConfig {
            profile_exists: profile.is_some(),
            profile_path: profile_path
                .to_str()
                .map(|p| p.to_string())
                .unwrap_or(defaults.profile_path),
            db_path,
        }
    }
}
