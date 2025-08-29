use axum::{
    extract::{Extension, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use validator::Validate;

use crate::{
    error::Result,
    middleware::Claims,
    models::{response::Response, user::User},
    routes::AppState,
    services::user::UserService,
};

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(length(min = 1, max = 100))]
    pub nickname: String,
    #[validate(length(min = 6))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 1))]
    pub username: String,
    #[validate(length(min = 1))]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user: UserResponse,
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub nickname: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id.map(|id| id.to_hex()).unwrap_or_default(),
            username: user.username,
            nickname: user.nickname,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<Response<AuthResponse>>> {
    payload
        .validate()
        .map_err(|e| crate::error::AppError::Validation(format!("Validation error: {}", e)))?;

    let user_service = UserService::new(
        &state.database.db,
        state.config.jwt_secret.clone(),
        Duration::from_secs(24 * 60 * 60),
    );

    let (user, token) = user_service
        .create_user(payload.username, payload.nickname, payload.password)
        .await?;

    let response = Response {
        data: Some(AuthResponse {
            user: user.into(),
            token,
        }),
        message: "User registered successfully".to_string(),
    };

    Ok(Json(response))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<Response<AuthResponse>>> {
    payload
        .validate()
        .map_err(|e| crate::error::AppError::Validation(format!("Validation error: {}", e)))?;

    let user_service = UserService::new(
        &state.database.db,
        state.config.jwt_secret.clone(),
        Duration::from_secs(24 * 60 * 60),
    );

    let (user, token) = user_service
        .authenticate(payload.username, payload.password)
        .await?;

    let response = Response {
        data: Some(AuthResponse {
            user: user.into(),
            token,
        }),
        message: "Login successful".to_string(),
    };

    Ok(Json(response))
}

pub async fn get_current_user(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Response<UserResponse>>> {
    let user_service = UserService::new(
        &state.database.db,
        state.config.jwt_secret.clone(),
        Duration::from_secs(24 * 60 * 60),
    );

    let user = user_service.get_user_by_id(&claims.sub).await?;

    let response = Response {
        data: Some(user.into()),
        message: "User fetched successfully".to_string(),
    };

    Ok(Json(response))
}
