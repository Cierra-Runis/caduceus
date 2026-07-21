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

#[derive(Deserialize, utoipa::ToSchema)]
pub struct CreateTeamRequest {
    pub name: String,
}

#[utoipa::path(
    post,
    path = "/api/team",
    tag = "team",
    request_body = CreateTeamRequest,
    responses(
        (status = 200, description = "Team created", body = crate::openapi::ApiSuccess<crate::models::team::TeamPayload>),
        (status = 401, description = "Missing or invalid JWT", body = crate::openapi::ApiMessage),
        (status = 404, description = "Creator not found", body = crate::openapi::ApiMessage),
        (status = 500, description = "Internal error", body = crate::openapi::ApiMessage),
    )
)]
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

#[utoipa::path(
    get,
    path = "/api/team/projects",
    tag = "team",
    params(
        ("id" = String, Query, description = "Team id (24-char hex ObjectId)"),
    ),
    responses(
        (status = 200, description = "Projects owned by the team", body = crate::openapi::ApiSuccess<Vec<crate::models::project::ProjectPayload>>),
        (status = 401, description = "Missing or invalid JWT", body = crate::openapi::ApiMessage),
        (status = 404, description = "Team not found", body = crate::openapi::ApiMessage),
        (status = 500, description = "Internal error", body = crate::openapi::ApiMessage),
    )
)]
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
