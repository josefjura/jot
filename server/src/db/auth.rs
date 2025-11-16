use rusqlite::{params, Connection, Result};

use crate::{
    errors::{AuthError, DbError},
    jwt::verify_password,
    model::{
        auth::ChallengeResult,
        user::{User, UserEntity},
    },
};

pub fn check_email_password(
    email: &str,
    password: String,
    conn: &Connection,
) -> Result<User, AuthError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, email, password FROM users WHERE email = ?"
    ).map_err(|_| AuthError::DatabaseError)?;

    let user = stmt.query_row(params![email], |row| {
        Ok(UserEntity {
            id: row.get(0)?,
            name: row.get(1)?,
            email: row.get(2)?,
            password: row.get(3)?,
        })
    });

    match user {
        Ok(user) => {
            if verify_password(&password, &user.password) {
                Ok(user.into())
            } else {
                Err(AuthError::PasswordIncorrect)
            }
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Err(AuthError::UserNotFound),
        Err(_) => Err(AuthError::DatabaseError),
    }
}

pub fn create_device_challenge(device_code: String, user_code: String, conn: &Connection) -> Result<(), DbError> {
    let now = chrono::Utc::now().timestamp();
    let expires_at = now + 600; // 10 minutes

    conn.execute(
        "INSERT INTO device_auth (device_code, user_code, expires_at, created_at) VALUES (?, ?, ?, ?)",
        params![device_code, user_code, expires_at, now]
    ).map_err(|e| DbError::Unknown(e.to_string()))?;

    Ok(())
}

pub fn add_token_to_device_challenge(
    device_code: &str,
    user_id: String,
    conn: &Connection,
) -> Result<bool, DbError> {
    let rows = conn.execute(
        "UPDATE device_auth SET user_id = ? WHERE device_code = ?",
        params![user_id, device_code]
    ).map_err(|e| DbError::Unknown(e.to_string()))?;

    Ok(rows > 0)
}

pub fn delete_device_challenge(
    device_code: String,
    conn: &Connection,
) -> Result<bool, DbError> {
    let rows = conn.execute(
        "DELETE FROM device_auth WHERE device_code = ?",
        params![device_code]
    ).map_err(|e| DbError::Unknown(e.to_string()))?;

    Ok(rows > 0)
}

pub fn get_token_from_device_challenge(
    device_code: String,
    conn: &Connection,
) -> Result<ChallengeResult, DbError> {
    let current_time = chrono::Utc::now().timestamp();

    let mut stmt = conn.prepare(
        "SELECT user_id FROM device_auth WHERE device_code = ? AND expires_at > ?"
    ).map_err(|e| DbError::Unknown(e.to_string()))?;

    let user_id = stmt.query_row(params![device_code, current_time], |row| {
        row.get::<_, Option<String>>(0)
    });

    let challenge_result = match user_id {
        Err(rusqlite::Error::QueryReturnedNoRows) => ChallengeResult::NoChallenge,
        Ok(None) => ChallengeResult::Pending,
        Ok(Some(user_id)) => ChallengeResult::Success(user_id),
        Err(e) => return Err(DbError::Unknown(e.to_string())),
    };

    Ok(challenge_result)
}
