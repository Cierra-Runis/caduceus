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
            TeamServiceError::AccessDenied => StatusCode::FORBIDDEN,
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
    user: UserClaims,
) -> Result<HttpResponse, TeamServiceError> {
    match data.team_service.list_projects(req.id, user.sub).await {
        Ok(projects) => {
            let response = ApiResponse::success("Projects fetched successfully", projects);
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => Err(e),
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use actix_web::{ResponseError, body::to_bytes};

    #[test]
    fn test_team_service_error_status_codes() {
        assert_eq!(
            TeamServiceError::UserNotFound.status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            TeamServiceError::TeamNotFound.status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            TeamServiceError::AccessDenied.status_code(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            TeamServiceError::Database(mongodb::error::Error::custom("boom")).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[actix_web::test]
    async fn test_team_service_error_response_body() {
        let resp = TeamServiceError::TeamNotFound.error_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let body = to_bytes(resp.into_body()).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["message"], "Team not found");
        assert_eq!(json["payload"], serde_json::Value::Null);
    }
}
