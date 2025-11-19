#![allow(clippy::unwrap_used)]

use assert_cmd::Command;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub struct TestContext {
    pub temp_dir: TempDir,
    pub config_path: PathBuf,
}

impl TestContext {
    pub fn new(toml_path: &str) -> Self {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();
        let config_path = dir_path.join(Path::new("local.toml"));

        // Copy test config if needed
        std::fs::copy(toml_path, &config_path).unwrap();

        Self {
            temp_dir,
            config_path,
        }
    }

    pub fn command(&self) -> Command {
        let mut cmd = Command::cargo_bin("jot-cli").unwrap();
        cmd.env("JOT_PROFILE", self.config_path.to_str().unwrap());
        cmd
    }
}
