use predicate::str::is_empty;
use predicates::prelude::*;

use crate::web_client::mock::MOCK_TOKEN;

use super::{asserts::contains_login_success_messages, test_context::TestContext};

#[test]
fn test_login() {
    // Arrange
    let ctx = TestContext::new("test_assets/profile/local.toml");

    // Act
    let assert = ctx.command().arg("login").assert();

    // Assert
    assert
        .success()
        .stdout(contains_login_success_messages())
        .stderr(is_empty());

    ctx.assert_key_file_contains(MOCK_TOKEN.as_bytes());
}
