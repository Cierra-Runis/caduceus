use bson::oid::ObjectId;
use bson::serde_helpers::time_0_3_offsetdatetime_as_bson_datetime;
use derive_more::Display;
use semver::Version;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use time::serde::rfc3339;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Display)]
pub enum OwnerType {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "team")]
    Team,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub owner_id: ObjectId,
    pub owner_type: OwnerType,
    pub creator_id: ObjectId,
    pub files: Vec<ProjectFile>,
    #[serde(with = "time_0_3_offsetdatetime_as_bson_datetime")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time_0_3_offsetdatetime_as_bson_datetime")]
    pub updated_at: OffsetDateTime,
    pub preview: Option<ObjectId>, // ID of the previewing File
    pub pinned_version: Option<Version>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectFile {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    // Last Change, CURD
    // pub last_change: Struct,
    pub size: i64,
    pub version: i32,
}

#[derive(Serialize)]
pub struct ProjectPayload {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub owner_type: OwnerType,
    pub creator_id: String,
    pub files: Vec<ProjectFilePayload>,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "rfc3339")]
    pub updated_at: OffsetDateTime,
    pub preview: Option<String>,
    pub pinned_version: Option<Version>,
}

#[derive(Serialize)]
pub struct ProjectFilePayload {
    pub id: String,
    pub name: String,
    pub size: i64,
    pub version: i32,
}

impl From<Project> for ProjectPayload {
    fn from(project: Project) -> Self {
        ProjectPayload {
            id: project.id.to_hex(),
            name: project.name,
            owner_id: project.owner_id.to_hex(),
            owner_type: project.owner_type,
            files: project
                .files
                .into_iter()
                .map(ProjectFilePayload::from)
                .collect(),
            creator_id: project.creator_id.to_hex(),
            created_at: project.created_at,
            updated_at: project.updated_at,
            preview: project.preview.map(|id| id.to_hex()),
            pinned_version: project.pinned_version,
        }
    }
}

impl From<ProjectFile> for ProjectFilePayload {
    fn from(file: ProjectFile) -> Self {
        ProjectFilePayload {
            id: file.id.to_hex(),
            name: file.name,
            size: file.size,
            version: file.version,
        }
    }
}
