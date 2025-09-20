use bson::oid::ObjectId;
use bson::serde_helpers::time_0_3_offsetdatetime_as_bson_datetime;
use derive_more::Display;
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
    #[serde(with = "time_0_3_offsetdatetime_as_bson_datetime")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time_0_3_offsetdatetime_as_bson_datetime")]
    pub updated_at: OffsetDateTime,
}

#[derive(Serialize)]
pub struct ProjectPayload {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub owner_type: OwnerType,
    pub creator_id: String,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl From<Project> for ProjectPayload {
    fn from(project: Project) -> Self {
        ProjectPayload {
            id: project.id.to_hex(),
            name: project.name,
            owner_id: project.owner_id.to_hex(),
            owner_type: project.owner_type,
            creator_id: project.creator_id.to_hex(),
            created_at: project.created_at,
            updated_at: project.updated_at,
        }
    }
}
