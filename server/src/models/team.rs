use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Team {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub creator_id: ObjectId,
    pub member_ids: Vec<ObjectId>,
    pub created_at: DateTime<Utc>,
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

        // Test that serialization works
        let json_str = serde_json::to_string(&team).unwrap();
        let _: Team = serde_json::from_str(&json_str).unwrap();
    }
}
