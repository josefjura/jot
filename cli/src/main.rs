#![deny(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
#![warn(clippy::expect_used)]

use crate::app_config::AppConfig;
use args::{CliArgs, Command};
use clap::Parser;
use commands::{config::config_cmd, init::init_cmd, note::note_cmd};
use profile::{get_profile_path, Profile};

mod app_config;
mod args;
mod commands;
mod db;
mod editor;
mod formatters;
mod init;
mod profile;
mod utils;

#[cfg(test)]
mod test;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse();

    let profile_path = get_profile_path(&args.config.profile_path);

    if let Some(command) = args.command {
        let profile = Profile::from_path(&profile_path)?;
        let config = AppConfig::from_args(args.config, &profile_path, profile.as_ref());

        match command {
            Command::Config => config_cmd(config)?,
            Command::Init => init_cmd(&config, &profile_path)?,
            Command::Note(subcommand) => {
                let db_path = std::path::Path::new(&config.db_path);
                note_cmd(db_path, subcommand)?;
            }
            Command::Down(args) => {
                let db_path = std::path::Path::new(&config.db_path);
                note_cmd(db_path, args::NoteCommand::Add(args))?;
            }
        }
    }

    Ok(())
}
