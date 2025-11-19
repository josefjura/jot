#![deny(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
#![warn(clippy::expect_used)]

use crate::app_config::AppConfig;
use args::{CliArgs, Command};
use clap::Parser;
use commands::{config::config_cmd, note::note_cmd, profile::profile_cmd};
use profile::{get_profile_path, Profile};

mod app_config;
mod args;
mod commands;
mod db;
mod editor;
mod formatters;
mod profile;
mod utils;

#[cfg(test)]
mod test;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse();

    // Determine profile name (from arg or current profile)
    let profile_name = if let Some(ref name) = args.config.profile {
        name.clone()
    } else {
        profile::get_current_profile_name().unwrap_or_else(|_| "default".to_string())
    };

    let profile_path = get_profile_path(&args.config.profile);

    if let Some(command) = args.command {
        let profile = Profile::from_path(&profile_path)?;
        let config = AppConfig::from_args(args.config, &profile_path, profile.as_ref(), &profile_name);

        match command {
            Command::Config => config_cmd(config)?,
            Command::Profile { command } => profile_cmd(command)?,
            Command::Note(subcommand) => {
                let db_path = std::path::Path::new(&config.db_path);
                note_cmd(db_path, subcommand, &config)?;
            }
            Command::Down(args) => {
                let db_path = std::path::Path::new(&config.db_path);
                note_cmd(db_path, args::NoteCommand::Add(args), &config)?;
            }
            Command::List(args) => {
                let db_path = std::path::Path::new(&config.db_path);
                note_cmd(db_path, args::NoteCommand::Search(args), &config)?;
            }
            Command::Completion { shell } => {
                use clap::CommandFactory;
                let mut cmd = args::CliArgs::command();
                clap_complete::generate(shell, &mut cmd, "jot", &mut std::io::stdout());
            }
        }
    }

    Ok(())
}
