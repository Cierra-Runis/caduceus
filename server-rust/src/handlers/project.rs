use axum::{
    extract::{Extension, State},
    Json,
};
use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    error::Result,
    middleware::Claims,
    models::{project::Project, response::Response},
    routes::AppState,
    services::project::ProjectService,
};

#[derive(Debug, Deserialize, Validate)]
pub struct CreateProjectRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub owner_type: String,       // "USER" or "TEAM"
    pub owner_id: Option<String>, // Optional, defaults to current user
}

#[derive(Debug, Serialize)]
pub struct ProjectResponse {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub owner_type: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Project> for ProjectResponse {
    fn from(project: Project) -> Self {
        Self {
            id: project.id.map(|id| id.to_hex()).unwrap_or_default(),
            name: project.name,
            owner_id: project.owner_id.to_hex(),
            owner_type: project.owner_type,
            created_at: project.created_at,
            updated_at: project.updated_at,
        }
    }
}

pub async fn create_project(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateProjectRequest>,
) -> Result<Json<Response<ProjectResponse>>> {
    // Validate input
    payload
        .validate()
        .map_err(|e| crate::error::AppError::Validation(format!("Validation error: {}", e)))?;

    let project_service = ProjectService::new(&state.database.db);

    // Determine owner ID
    let owner_id = if let Some(owner_id_str) = payload.owner_id {
        ObjectId::parse_str(&owner_id_str)?
    } else {
        ObjectId::parse_str(&claims.sub)?
    };

    let project = project_service
        .create_project(payload.name, owner_id, payload.owner_type)
        .await?;

    let response = Response {
        success: true,
        data: Some(project.into()),
        message: Some("Project created successfully".to_string()),
    };

    Ok(Json(response))
}
