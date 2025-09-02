use actix_web::{
    body::BoxBody,
    cookie::{Cookie, SameSite},
    http::StatusCode,
    web, HttpResponse, ResponseError,
};
use bcrypt::BcryptError;
use serde::{Deserialize, Serialize};

use crate::services::user::UserServiceError;

#[derive(Serialize)]
struct Response {
    message: String,
}

impl ResponseError for UserServiceError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(self.status_code()).json(Response {
            message: self.to_string(),
        })
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
            let expires = actix_web::cookie::time::OffsetDateTime::now_utc()
                .checked_add(actix_web::cookie::time::Duration::hours(24));
            let cookie = Cookie::build("token", auth.token.clone())
                .path("/")
                .expires(expires)
                .same_site(SameSite::Lax)
                .http_only(true)
                .finish();
            Ok(HttpResponse::Ok().cookie(cookie).json(auth))
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
            let expires = actix_web::cookie::time::OffsetDateTime::now_utc()
                .checked_add(actix_web::cookie::time::Duration::hours(24));
            let cookie = Cookie::build("token", auth.token.clone())
                .path("/")
                .expires(expires)
                .same_site(SameSite::Lax)
                .http_only(true)
                .finish();
            Ok(HttpResponse::Ok().cookie(cookie).json(auth))
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
    HttpResponse::Ok().cookie(cookie).json(Response {
        message: "Logged out successfully".to_string(),
    })
}
