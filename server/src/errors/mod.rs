use std::env::VarError;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use dto::AppErrorDto;
use serde_json::json;
use thiserror::Error;

pub mod dto;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("Cannot bind to selected address. Error: {0}")]
    CannotBind(#[from] std::io::Error),

    #[error("Error while starting the server. Error: {0}")]
    CannotServe(std::io::Error),

    #[error("Missing environment value: {1}")]
    EnvError(#[source] VarError, String),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Error, Debug)]
pub enum RestError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Database error: {0}")]
    Database(#[from] DbError),
    #[error("Unauthorized: {0}")]
    Authorization(#[from] AuthError),
    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Error communicating with database: {0}")]
    Unknown(String),
}

#[derive(Error, Debug, Clone)]
pub enum AuthError {
    #[error("Username or password incorrect")]
    PasswordIncorrect,
    #[error("Username or password incorrect")]
    UserNotFound,
    #[error("Token was not found")]
    TokenNotFound,
    #[error("Error while connecting to the database.")]
    DatabaseError,
    #[error("Error while creating a token.")]
    TokenCreation(String),
    #[error("Error while hashing password.")]
    PasswordHash(String),
}

// Implementation to convert AppError into a Response
impl IntoResponse for RestError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            RestError::InvalidInput(_) => (
                StatusCode::BAD_REQUEST,
                Json(AppErrorDto::new(&self.to_string()).with_status(StatusCode::BAD_REQUEST)),
            ),
            RestError::Authorization(AuthError::TokenNotFound) => (
                StatusCode::FORBIDDEN,
                Json(AppErrorDto::new(&self.to_string()).with_status(StatusCode::FORBIDDEN)),
            ),
            RestError::Authorization(_) => (
                StatusCode::UNAUTHORIZED,
                Json(AppErrorDto::new(&self.to_string()).with_status(StatusCode::UNAUTHORIZED)),
            ),
            RestError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AppErrorDto::new("Internal server error")),
            ),
            RestError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    AppErrorDto::new("Internal server error")
                        .with_details(json!({ "error": self.to_string() })),
                ),
            ),
        };

        (status, error_message).into_response()
    }
}

// Type alias for handler results
pub type RestResult<T> = Result<T, RestError>;
