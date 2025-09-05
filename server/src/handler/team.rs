use actix_web::{web, HttpResponse, ResponseError};
use serde::Deserialize;

use crate::{
    models::{response::ApiResponse, user::UserClaims},
    services::team::TeamServiceError,
};

impl ResponseError for TeamServiceError {
    fn error_response(&self) -> HttpResponse {
        let response = ApiResponse::error(&self.to_string());
        HttpResponse::build(self.status_code()).json(response)
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        match *self {
            TeamServiceError::InvalidUserId => actix_web::http::StatusCode::BAD_REQUEST,
            TeamServiceError::UserNotFound => actix_web::http::StatusCode::NOT_FOUND,
            TeamServiceError::Database(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Deserialize)]
pub struct CreateTeamRequest {
    pub name: String,
}

pub async fn create(
    req: web::Json<CreateTeamRequest>,
    data: web::Data<crate::AppState>,
    user: UserClaims,
) -> Result<HttpResponse, TeamServiceError> {
    match data.team_service.create(user.sub, req.name.clone()).await {
        Ok(team) => {
            let response = ApiResponse::success("Team created successfully", team);
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => Err(e),
    }
}
