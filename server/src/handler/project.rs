use actix_web::{HttpResponse, ResponseError, http::StatusCode};
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
            ProjectServiceError::AccessDenied
            | ProjectServiceError::CreatorNotMatchOwner
            | ProjectServiceError::CreatorNotMemberOfTeam => StatusCode::FORBIDDEN,
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
    user: UserClaims,
) -> Result<HttpResponse, ProjectServiceError> {
    let project_id =
        ObjectId::parse_str(id.into_inner()).map_err(|_| ProjectServiceError::ProjectNotFound)?;

    // Check if user has access to this project
    match data.project_service.accessible(project_id, user.sub).await {
        Ok(true) => {}
        Ok(false) => return Err(ProjectServiceError::AccessDenied),
        Err(e) => return Err(e),
    };

    match data.project_service.find_by_id(project_id).await {
        Ok(project) => {
            let response = ApiResponse::success("Project fetched successfully", project);
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => Err(e),
    }
}

#[derive(Deserialize, Serialize)]
pub struct UpdateFileRequest {
    pub text: String,
}

pub async fn update_file(
    path: actix_web::web::Path<(String, String)>,
    body: actix_web::web::Json<UpdateFileRequest>,
    data: actix_web::web::Data<crate::AppState>,
    user: UserClaims,
) -> Result<HttpResponse, ProjectServiceError> {
    let (id, file_id) = path.into_inner();
    let project_id = ObjectId::parse_str(id).map_err(|_| ProjectServiceError::ProjectNotFound)?;
    let file_id = ObjectId::parse_str(file_id).map_err(|_| ProjectServiceError::ProjectNotFound)?;

    match data
        .project_service
        .update_file(project_id, user.sub, file_id, body.text.clone())
        .await
    {
        Ok(payload) => {
            let response = ApiResponse::success("File updated successfully", payload);
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => Err(e),
    }
}
