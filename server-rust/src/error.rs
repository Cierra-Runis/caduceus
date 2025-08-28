use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] mongodb::error::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal server error: {0}")]
    Internal(#[from] anyhow::Error),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("Bcrypt error: {0}")]
    Bcrypt(#[from] bcrypt::BcryptError),

    #[error("BSON ObjectId error: {0}")]
    ObjectId(#[from] mongodb::bson::oid::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(err) => {
                tracing::error!("Database error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error occurred".to_string(),
                )
            }
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Authentication(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            AppError::Internal(err) => {
                tracing::error!("Internal server error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            AppError::Jwt(err) => {
                tracing::error!("JWT error: {:?}", err);
                (StatusCode::UNAUTHORIZED, "Invalid token".to_string())
            }
            AppError::Bcrypt(err) => {
                tracing::error!("Bcrypt error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Encryption error".to_string(),
                )
            }
            AppError::ObjectId(err) => {
                tracing::error!("ObjectId error: {:?}", err);
                (StatusCode::BAD_REQUEST, "Invalid ID format".to_string())
            }
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
