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

#[derive(Deserialize, Serialize, utoipa::ToSchema)]
pub struct CreateProjectRequest {
    // ObjectId crosses the wire as its 24-char hex form
    #[schema(value_type = String)]
    #[serde(serialize_with = "serialize_object_id_as_hex_string")]
    pub owner_id: ObjectId,
    pub owner_type: OwnerType,
    pub name: String,
}

#[utoipa::path(
    post,
    path = "/api/project",
    tag = "project",
    request_body = CreateProjectRequest,
    responses(
        (status = 200, description = "Project created", body = crate::openapi::ApiSuccess<crate::models::project::ProjectPayload>),
        (status = 400, description = "Invalid owner type", body = crate::openapi::ApiMessage),
        (status = 401, description = "Missing or invalid JWT", body = crate::openapi::ApiMessage),
        (status = 403, description = "Creator does not match owner or is not a team member", body = crate::openapi::ApiMessage),
        (status = 404, description = "Owner not found", body = crate::openapi::ApiMessage),
        (status = 500, description = "Internal error", body = crate::openapi::ApiMessage),
    )
)]
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

#[utoipa::path(
    get,
    path = "/api/project/{id}",
    tag = "project",
    params(
        ("id" = String, Path, description = "Project id (24-char hex ObjectId)"),
    ),
    responses(
        (status = 200, description = "Project with full file tree and inlined text content", body = crate::openapi::ApiSuccess<crate::models::project::ProjectDetailPayload>),
        (status = 401, description = "Missing or invalid JWT", body = crate::openapi::ApiMessage),
        (status = 403, description = "No access to this project", body = crate::openapi::ApiMessage),
        (status = 404, description = "Project not found", body = crate::openapi::ApiMessage),
        (status = 500, description = "Internal error", body = crate::openapi::ApiMessage),
    )
)]
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

/// Clone a project the caller can access into a new, independent project.
/// Access is enforced inside `ProjectService::duplicate` itself (mirroring
/// `update_file`), so there is no separate check here.
#[utoipa::path(
    post,
    path = "/api/project/{id}/duplicate",
    tag = "project",
    params(
        ("id" = String, Path, description = "Source project id (24-char hex ObjectId)"),
    ),
    responses(
        (status = 200, description = "The newly created copy", body = crate::openapi::ApiSuccess<crate::models::project::ProjectPayload>),
        (status = 401, description = "Missing or invalid JWT", body = crate::openapi::ApiMessage),
        (status = 403, description = "No access to the source project", body = crate::openapi::ApiMessage),
        (status = 404, description = "Project not found", body = crate::openapi::ApiMessage),
        (status = 500, description = "Internal error", body = crate::openapi::ApiMessage),
    )
)]
pub async fn duplicate(
    id: actix_web::web::Path<String>,
    data: actix_web::web::Data<crate::AppState>,
    user: UserClaims,
) -> Result<HttpResponse, ProjectServiceError> {
    let project_id =
        ObjectId::parse_str(id.into_inner()).map_err(|_| ProjectServiceError::ProjectNotFound)?;

    match data.project_service.duplicate(project_id, user.sub).await {
        Ok(project) => {
            let response = ApiResponse::success("Project duplicated successfully", project);
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => Err(e),
    }
}

#[derive(Deserialize, Serialize, utoipa::ToSchema)]
pub struct UpdateFileRequest {
    pub text: String,
}

#[utoipa::path(
    put,
    path = "/api/project/{id}/file/{file_id}",
    tag = "project",
    params(
        ("id" = String, Path, description = "Project id (24-char hex ObjectId)"),
        ("file_id" = String, Path, description = "File id (24-char hex ObjectId)"),
    ),
    request_body = UpdateFileRequest,
    responses(
        (status = 200, description = "File saved; returns the bumped version and timestamp", body = crate::openapi::ApiSuccess<crate::models::project::UpdateFilePayload>),
        (status = 401, description = "Missing or invalid JWT", body = crate::openapi::ApiMessage),
        (status = 403, description = "No access to this project", body = crate::openapi::ApiMessage),
        (status = 404, description = "Project or file not found", body = crate::openapi::ApiMessage),
        (status = 500, description = "Internal error", body = crate::openapi::ApiMessage),
    )
)]
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
