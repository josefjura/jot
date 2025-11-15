use std::path::Path;

use serde::Serialize;

use crate::{args::ConfigArgs, profile::Profile};

pub const DEFAULT_API_KEY_FILENAME: &str = "api_key";

#[derive(Debug, Serialize)]
pub struct AppConfig {
    #[cfg(debug_assertions)]
    pub mock_server: bool,
    #[cfg(debug_assertions)]
    pub mock_param: Option<String>,
    pub server_url: String,
    pub profile_path: String,
    pub api_key_path: String,
    pub profile_exists: bool,
    pub token: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            #[cfg(debug_assertions)]
            mock_server: false,
            mock_param: None,
            server_url: "http://localhost:9000".to_string(),
            profile_path: "./".to_string(),
            api_key_path: format!("./{}", DEFAULT_API_KEY_FILENAME),
            profile_exists: false,
            token: None,
        }
    }
}

impl AppConfig {
    pub fn from_args(args: ConfigArgs, profile_path: &Path, profile: Option<&Profile>) -> Self {
        let defaults = AppConfig::default();

        let profile_server_url = profile.and_then(|p| p.server_url.as_ref());
        let api_key_path = profile
            .and_then(|p| p.api_key_path.as_ref())
            .cloned()
            .or(build_api_key_path(profile_path))
            .unwrap_or(defaults.api_key_path);

        let token = std::fs::read_to_string(&api_key_path).ok();

        let config = AppConfig {
            #[cfg(debug_assertions)]
            mock_server: args.mock,
            #[cfg(debug_assertions)]
            mock_param: args.mock_param,
            profile_exists: profile.is_some(),
            profile_path: profile_path
                .to_str()
                .map(|p| p.to_string())
                .unwrap_or(defaults.profile_path),
            server_url: args
                .server_url
                .or(profile_server_url.cloned())
                .unwrap_or(defaults.server_url),
            api_key_path,
            token,
        };

        config
    }

    #[allow(dead_code)]
    pub fn is_mock(&self) -> bool {
        #[cfg(debug_assertions)]
        {
            self.mock_server
        }
        #[cfg(not(debug_assertions))]
        {
            false
        }
    }
}



fn build_api_key_path(profile_path: &Path) -> Option<String> {
    profile_path
        .parent()
        .map(|p| p.join(Path::new(DEFAULT_API_KEY_FILENAME)))
        .map(|p| p.to_string_lossy().into_owned())
}
