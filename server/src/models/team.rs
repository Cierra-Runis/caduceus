use bson::oid::ObjectId;
use bson::serde_helpers::time_0_3_offsetdatetime_as_bson_datetime;
use serde::{Deserialize, Serialize};
use time::serde::rfc3339;
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Team {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub creator_id: ObjectId,
    pub member_ids: Vec<ObjectId>,
    #[serde(with = "time_0_3_offsetdatetime_as_bson_datetime")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time_0_3_offsetdatetime_as_bson_datetime")]
    pub updated_at: OffsetDateTime,
}

#[derive(Serialize)]
pub struct TeamPayload {
    pub id: String,
    pub name: String,
    pub creator_id: String,
    pub member_ids: Vec<String>,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl From<Team> for TeamPayload {
    fn from(team: Team) -> Self {
        TeamPayload {
            id: team.id.to_hex(),
            name: team.name,
            creator_id: team.creator_id.to_hex(),
            member_ids: team.member_ids.iter().map(|id| id.to_hex()).collect(),
            created_at: team.created_at,
            updated_at: team.updated_at,
        }
    }
}
