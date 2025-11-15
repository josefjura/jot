use anyhow::{Context, Ok};
use cliclack::input;

use crate::{app_config::AppConfig, profile::Profile};

pub fn read_profile(defaults: &AppConfig) -> anyhow::Result<Profile> {
    let profile = Profile {
        server_url: Some(read_server_url(&defaults.server_url)?),
        api_key_path: Some(read_api_key_path(&defaults.api_key_path)?),
    };

    Ok(profile)
}

fn read_server_url(default: &str) -> anyhow::Result<String> {
    input("Backend server url")
        .placeholder(default)
        .default_input(default)
        .required(true)
        .interact()
        .context("Couldn't read server URL")
}

fn read_api_key_path(default: &str) -> anyhow::Result<String> {
    input("Api key path")
        .placeholder(default)
        .default_input(default)
        .required(true)
        .interact()
        .context("Couldn't read server URL")
}
