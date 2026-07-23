use actix_multipart::Multipart;
use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use bson::oid::ObjectId;
use bson::serde_helpers::serialize_object_id_as_hex_string;
use derive_more::Display;
use futures_util::StreamExt as _;
use serde::{Deserialize, Serialize};

use crate::{
    models::{
        project::{FileContent, OwnerType},
        response::ApiResponse,
        user::UserClaims,
    },
    services::project::{ProjectServiceError, UploadedFile},
    storage::StorageError,
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
            | ProjectServiceError::ProjectNotFound
            | ProjectServiceError::FileNotFound => StatusCode::NOT_FOUND,
            ProjectServiceError::AccessDenied
            | ProjectServiceError::CreatorNotMatchOwner
            | ProjectServiceError::CreatorNotMemberOfTeam => StatusCode::FORBIDDEN,
            ProjectServiceError::InvalidOwnerType | ProjectServiceError::InvalidPath(_) => {
                StatusCode::BAD_REQUEST
            }
            ProjectServiceError::PathConflict(_) => StatusCode::CONFLICT,
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

/// Clone a project the caller can access into a new, independent project.
/// Access is enforced inside `ProjectService::duplicate` itself (mirroring
/// `update_file`), so there is no separate check here.
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

#[derive(Deserialize, Serialize)]
pub struct UpdateProjectRequest {
    #[serde(serialize_with = "serialize_object_id_as_hex_string")]
    pub owner_id: ObjectId,
    pub owner_type: OwnerType,
    pub name: String,
}

/// Update a project's metadata (rename / move between owners). Access and
/// target-owner validation live in `ProjectService::update`.
pub async fn update(
    id: actix_web::web::Path<String>,
    req: actix_web::web::Json<UpdateProjectRequest>,
    data: actix_web::web::Data<crate::AppState>,
    user: UserClaims,
) -> Result<HttpResponse, ProjectServiceError> {
    let project_id =
        ObjectId::parse_str(id.into_inner()).map_err(|_| ProjectServiceError::ProjectNotFound)?;

    match data
        .project_service
        .update(
            project_id,
            user.sub,
            req.name.clone(),
            req.owner_id,
            req.owner_type.clone(),
        )
        .await
    {
        Ok(project) => {
            let response = ApiResponse::success("Project updated successfully", project);
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

/// Upload caps. Generous for document assets (images, fonts); mostly a guard
/// against a single request exhausting memory, since uploads are buffered to
/// validate the whole batch before anything is written.
const MAX_FILE_BYTES: usize = 50 * 1024 * 1024;
const MAX_TOTAL_BYTES: usize = 200 * 1024 * 1024;
const MAX_FILES_PER_UPLOAD: usize = 100;

/// Errors for the binary-asset handlers, which combine metadata (service),
/// object storage, and multipart parsing failures under one HTTP mapping.
#[derive(Debug, Display)]
pub enum FileError {
    #[display("{_0}")]
    Service(ProjectServiceError),
    #[display("Storage error: {_0}")]
    Storage(StorageError),
    #[display("Malformed upload: {_0}")]
    Multipart(String),
    #[display("Upload is too large")]
    TooLarge,
    #[display("The requested file has no downloadable content")]
    NotBinary,
}

impl From<ProjectServiceError> for FileError {
    fn from(e: ProjectServiceError) -> Self {
        FileError::Service(e)
    }
}
impl From<StorageError> for FileError {
    fn from(e: StorageError) -> Self {
        FileError::Storage(e)
    }
}

impl ResponseError for FileError {
    fn error_response(&self) -> HttpResponse {
        // Delegate to the service error's own response for that arm so error
        // messages/status stay consistent with the rest of the project API.
        if let FileError::Service(e) = self {
            return e.error_response();
        }
        HttpResponse::build(self.status_code()).json(ApiResponse::error(&self.to_string()))
    }

    fn status_code(&self) -> StatusCode {
        match self {
            FileError::Service(e) => e.status_code(),
            FileError::Storage(StorageError::NotFound) | FileError::NotBinary => {
                StatusCode::NOT_FOUND
            }
            FileError::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
            FileError::Multipart(_) => StatusCode::BAD_REQUEST,
            FileError::TooLarge => StatusCode::PAYLOAD_TOO_LARGE,
        }
    }
}

fn parse_project_id(id: String) -> Result<ObjectId, ProjectServiceError> {
    ObjectId::parse_str(id).map_err(|_| ProjectServiceError::ProjectNotFound)
}
fn parse_file_id(id: String) -> Result<ObjectId, ProjectServiceError> {
    ObjectId::parse_str(id).map_err(|_| ProjectServiceError::FileNotFound)
}

#[derive(Deserialize, Serialize)]
pub struct PathRequest {
    pub path: String,
}

/// Create a new empty text file at the given path.
pub async fn create_file(
    id: actix_web::web::Path<String>,
    body: actix_web::web::Json<PathRequest>,
    data: actix_web::web::Data<crate::AppState>,
    user: UserClaims,
) -> Result<HttpResponse, ProjectServiceError> {
    let project_id = parse_project_id(id.into_inner())?;
    let payload = data
        .project_service
        .create_file(project_id, user.sub, body.path.clone())
        .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success("File created successfully", payload)))
}

/// Create a new (empty) directory at the given path.
pub async fn create_folder(
    id: actix_web::web::Path<String>,
    body: actix_web::web::Json<PathRequest>,
    data: actix_web::web::Data<crate::AppState>,
    user: UserClaims,
) -> Result<HttpResponse, ProjectServiceError> {
    let project_id = parse_project_id(id.into_inner())?;
    let payload = data
        .project_service
        .create_directory(project_id, user.sub, body.path.clone())
        .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success("Folder created successfully", payload)))
}

