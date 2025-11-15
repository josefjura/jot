use assert_cmd::Command;
use predicates::prelude::*;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub struct TestContext {
    pub temp_dir: TempDir,
    pub config_path: PathBuf,
    pub key_path: PathBuf,
}

impl TestContext {
    pub fn new(toml_path: &str) -> Self {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();
        let config_path = dir_path.join(Path::new("local.toml"));
        let key_path = dir_path.join(Path::new("api_key"));

        // Copy test config if needed
        std::fs::copy(toml_path, &config_path).unwrap();

        Self {
            temp_dir,
            config_path,
            key_path,
        }
    }

    pub fn command(&self) -> Command {
        let mut cmd = Command::cargo_bin("jot-cli").unwrap();
        cmd.env("JOT_PROFILE", self.config_path.to_str().unwrap())
            .arg("-m"); // Always use mock mode in tests
        cmd
    }

    pub fn assert_key_file_contains(&self, expected_content: &[u8]) {
        assert!(self.key_path.exists(), "Key file should exist");
        let content = std::fs::read(&self.key_path).unwrap();
        assert_eq!(content, expected_content, "Key file content mismatch");
    }
}
