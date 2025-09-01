use actix_web::{web, HttpResponse, Result};
use serde::Deserialize;

use crate::services::user::UserServiceError;

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
}

pub async fn create_user(
    req: web::Json<CreateUserRequest>,
    data: web::Data<crate::AppState>,
) -> Result<HttpResponse> {
    match data
        .user
        .register(req.username.clone(), req.password.clone())
        .await
    {
        Ok(user) => Ok(HttpResponse::Ok().json(user)),
        Err(UserServiceError::UserAlreadyExists) => {
            Ok(HttpResponse::Conflict().json("User already exists"))
        }
        Err(UserServiceError::InternalError { details }) => {
            Ok(HttpResponse::InternalServerError().json(details))
        }
    }
}