/// Rename or move a single file.
pub async fn rename_file(
    path: actix_web::web::Path<(String, String)>,
    body: actix_web::web::Json<PathRequest>,
    data: actix_web::web::Data<crate::AppState>,
    user: UserClaims,
) -> Result<HttpResponse, ProjectServiceError> {
    let (id, file_id) = path.into_inner();
    let project_id = parse_project_id(id)?;
    let file_id = parse_file_id(file_id)?;
    let payload = data
        .project_service
        .rename_file(project_id, user.sub, file_id, body.path.clone())
        .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success("File renamed successfully", payload)))
}

/// Delete a single file, freeing its object-storage bytes if it is a binary.
pub async fn delete_file(
    path: actix_web::web::Path<(String, String)>,
    data: actix_web::web::Data<crate::AppState>,
    user: UserClaims,
) -> Result<HttpResponse, FileError> {
    let (id, file_id) = path.into_inner();
    let project_id = parse_project_id(id)?;
    let file_id = parse_file_id(file_id)?;

    let removed = data
        .project_service
        .delete_file(project_id, user.sub, file_id)
        .await?;

    if let FileContent::Binary { storage_key } = removed.content {
        // Best-effort: the metadata row is already gone, so a storage hiccup
        // just leaves an orphan blob rather than a dangling reference.
        let _ = data
            .object_store
            .delete(&object_key(project_id, storage_key))
            .await;
    }

    Ok(HttpResponse::Ok().json(ApiResponse::<()>::success_no_payload(
        "File deleted successfully",
    )))
}

/// Delete a directory and everything under it, freeing every binary asset's
/// bytes.
pub async fn delete_folder(
    id: actix_web::web::Path<String>,
    body: actix_web::web::Json<PathRequest>,
    data: actix_web::web::Data<crate::AppState>,
    user: UserClaims,
) -> Result<HttpResponse, FileError> {
    let project_id = parse_project_id(id.into_inner())?;
    let removed_keys = data
        .project_service
        .delete_directory(project_id, user.sub, body.path.clone())
        .await?;

    for storage_key in removed_keys {
        let _ = data
            .object_store
            .delete(&object_key(project_id, storage_key))
            .await;
    }

    Ok(HttpResponse::Ok().json(ApiResponse::<()>::success_no_payload(
        "Folder deleted successfully",
    )))
}

