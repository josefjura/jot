use std::path::Path;

use serde::Serialize;

use crate::{
    args::ConfigArgs,
    profile::{self, Profile},
};

#[derive(Debug, Serialize)]
pub struct AppConfig {
    pub profile_name: String,
    pub profile_path: String,
    pub db_path: String,
    pub profile_exists: bool,
    pub default_tags: Vec<String>,
}

impl AppConfig {
    pub fn from_args(
        _args: ConfigArgs,
        profile_path: &Path,
        profile: Option<&Profile>,
        profile_name: &str,
    ) -> Self {
        // Get DB path: profile config > computed path for profile name
        let db_path = profile
            .and_then(|p| p.db_path.as_ref())
            .cloned()
            .unwrap_or_else(|| {
                profile::get_profile_db_path(profile_name)
                    .to_string_lossy()
                    .to_string()
            });

        let default_tags = profile.map(|p| p.default_tags.clone()).unwrap_or_default();

        AppConfig {
            profile_name: profile_name.to_string(),
            profile_exists: profile.is_some(),
            profile_path: profile_path
                .to_str()
                .map(|p| p.to_string())
                .unwrap_or_else(|| "./".to_string()),
            db_path,
            default_tags,
        }
    }
}
