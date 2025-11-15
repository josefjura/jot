use predicates::{
    prelude::{predicate, PredicateBooleanExt},
    Predicate,
};

// Helper predicates for common assertions
pub fn contains_login_success_messages() -> impl Predicate<str> {
    predicate::str::contains(r#"Mocking sending device code"#)
        .and(predicate::str::contains(
            r#"Browser window should open, if not, please visit following URL to login"#,
        ))
        .and(predicate::str::contains(r#"Mocking polling for token with device code:"#).count(2))
        .and(predicate::str::contains(r#"User successfully logged in."#))
}