#[derive(Deserialize, Serialize)]
pub struct SetEntryRequest {
    pub file_id: String,
}

/// Set the project's compile entry to an existing file.
pub async fn set_entry(
    id: actix_web::web::Path<String>,
    body: actix_web::web::Json<SetEntryRequest>,
    data: actix_web::web::Data<crate::AppState>,
    user: UserClaims,
) -> Result<HttpResponse, ProjectServiceError> {
    let project_id = parse_project_id(id.into_inner())?;
    let file_id = parse_file_id(body.file_id.clone())?;
    let payload = data
        .project_service
        .set_entry(project_id, user.sub, file_id)
        .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success("Entry updated successfully", payload)))
}

/// Serve a binary asset's raw bytes. Text files are not served here — their
/// content ships inline with the project detail payload.
pub async fn download_file(
    path: actix_web::web::Path<(String, String)>,
    data: actix_web::web::Data<crate::AppState>,
    user: UserClaims,
) -> Result<HttpResponse, FileError> {
    let (id, file_id) = path.into_inner();
    let project_id = parse_project_id(id)?;
    let file_id = parse_file_id(file_id)?;

    // Access check via the service, which also gives us the file's path/kind.
    let (storage_key, file_path) = data
        .project_service
        .file_for_download(project_id, user.sub, file_id)
        .await?
        .ok_or(FileError::NotBinary)?;

    let bytes = data
        .object_store
        .get(&object_key(project_id, storage_key))
        .await?;
    Ok(HttpResponse::Ok()
        .content_type(guess_content_type(&file_path))
        .body(bytes))
}

/// Uploaded bytes are stored inline as editable text when they are a UTF-8
/// source file (`.md`, `.typ`, `.bib`, …) rather than a binary asset. Cap kept
/// well under MongoDB's 16 MB document limit, since inline text lives in the
/// project document; larger "text" files fall back to object storage.
const INLINE_TEXT_MAX_BYTES: usize = 1024 * 1024;

/// Whether uploaded bytes should be stored inline as a text source: valid
/// UTF-8, no NUL byte (the strongest binary tell), and small enough to inline.
fn as_inline_text(bytes: &[u8]) -> Option<String> {
    if bytes.len() > INLINE_TEXT_MAX_BYTES || bytes.contains(&0) {
        return None;
    }
    std::str::from_utf8(bytes).ok().map(str::to_string)
}

