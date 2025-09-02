use bson::oid::ObjectId;
use bson::serde_helpers::chrono_datetime_as_bson_datetime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Team {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub description: String,
    pub creator_id: ObjectId,
    pub member_ids: Vec<ObjectId>,
    #[serde(with = "chrono_datetime_as_bson_datetime")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(with = "chrono_datetime_as_bson_datetime")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
