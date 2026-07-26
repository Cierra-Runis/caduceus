//! Uploading and fetching a project's binary blobs (images, fonts, …).
//!
//! A text file's bytes flow through the CRDT (a `Y.Text` flushed to a blob by
//! the room); a binary file has no text overlay, so the client uploads its bytes
//! here directly and then creates a file node referencing the returned hash.
//! Fetching is for previewing an image or downloading an asset.

use actix_web::web::Bytes;
use actix_web::{HttpResponse, ResponseError, http::StatusCode, web};
use bson::oid::ObjectId;
use derive_more::Display;
use serde::Serialize;

use crate::models::response::ApiResponse;
use crate::models::user::UserClaims;
use crate::storage::{ProjectStore, is_valid_sha256};

#[derive(Debug, Display)]
pub enum BlobError {
    #[display("Project not Found")]
    ProjectNotFound,
    #[display("Forbidden: You don't have access to this project")]
    Forbidden,
    #[display("Invalid blob hash")]
    InvalidHash,
    #[display("Blob not Found")]
    NotFound,
    #[display("Storage error")]
    Storage,
}

impl std::error::Error for BlobError {}

impl ResponseError for BlobError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ApiResponse::error(&self.to_string()))
    }
    fn status_code(&self) -> StatusCode {
        match self {
            BlobError::ProjectNotFound | BlobError::NotFound => StatusCode::NOT_FOUND,
            BlobError::Forbidden => StatusCode::FORBIDDEN,
            BlobError::InvalidHash => StatusCode::BAD_REQUEST,
            BlobError::Storage => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// The stored blob's reference, returned after an upload so the client can put
/// it on a file node.
#[derive(Serialize)]
pub struct BlobPayload {
    pub sha256: String,
    pub size: u64,
}

async fn require_access(
    data: &crate::AppState,
    project_id: ObjectId,
    user_sub: ObjectId,
) -> Result<(), BlobError> {
    match data.project_service.accessible(project_id, user_sub).await {
        Ok(true) => Ok(()),
        Ok(false) => Err(BlobError::Forbidden),
        Err(_) => Err(BlobError::ProjectNotFound),
    }
}

/// Store the request body as a content-addressed blob in the project and return
/// its sha256 + size.
pub async fn upload(
    id: web::Path<String>,
    body: Bytes,
    data: web::Data<crate::AppState>,
    store: web::Data<ProjectStore>,
    user: UserClaims,
) -> Result<HttpResponse, BlobError> {
    let project_id =
        ObjectId::parse_str(id.into_inner()).map_err(|_| BlobError::ProjectNotFound)?;
    require_access(&data, project_id, user.sub).await?;

    let blob = store
        .put_blob(&project_id.to_hex(), &body)
        .await
        .map_err(|_| BlobError::Storage)?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(
        "Blob uploaded successfully",
        BlobPayload {
            sha256: blob.sha256,
            size: blob.size,
        },
    )))
}

/// Fetch a blob's raw bytes (for an image preview or an asset download).
pub async fn download(
    path: web::Path<(String, String)>,
    data: web::Data<crate::AppState>,
    store: web::Data<ProjectStore>,
    user: UserClaims,
) -> Result<HttpResponse, BlobError> {
    let (id, sha256) = path.into_inner();
    let project_id = ObjectId::parse_str(id).map_err(|_| BlobError::ProjectNotFound)?;
    require_access(&data, project_id, user.sub).await?;
    if !is_valid_sha256(&sha256) {
        return Err(BlobError::InvalidHash);
    }

    match store.get_blob(&project_id.to_hex(), &sha256).await {
        Ok(Some(bytes)) => Ok(HttpResponse::Ok()
            .content_type("application/octet-stream")
            .body(bytes)),
        Ok(None) => Err(BlobError::NotFound),
        Err(_) => Err(BlobError::Storage),
    }
}
