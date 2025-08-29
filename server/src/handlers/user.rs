use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use validator::Validate;

use crate::{
    error::Result,
    middleware::ValidatedJson,
    models::{response::Response, user::UserPayload},
    routes::AppState,
    services::user::UserService,
};

#[derive(Debug, Serialize)]
pub struct AuthPayload {
    pub user: UserPayload,
    pub token: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    // #[validate(length(min = 1, max = 100))]
    pub nickname: Option<String>,
    #[validate(length(min = 6))]
    pub password: String,
}

pub async fn register(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<RegisterRequest>,
) -> Result<Json<Response<AuthPayload>>> {
    let user_service = UserService::new(
        &state.database.db,
        state.config.jwt_secret.clone(),
        Duration::from_secs(24 * 60 * 60),
    );

    let (user, token) = user_service
        .create_user(payload.username, payload.nickname, payload.password)
        .await?;

    let response = Response {
        data: Some(AuthPayload {
            user: user.into(),
            token,
        }),
        message: "User registered successfully".to_string(),
    };

    Ok(Json(response))
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 1))]
    pub username: String,
    #[validate(length(min = 1))]
    pub password: String,
}

pub async fn login(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<LoginRequest>,
) -> Result<Json<Response<AuthPayload>>> {
    let user_service = UserService::new(
        &state.database.db,
        state.config.jwt_secret.clone(),
        Duration::from_secs(24 * 60 * 60),
    );

    let (user, token) = user_service
        .authenticate(payload.username, payload.password)
        .await?;

    let response = Response {
        data: Some(AuthPayload {
            user: user.into(),
            token,
        }),
        message: "Login successful".to_string(),
    };

    Ok(Json(response))
}
