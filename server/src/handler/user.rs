use actix_web::{body::BoxBody, http::StatusCode, web, HttpResponse, ResponseError};
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
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
}

pub async fn create_user(
    req: web::Json<CreateUserRequest>,
    data: web::Data<crate::AppState>,
) -> Result<HttpResponse, UserServiceError> {
    match data
        .user
        .register(req.username.clone(), req.password.clone())
        .await
    {
        Ok(auth) => Ok(HttpResponse::Ok().json(auth)),
        Err(err) => Err(err),
    }
}
