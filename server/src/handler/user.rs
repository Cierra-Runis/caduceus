use actix_web::{
    body::BoxBody,
    cookie::{Cookie, SameSite},
    http::StatusCode,
    web, HttpResponse, ResponseError,
};
use bcrypt::BcryptError;
use serde::Deserialize;
use time::{Duration, OffsetDateTime};

use crate::{
    models::{response::ApiResponse, user::UserClaims},
    services::user::UserServiceError,
};

impl ResponseError for UserServiceError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        let response = ApiResponse::error(&self.to_string());
        HttpResponse::build(self.status_code()).json(response)
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            UserServiceError::UserNotFound => StatusCode::NOT_FOUND,
            UserServiceError::PasswordNotMatched => StatusCode::UNAUTHORIZED,
            UserServiceError::UserAlreadyExists => StatusCode::CONFLICT,
            UserServiceError::Bcrypt(BcryptError::Truncation(_)) => StatusCode::BAD_REQUEST,
            UserServiceError::Bcrypt(_) => StatusCode::INTERNAL_SERVER_ERROR,
            UserServiceError::Jwt(_) | UserServiceError::Database(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

pub async fn register(
    req: web::Json<RegisterRequest>,
    data: web::Data<crate::AppState>,
) -> Result<HttpResponse, UserServiceError> {
    match data
        .user_service
        .register(req.username.clone(), req.password.clone())
        .await
    {
        Ok(auth) => {
            let expires = OffsetDateTime::now_utc().checked_add(Duration::hours(24));
            let cookie = Cookie::build("token", auth.token.clone())
                .path("/")
                .expires(expires)
                .same_site(SameSite::None)
                .secure(true)
                .http_only(true)
                .finish();
            let response = ApiResponse::success("User registered successfully", auth);
            Ok(HttpResponse::Ok().cookie(cookie).json(response))
        }
        Err(err) => Err(err),
    }
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

pub async fn login(
    req: web::Json<LoginRequest>,
    data: web::Data<crate::AppState>,
) -> Result<HttpResponse, UserServiceError> {
    match data
        .user_service
        .login(req.username.clone(), req.password.clone())
        .await
    {
        Ok(auth) => {
            let expires = OffsetDateTime::now_utc().checked_add(Duration::hours(24));
            let cookie = Cookie::build("token", auth.token.clone())
                .path("/")
                .expires(expires)
                .same_site(SameSite::None)
                .secure(true)
                .http_only(true)
                .finish();
            let response = ApiResponse::success("User logged in successfully", auth);
            Ok(HttpResponse::Ok().cookie(cookie).json(response))
        }
        Err(err) => Err(err),
    }
}

pub async fn logout() -> HttpResponse {
    let mut cookie = Cookie::build("token", "")
        .path("/")
        .same_site(SameSite::Lax)
        .http_only(true)
        .finish();
    cookie.make_removal();
    let response = ApiResponse::success_no_payload("Logged out successfully");
    HttpResponse::Ok().cookie(cookie).json(response)
}

pub async fn teams(
    data: web::Data<crate::AppState>,
    user: UserClaims,
) -> Result<HttpResponse, UserServiceError> {
    match data.user_service.list_teams(user.sub).await {
        Ok(teams) => {
            let response = ApiResponse::success("Teams retrieved successfully", teams);
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => Err(e),
    }
}

pub async fn projects(
    data: web::Data<crate::AppState>,
    user: UserClaims,
) -> Result<HttpResponse, UserServiceError> {
    match data.user_service.list_projects(user.sub).await {
        Ok(projects) => {
            let response = ApiResponse::success("Projects retrieved successfully", projects);
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => Err(e),
    }
}

pub async fn me(
    data: web::Data<crate::AppState>,
    user: UserClaims,
) -> Result<HttpResponse, UserServiceError> {
    match data.user_service.get_user_by_id(user.sub).await {
        Ok(user) => {
            let response = ApiResponse::success("User retrieved successfully", user);
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => Err(e),
    }
}
