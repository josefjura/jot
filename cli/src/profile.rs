use std::path::{Path, PathBuf};

use anyhow::{Context, Ok};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    pub db_path: Option<String>,
    #[serde(default)]
    pub default_tags: Vec<String>,
}

impl Default for Profile {
    fn default() -> Self {
        Profile {
            db_path: None,
            default_tags: vec![],
        }
    }
}

impl Profile {
    pub fn from_path(profile: &Path) -> anyhow::Result<Option<Self>> {
        if !profile.exists() {
            return Ok(None);
        }

        let contents = std::fs::read_to_string(profile).context("Failed to read profile file")?;

        let profile: Self = toml::from_str(&contents).context("Failed to deserialize profile")?;

        Ok(Some(profile))
    }

    pub fn save(&self, profile_path: &Path) -> anyhow::Result<()> {
        let content = toml::to_string(self).context("Failed to serialize profile")?;

        std::fs::write(profile_path, content).context("Failed to write profile")?;

        Ok(())
    }
}

/// Get the current active profile name
pub fn get_current_profile_name() -> anyhow::Result<String> {
    let current_file = get_current_profile_file();

    if current_file.exists() {
        let name =
            std::fs::read_to_string(&current_file).context("Failed to read current profile")?;
        Ok(name.trim().to_string())
    } else {
        Ok("default".to_string())
    }
}

/// Set the current active profile name
pub fn set_current_profile_name(name: &str) -> anyhow::Result<()> {
    let current_file = get_current_profile_file();

    if let Some(parent) = current_file.parent() {
        std::fs::create_dir_all(parent).context("Failed to create config directory")?;
    }

    std::fs::write(&current_file, name).context("Failed to write current profile")?;

    Ok(())
}

/// Get the XDG config directory, respecting XDG_CONFIG_HOME
fn get_config_dir() -> PathBuf {
    if let std::result::Result::Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
        // XDG_CONFIG_HOME is the base directory, add "jot" subdirectory
        PathBuf::from(xdg_config).join("jot")
    } else {
        directories::ProjectDirs::from("com", "beardo", "jot")
            .map(|dirs| dirs.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    }
}

/// Get the XDG data directory, respecting XDG_DATA_HOME
fn get_data_dir() -> PathBuf {
    if let std::result::Result::Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
        // XDG_DATA_HOME is the base directory, add "jot" subdirectory
        PathBuf::from(xdg_data).join("jot")
    } else {
        directories::ProjectDirs::from("com", "beardo", "jot")
            .map(|dirs| dirs.data_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    }
}

/// Get path to the "current" profile marker file
fn get_current_profile_file() -> PathBuf {
    get_config_dir().join("current")
}

/// Get path to a profile's config file
pub fn get_profile_config_path(profile_name: &str) -> PathBuf {
    get_config_dir()
        .join("profiles")
        .join(format!("{}.toml", profile_name))
}

/// Get path to a profile's database
pub fn get_profile_db_path(profile_name: &str) -> PathBuf {
    get_data_dir()
        .join("profiles")
        .join(profile_name)
        .join("notes.db")
}

/// List all available profiles
pub fn list_profiles() -> anyhow::Result<Vec<String>> {
    let profiles_dir = get_config_dir().join("profiles");

    if !profiles_dir.exists() {
        return Ok(vec!["default".to_string()]);
    }

    let mut profiles = vec![];

    for entry in std::fs::read_dir(&profiles_dir).context("Failed to read profiles directory")? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("toml") {
            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                profiles.push(name.to_string());
            }
        }
    }

    // Always include default if not present
    if !profiles.contains(&"default".to_string()) {
        profiles.insert(0, "default".to_string());
    }

    profiles.sort();
    Ok(profiles)
}

pub fn get_profile_path(arg_profile: &Option<String>) -> PathBuf {
    if let Some(profile_name) = arg_profile {
        get_profile_config_path(profile_name)
    } else {
        // Use current profile
        let current_name = get_current_profile_name().unwrap_or_else(|_| "default".to_string());
        get_profile_config_path(&current_name)
    }
}
