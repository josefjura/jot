use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use jsonwebtoken::{EncodingKey, Header};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::errors::AuthError;

/// Struct for holding data from the JWT.
#[derive(Debug, Deserialize, Serialize)]
pub struct TokenClaims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}

#[allow(dead_code)]
pub fn hash_password(password: &str) -> Result<String, AuthError> {
    // Create an instance of the Argon2 hasher
    let argon2 = Argon2::default();

    // Generate a secure random salt
    let salt = SaltString::generate(&mut OsRng);

    // Hash the password
    let hashed = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| {
            error!("{}", e);
            AuthError::PasswordHash("Failed to hash password".to_string())
        })?
        .to_string();

    Ok(hashed)
}

pub fn verify_password(password: &str, hashed_password: &str) -> bool {
    let is_valid = match PasswordHash::new(hashed_password) {
        Ok(parsed_hash) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_or(false, |_| true),
        Err(_err) => false,
    };

    is_valid
}

pub fn create_token(user_id: i64, secret: &[u8]) -> Result<String, AuthError> {
    let now = chrono::Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + chrono::Duration::days(7)).timestamp() as usize;
    let claims = TokenClaims {
        sub: user_id.to_string(),
        exp,
        iat,
    };

    let token = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )
    .map_err(|e| {
        error!("{}", e);
        AuthError::TokenCreation("Failed to create token".to_string())
    })?;

    Ok(token)
}

#[cfg(test)]
mod test {

    use crate::jwt::{hash_password, verify_password};

    #[test]
    fn test_token_claims() {
        let hash = hash_password("pass").unwrap();
        println!("hash: {}", hash);
        let valid = verify_password("pass", &hash);
        assert!(valid);
    }
}
