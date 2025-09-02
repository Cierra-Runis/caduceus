use actix_web::{web, HttpResponse, ResponseError};
use bson::de;
use serde::{Deserialize, Serialize};

use crate::{models::user::UserClaims, services::team::TeamServiceError};

#[derive(Serialize)]
struct Response {
    message: String,
}

impl ResponseError for TeamServiceError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(Response {
            message: self.to_string(),
        })
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
        Ok(team) => Ok(HttpResponse::Ok().json(team)),
        Err(e) => Err(e),
    }
}
