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
    // Test that --profile-path arg overrides JOT_PROFILE env var
    let mut cmd = Command::cargo_bin("jot-cli").unwrap();

    let assert = cmd
        .env("JOT_PROFILE", "wrong_profile")
        .args(&["--profile-path", "test_profile_arg"])
        .arg("config")
        .assert();

    assert
        .success()
        .stdout(
            contains(r#""profile_name": "test_profile_arg""#)
                .and(contains(r#""db_path""#)),
        )
        .stderr(is_empty());
}

#[test]
fn test_profile_env() {
    // Test that JOT_PROFILE env var sets the profile name
    let mut cmd = Command::cargo_bin("jot-cli").unwrap();

    let assert = cmd
        .env("JOT_PROFILE", "test_profile_env")
        .arg("config")
        .assert();

    assert
        .success()
        .stdout(
            contains(r#""profile_name": "test_profile_env""#)
                .and(contains(r#""db_path""#)),
        )
        .stderr(is_empty());
}
