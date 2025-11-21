use crate::{
    args::ProfileCommand,
    profile::{self, Profile},
};

pub fn profile_cmd(subcommand: Option<ProfileCommand>) -> Result<(), anyhow::Error> {
    match subcommand.unwrap_or(ProfileCommand::Current) {
        ProfileCommand::Use { name } => {
            // Set as current profile
            profile::set_current_profile_name(&name)?;

            // Create profile config if it doesn't exist
            let config_path = profile::get_profile_config_path(&name);
            if !config_path.exists() {
                if let Some(parent) = config_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                let new_profile = Profile::default();
                new_profile.save(&config_path)?;
                println!("Created new profile: {}", name);
            }

            println!("Switched to profile: {}", name);
        }
        ProfileCommand::List => {
            let profiles = profile::list_profiles()?;
            let current =
                profile::get_current_profile_name().unwrap_or_else(|_| "default".to_string());

            println!("Available profiles:");
            for profile_name in profiles {
                let marker = if profile_name == current { "*" } else { " " };
                let db_path = profile::get_profile_db_path(&profile_name);
                println!("{} {} ({})", marker, profile_name, db_path.display());
            }
        }
        ProfileCommand::Current => {
            let current = profile::get_current_profile_name()?;
            let db_path = profile::get_profile_db_path(&current);
            println!("Current profile: {} ({})", current, db_path.display());
        }
    }

    Ok(())
}
