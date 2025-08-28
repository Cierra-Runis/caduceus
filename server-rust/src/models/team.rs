use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Team {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub creator_id: ObjectId,
    pub member_ids: Vec<ObjectId>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_team_model_creation() {
        let team = Team {
            id: Some(ObjectId::new()),
            name: "Test Team".to_string(),
            creator_id: ObjectId::new(),
            member_ids: vec![ObjectId::new(), ObjectId::new()],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        assert_eq!(team.name, "Test Team");
        assert_eq!(team.member_ids.len(), 2);
        assert!(team.id.is_some());
    }

    #[test]
    fn test_team_serialization() {
        let team = Team {
            id: Some(ObjectId::new()),
            name: "Test Team".to_string(),
            creator_id: ObjectId::new(),
            member_ids: vec![ObjectId::new(), ObjectId::new()],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&team).unwrap();
        assert!(json.contains("Test Team"));
        assert!(json.contains("creator_id"));
        assert!(json.contains("member_ids"));
    }
}
