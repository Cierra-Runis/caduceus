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

#[derive(Deserialize, utoipa::ToSchema)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[utoipa::path(
    post,
    path = "/api/register",
    tag = "user",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "User registered; also sets the `token` cookie", body = crate::openapi::ApiSuccess<crate::services::user::AuthPayload>),
        (status = 400, description = "Password too long", body = crate::openapi::ApiMessage),
        (status = 409, description = "Username already taken", body = crate::openapi::ApiMessage),
        (status = 500, description = "Internal error", body = crate::openapi::ApiMessage),
    )
)]
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

            let builder = Cookie::build("token", auth.token.clone())
                .path("/")
                .expires(expires)
                .same_site(SameSite::None)
                .http_only(true);

            #[cfg(debug_assertions)]
            let cookie = builder.finish();
            #[cfg(not(debug_assertions))]
            let cookie = builder.secure(true).finish();

            let response = ApiResponse::success("User registered successfully", auth);
            Ok(HttpResponse::Ok().cookie(cookie).json(response))
        }
        Err(err) => Err(err),
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[utoipa::path(
    post,
    path = "/api/login",
    tag = "user",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Logged in; also sets the `token` cookie", body = crate::openapi::ApiSuccess<crate::services::user::AuthPayload>),
        (status = 401, description = "Wrong password", body = crate::openapi::ApiMessage),
        (status = 404, description = "User not found", body = crate::openapi::ApiMessage),
        (status = 500, description = "Internal error", body = crate::openapi::ApiMessage),
    )
)]
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

            let builder = Cookie::build("token", auth.token.clone())
                .path("/")
                .expires(expires)
                .same_site(SameSite::None)
                .http_only(true);

            #[cfg(debug_assertions)]
            let cookie = builder.finish();
            #[cfg(not(debug_assertions))]
            let cookie = builder.secure(true).finish();

            let response = ApiResponse::success("User logged in successfully", auth);
            Ok(HttpResponse::Ok().cookie(cookie).json(response))
        }
        Err(err) => Err(err),
    }
}

#[utoipa::path(
    post,
    path = "/api/logout",
    tag = "user",
    responses(
        (status = 200, description = "Logged out; clears the `token` cookie", body = crate::openapi::ApiMessage),
    )
)]
pub async fn logout() -> HttpResponse {
    let cookie = Cookie::build("token", "")
        .path("/")
        .expires(OffsetDateTime::now_utc() - Duration::days(365))
        .max_age(Duration::seconds(0))
        .finish();

    let response = ApiResponse::success_no_payload("Logged out successfully");
    HttpResponse::Ok().cookie(cookie).json(response)
}

#[utoipa::path(
    get,
    path = "/api/user/teams",
    tag = "user",
    responses(
        (status = 200, description = "Teams the current user belongs to", body = crate::openapi::ApiSuccess<Vec<crate::models::team::TeamPayload>>),
        (status = 401, description = "Missing or invalid JWT", body = crate::openapi::ApiMessage),
        (status = 404, description = "User not found", body = crate::openapi::ApiMessage),
        (status = 500, description = "Internal error", body = crate::openapi::ApiMessage),
    )
)]
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

#[utoipa::path(
    get,
    path = "/api/user/projects",
    tag = "user",
    responses(
        (status = 200, description = "Projects the current user can access", body = crate::openapi::ApiSuccess<Vec<crate::models::project::ProjectPayload>>),
        (status = 401, description = "Missing or invalid JWT", body = crate::openapi::ApiMessage),
        (status = 404, description = "User not found", body = crate::openapi::ApiMessage),
        (status = 500, description = "Internal error", body = crate::openapi::ApiMessage),
    )
)]
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

#[utoipa::path(
    get,
    path = "/api/user/me",
    tag = "user",
    responses(
        (status = 200, description = "The current user", body = crate::openapi::ApiSuccess<crate::models::user::UserPayload>),
        (status = 401, description = "Missing or invalid JWT", body = crate::openapi::ApiMessage),
        (status = 404, description = "User not found", body = crate::openapi::ApiMessage),
        (status = 500, description = "Internal error", body = crate::openapi::ApiMessage),
    )
)]
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
