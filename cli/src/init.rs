use anyhow::{Context, Ok};
use cliclack::input;

use crate::{app_config::AppConfig, profile::Profile};

pub fn read_profile(defaults: &AppConfig) -> anyhow::Result<Profile> {
    let profile = Profile {
        db_path: Some(read_db_path(&defaults.db_path)?),
    };

    Ok(profile)
}

fn read_db_path(default: &str) -> anyhow::Result<String> {
    input("Local database path")
        .placeholder(default)
        .default_input(default)
        .required(true)
        .interact()
        .context("Couldn't read database path")
}
