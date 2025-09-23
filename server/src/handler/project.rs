use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use bson::oid::ObjectId;
use bson::serde_helpers::serialize_object_id_as_hex_string;
use serde::{Deserialize, Serialize};

use crate::{
    models::{project::OwnerType, response::ApiResponse, user::UserClaims},
    services::project::ProjectServiceError,
};

impl ResponseError for ProjectServiceError {
    fn error_response(&self) -> HttpResponse {
        let response = ApiResponse::error(&self.to_string());
        HttpResponse::build(self.status_code()).json(response)
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            ProjectServiceError::UserNotFound
            | ProjectServiceError::OwnerNotFound(_)
            | ProjectServiceError::ProjectNotFound => StatusCode::NOT_FOUND,
            ProjectServiceError::CreatorNotMatchOwner => StatusCode::FORBIDDEN,
            ProjectServiceError::CreatorNotMemberOfTeam => StatusCode::FORBIDDEN,
            ProjectServiceError::InvalidOwnerType => StatusCode::BAD_REQUEST,
            ProjectServiceError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct CreateProjectRequest {
    #[serde(serialize_with = "serialize_object_id_as_hex_string")]
    pub owner_id: ObjectId,
    pub owner_type: OwnerType,
    pub name: String,
}

pub async fn create(
    req: actix_web::web::Json<CreateProjectRequest>,
    data: actix_web::web::Data<crate::AppState>,
    user: UserClaims,
) -> Result<HttpResponse, ProjectServiceError> {
    match data
        .project_service
        .create(
            user.sub,
            req.owner_id,
            req.owner_type.clone(),
            req.name.clone(),
        )
        .await
    {
        Ok(project) => {
            let response = ApiResponse::success("Project created successfully", project);
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => Err(e),
    }
}

pub async fn find_by_id(
    id: actix_web::web::Path<String>,
    data: actix_web::web::Data<crate::AppState>,
) -> Result<HttpResponse, ProjectServiceError> {
    let project_id =
        ObjectId::parse_str(id.into_inner()).map_err(|_| ProjectServiceError::ProjectNotFound)?;
    match data.project_service.find_by_id(project_id).await {
        Ok(project) => {
            let response = ApiResponse::success("Project fetched successfully", project);
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => Err(e),
    }
}
