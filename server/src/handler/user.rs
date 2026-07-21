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

pub async fn logout() -> HttpResponse {
    let cookie = Cookie::build("token", "")
        .path("/")
        .expires(OffsetDateTime::now_utc() - Duration::days(365))
        .max_age(Duration::seconds(0))
        .finish();

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

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    // `actix_web::test` is deliberately not imported: doing so would also pull
    // in its `test` attribute macro, shadowing the built-in `#[test]`.
    use actix_web::{App, ResponseError, body::to_bytes};

    #[test]
    fn test_user_service_error_status_codes() {
        assert_eq!(
            UserServiceError::UserNotFound.status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            UserServiceError::PasswordNotMatched.status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            UserServiceError::UserAlreadyExists.status_code(),
            StatusCode::CONFLICT
        );
        assert_eq!(
            UserServiceError::Bcrypt(BcryptError::Truncation(100)).status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            UserServiceError::Bcrypt(BcryptError::CostNotAllowed(99)).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(
            UserServiceError::Jwt(jsonwebtoken::errors::ErrorKind::InvalidToken.into())
                .status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(
            UserServiceError::Database(mongodb::error::Error::custom("boom")).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[actix_web::test]
    async fn test_user_service_error_response_body() {
        let resp = UserServiceError::UserAlreadyExists.error_response();
        assert_eq!(resp.status(), StatusCode::CONFLICT);
        let body = to_bytes(resp.into_body()).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["message"], "User already exists");
        assert_eq!(json["payload"], serde_json::Value::Null);
    }

    #[actix_web::test]
    async fn test_logout_clears_token_cookie() {
        let app = actix_web::test::init_service(
            App::new().route("/logout", web::post().to(logout)),
        )
        .await;
        let req = actix_web::test::TestRequest::post()
            .uri("/logout")
            .to_request();
        let resp = actix_web::test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let cookie = resp
            .response()
            .cookies()
            .find(|c| c.name() == "token")
            .expect("logout must reset the token cookie");
        assert_eq!(cookie.value(), "");
        assert_eq!(cookie.max_age(), Some(Duration::seconds(0)));
        assert!(
            cookie.expires_datetime().unwrap() < OffsetDateTime::now_utc(),
            "cookie expiry must be in the past"
        );

        let body: serde_json::Value = actix_web::test::read_body_json(resp).await;
        assert_eq!(body["message"], "Logged out successfully");
    }
}
