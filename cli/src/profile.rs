use std::path::{Path, PathBuf};

use anyhow::{Context, Ok};
use config::{Config, File, FileFormat};
use serde::{Deserialize, Serialize};

use crate::app_config::AppConfig;

const DEFAULT_PROFILE_FILENAME: &str = "default.toml";

#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    pub server_url: Option<String>,
    pub api_key_path: Option<String>,
}

impl Default for Profile {
    fn default() -> Self {
        let c = AppConfig::default();

        Profile {
            server_url: Some(c.server_url),
            api_key_path: Some(c.api_key_path),
        }
    }
}

impl Profile {
    pub fn from_path(profile: &Path) -> anyhow::Result<Option<Self>> {
        if !profile.exists() {
            return Ok(None);
        }

        let profile_path = profile
            .to_str()
            .map(|p| p.to_string())
            .context("Invalid profile path, probably not UTF")?;

        let config = Config::builder()
            .add_source(File::new(&profile_path, FileFormat::Toml))
            .build()
            .context("Failed to build config")?;

        let profile: Self = config
            .try_deserialize()
            .context("Failed to deserialize config")?;

        Ok(Some(profile))
    }

    pub fn save(&self, profile_path: &Path) -> anyhow::Result<()> {
        let content = toml::to_string(self).context("Failed to serialize profile")?;

        std::fs::write(profile_path, content).context("Failed to write profile")?;

        Ok(())
    }
}

pub fn get_profile_path(arg_profile: &Option<String>) -> PathBuf {
    let dirs = directories::ProjectDirs::from("com", "beardo", "jot");

    let xdg_path = dirs.map(|dir| {
        let d = dir.config_dir();
        d.join(DEFAULT_PROFILE_FILENAME).to_path_buf()
    });

    let arg_path = arg_profile.clone().map(PathBuf::from);

    arg_path
        .or(xdg_path)
        .unwrap_or_else(|| Path::new(".").to_path_buf())
}
