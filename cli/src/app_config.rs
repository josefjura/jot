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
        AppConfig {
            profile_path: "./".to_string(),
            db_path: format!("./{}", DEFAULT_DB_FILENAME),
            profile_exists: false,
        }
    }
}

impl AppConfig {
    pub fn from_args(_args: ConfigArgs, profile_path: &Path, profile: Option<&Profile>) -> Self {
        let defaults = AppConfig::default();

        let db_path = profile
            .and_then(|p| p.db_path.as_ref())
            .cloned()
            .or(build_db_path(profile_path))
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

fn build_db_path(profile_path: &Path) -> Option<String> {
    profile_path
        .parent()
        .map(|p| p.join(Path::new(DEFAULT_DB_FILENAME)))
        .map(|p| p.to_string_lossy().into_owned())
}
