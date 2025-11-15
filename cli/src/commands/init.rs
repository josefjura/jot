use std::path::Path;

use anyhow::Context;

use crate::{app_config::AppConfig, init::read_profile};

pub fn init_cmd(config: &AppConfig, profile_path: &Path) -> Result<(), anyhow::Error> {
    if config.profile_exists {
        println!("Using profile: {:?}", &profile_path);
        let new_profile = read_profile(config).context("An error during profile initialization")?;

        new_profile.save(profile_path)?;
        println!("Profile updated: {:?}", profile_path);
    } else {
        println!("Profile will be saved as '{:?}'", &profile_path);

        if let Ok(parent) = profile_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid profile path"))
        {
            std::fs::create_dir_all(parent)?;
        }

        let new_profile = read_profile(config).context("An error during profile initialization")?;

        new_profile.save(profile_path)?;
        println!("Profile saved as {:?}", profile_path);
    }

    Ok(())
}
