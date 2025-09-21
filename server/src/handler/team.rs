use actix_web::{http::StatusCode, web, HttpResponse, ResponseError};
use bson::oid::ObjectId;
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

    fn status_code(&self) -> StatusCode {
        match *self {
            TeamServiceError::UserNotFound => StatusCode::NOT_FOUND,
            TeamServiceError::TeamNotFound => StatusCode::NOT_FOUND,
            TeamServiceError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
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

#[derive(Debug, Deserialize)]
pub struct TeamProjectsQuery {
    id: ObjectId,
}

pub async fn projects(
    req: web::Query<TeamProjectsQuery>,
    data: web::Data<crate::AppState>,
) -> Result<HttpResponse, TeamServiceError> {
    match data.team_service.list_projects(req.id).await {
        Ok(projects) => {
            let response = ApiResponse::success("Projects fetched successfully", projects);
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => Err(e),
    }
}
