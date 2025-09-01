use actix_web::{web, HttpResponse};
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