/// Upload one or more files (proxied through the server, which authenticates
/// and validates every target path before a single byte is stored). Text
/// sources are inlined as editable, collaborative files; only true binaries go
/// to object storage. Each multipart part's *field name* carries the target
/// path relative to the project root.
pub async fn upload_files(
    id: actix_web::web::Path<String>,
    mut payload: Multipart,
    data: actix_web::web::Data<crate::AppState>,
    user: UserClaims,
) -> Result<HttpResponse, FileError> {
    let project_id = parse_project_id(id.into_inner())?;

    // Buffer the whole batch first so validation is all-or-nothing: a single
    // bad path rejects the request without leaving half the files uploaded.
    let mut staged: Vec<(String, Vec<u8>, String)> = Vec::new();
    let mut total: usize = 0;

    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| FileError::Multipart(e.to_string()))?;
        let target_path = field.name().unwrap_or_default().to_string();
        if target_path.is_empty() {
            return Err(FileError::Multipart(
                "each file part must name its target path".to_string(),
            ));
        }
        let content_type = field
            .content_type()
            .map(|m| m.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());

        let mut bytes: Vec<u8> = Vec::new();
        while let Some(chunk) = field.next().await {
            let chunk = chunk.map_err(|e| FileError::Multipart(e.to_string()))?;
            bytes.extend_from_slice(&chunk);
            if bytes.len() > MAX_FILE_BYTES {
                return Err(FileError::TooLarge);
            }
            total += chunk.len();
            if total > MAX_TOTAL_BYTES {
                return Err(FileError::TooLarge);
            }
        }
        staged.push((target_path, bytes, content_type));
        if staged.len() > MAX_FILES_PER_UPLOAD {
            return Err(FileError::TooLarge);
        }
    }

    if staged.is_empty() {
        return Err(FileError::Multipart("no files in upload".to_string()));
    }

    // Validate every target path against the current tree *before* touching
    // object storage, so a rejected upload never orphans a blob.
    let paths: Vec<String> = staged.iter().map(|(p, _, _)| p.clone()).collect();
    data.project_service
        .check_can_add_files(project_id, user.sub, &paths)
        .await?;

    // Classify each file: inline UTF-8 text, or a binary written to object
    // storage. Track written keys so we can roll them back if a later put fails.
    let mut uploads: Vec<UploadedFile> = Vec::with_capacity(staged.len());
    let mut written: Vec<ObjectId> = Vec::new();
    for (path, bytes, content_type) in staged {
        let size = bytes.len() as i64;
        let content = if let Some(text) = as_inline_text(&bytes) {
            FileContent::Text { text }
        } else {
            let storage_key = ObjectId::new();
            let key = object_key(project_id, storage_key);
            if let Err(e) = data.object_store.put(&key, &bytes, &content_type).await {
                for k in &written {
                    let _ = data.object_store.delete(&object_key(project_id, *k)).await;
                }
                return Err(FileError::Storage(e));
            }
            written.push(storage_key);
            FileContent::Binary { storage_key }
        };
        uploads.push(UploadedFile { path, content, size });
    }

    match data
        .project_service
        .add_uploaded_files(project_id, user.sub, uploads)
        .await
    {
        Ok(payloads) => Ok(HttpResponse::Ok()
            .json(ApiResponse::success("Files uploaded successfully", payloads))),
        Err(e) => {
            // Metadata write failed after bytes landed: roll the blobs back.
            for k in &written {
                let _ = data.object_store.delete(&object_key(project_id, *k)).await;
            }
            Err(FileError::Service(e))
        }
    }
}

/// The object-storage key for a project's binary asset. Namespacing by project
/// keeps a future per-project export / GitHub-sync a simple prefix listing.
fn object_key(project_id: ObjectId, storage_key: ObjectId) -> String {
    format!("projects/{}/{}", project_id.to_hex(), storage_key.to_hex())
}

/// Best-effort content type from a file extension, for serving assets back to
/// the browser. Unknown extensions fall back to a generic binary type.
fn guess_content_type(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "bmp" => "image/bmp",
        "ico" => "image/x-icon",
        "pdf" => "application/pdf",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "txt" | "csv" | "bib" | "csl" | "typ" => "text/plain; charset=utf-8",
        "json" => "application/json",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use actix_web::body::to_bytes;

    #[test]
    fn test_project_service_error_status_codes() {
        assert_eq!(
            ProjectServiceError::UserNotFound.status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            ProjectServiceError::OwnerNotFound(OwnerType::User).status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            ProjectServiceError::OwnerNotFound(OwnerType::Team).status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            ProjectServiceError::ProjectNotFound.status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            ProjectServiceError::AccessDenied.status_code(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            ProjectServiceError::CreatorNotMatchOwner.status_code(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            ProjectServiceError::CreatorNotMemberOfTeam.status_code(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            ProjectServiceError::InvalidOwnerType.status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            ProjectServiceError::Database(mongodb::error::Error::custom("boom")).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[actix_web::test]
    async fn test_project_service_error_response_body() {
        let resp = ProjectServiceError::AccessDenied.error_response();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        let body = to_bytes(resp.into_body()).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            json["message"],
            "Access denied: You do not have permission to access this project"
        );
        assert_eq!(json["payload"], serde_json::Value::Null);
    }
}
