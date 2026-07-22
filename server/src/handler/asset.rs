use actix_web::http::{StatusCode, header};
use actix_web::{HttpRequest, HttpResponse, ResponseError, web};
use bson::oid::ObjectId;
use serde::Deserialize;

use crate::models::response::ApiResponse;
use crate::models::user::UserClaims;
use crate::services::asset::AssetServiceError;

impl ResponseError for AssetServiceError {
    fn error_response(&self) -> HttpResponse {
        let response = ApiResponse::error(&self.to_string());
        HttpResponse::build(self.status_code()).json(response)
    }

    fn status_code(&self) -> StatusCode {
        match self {
            AssetServiceError::ProjectNotFound | AssetServiceError::FileNotFound => {
                StatusCode::NOT_FOUND
            }
            AssetServiceError::AccessDenied => StatusCode::FORBIDDEN,
            AssetServiceError::NotBinary | AssetServiceError::InvalidPath => {
                StatusCode::BAD_REQUEST
            }
            AssetServiceError::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Deserialize)]
pub struct UploadAssetQuery {
    /// Virtual-FS path the asset takes within the project, e.g. `logo.png` or
    /// `images/figure.svg` — the same name a Typst `#image("…")` will resolve.
    pub path: String,
}

/// `POST /api/project/{id}/asset?path=<vfs-path>` — upload a binary asset.
///
/// The body is the raw file bytes and the `Content-Type` header carries the MIME
/// type, both stored so the asset can be served back verbatim. Returns the newly
/// created file's metadata (id, path, storage key).
pub async fn upload_asset(
    id: web::Path<String>,
    query: web::Query<UploadAssetQuery>,
    body: web::Bytes,
    req: HttpRequest,
    data: web::Data<crate::AppState>,
    user: UserClaims,
) -> Result<HttpResponse, AssetServiceError> {
    let project_id =
        ObjectId::parse_str(id.into_inner()).map_err(|_| AssetServiceError::ProjectNotFound)?;

    let content_type = req
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);

    let payload = data
        .asset_service
        .upload_asset(
            project_id,
            user.sub,
            query.into_inner().path,
            content_type,
            body.to_vec(),
        )
        .await?;

    let response = ApiResponse::success("Asset uploaded successfully", payload);
    Ok(HttpResponse::Ok().json(response))
}

/// `GET /api/project/{id}/asset/{file_id}` — stream a binary asset's bytes back
/// with its stored `Content-Type`. Access is enforced against the owning
/// project.
pub async fn get_asset(
    path: web::Path<(String, String)>,
    data: web::Data<crate::AppState>,
    user: UserClaims,
) -> Result<HttpResponse, AssetServiceError> {
    let (id, file_id) = path.into_inner();
    let project_id = ObjectId::parse_str(id).map_err(|_| AssetServiceError::ProjectNotFound)?;
    let file_id = ObjectId::parse_str(file_id).map_err(|_| AssetServiceError::FileNotFound)?;

    let asset = data
        .asset_service
        .read_asset(project_id, user.sub, file_id)
        .await?;

    let mut builder = HttpResponse::Ok();
    if let Some(content_type) = asset.content_type {
        builder.insert_header((header::CONTENT_TYPE, content_type));
    }
    Ok(builder.body(asset.bytes))
}
