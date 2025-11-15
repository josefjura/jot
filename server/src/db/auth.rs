use sqlx::SqlitePool;

use crate::{
    errors::{AuthError, DbError},
    jwt::verify_password,
    model::{
        auth::ChallengeResult,
        user::{User, UserEntity},
    },
};

pub async fn check_email_password(
    email: &str,
    password: String,
    db: &SqlitePool,
) -> Result<User, AuthError> {
    let user = sqlx::query_as!(
        UserEntity,
        r#"
				SELECT id, name, email, password
				FROM users
				WHERE email = ?
			"#,
        email
    )
    .fetch_optional(db)
    .await
    .map_err(|_| AuthError::DatabaseError)?;

    match user {
        Some(user) => {
            if verify_password(&password, &user.password) {
                Ok(user.into())
            } else {
                Err(AuthError::PasswordIncorrect)?
            }
        }
        None => Err(AuthError::UserNotFound)?,
    }
}

pub async fn create_device_challenge(device_code: String, db: &SqlitePool) -> Result<(), DbError> {
    sqlx::query!(
        r#"
			INSERT INTO device_auth (device_code)
			VALUES (?)
		"#,
        device_code
    )
    .execute(db)
    .await
    .map_err(DbError::Unknown)?;

    Ok(())
}

pub async fn add_token_to_device_challenge(
    device_code: &str,
    token: String,
    db: &SqlitePool,
) -> Result<bool, DbError> {
    let query = sqlx::query!(
        r#"
			UPDATE device_auth
			SET token = ?
			WHERE device_code = ?
		"#,
        token,
        device_code
    )
    .execute(db)
    .await
    .map_err(DbError::Unknown)?;

    if query.rows_affected() == 0 {
        return Ok(false);
    }

    Ok(true)
}

pub async fn delete_device_challenge(
    device_code: String,
    db: &SqlitePool,
) -> Result<bool, DbError> {
    let query = sqlx::query!(
        r#"
			DELETE FROM device_auth
			WHERE device_code = ?
		"#,
        device_code
    )
    .execute(db)
    .await
    .map_err(DbError::Unknown)?;

    Ok(query.rows_affected() > 0)
}

pub async fn get_token_from_device_challenge(
    device_code: String,
    db: &SqlitePool,
) -> Result<ChallengeResult, DbError> {
    let current_time = chrono::Utc::now().timestamp();

    let auth = sqlx::query!(
        "SELECT token FROM device_auth WHERE device_code = ? AND expire_date > ?",
        device_code,
        current_time
    )
    .fetch_optional(db)
    .await
    .map_err(DbError::Unknown)?;

    let challenge_result = match auth.map(|a| a.token) {
        None => ChallengeResult::NoChallenge,
        Some(None) => ChallengeResult::Pending,
        Some(Some(token)) => ChallengeResult::Success(token),
    };

    Ok(challenge_result)
}
