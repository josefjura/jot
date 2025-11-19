#![allow(clippy::unwrap_used)]

use assert_cmd::Command;
use predicates::prelude::{
    predicate::str::{contains, is_empty},
    PredicateBooleanExt,
};

pub mod asserts;
mod e2e;
mod note_operations;
pub mod test_context;

#[test]
fn test_profile_arg() {
    let mut cmd = Command::cargo_bin("jot-cli").unwrap();

    let assert = cmd
        .env("JOT_PROFILE", "bad_test.toml")
        .args(&["--profile-path", "test_assets/profile/default.toml"])
        .arg("config")
        .assert();

    assert
        .success()
        .stdout(
            contains(r#""profile_path": "test_assets/profile/default.toml""#)
                .and(contains(r#""db_path""#)),
        )
        .stderr(is_empty());
}

#[test]
fn test_profile_env() {
    let mut cmd = Command::cargo_bin("jot-cli").unwrap();

    let assert = cmd
        .env("JOT_PROFILE", "test_assets/profile/default.toml")
        .arg("config")
        .assert();

    assert
        .success()
        .stdout(
            contains(r#""profile_path": "test_assets/profile/default.toml""#)
                .and(contains(r#""db_path""#)),
        )
        .stderr(is_empty());
}
