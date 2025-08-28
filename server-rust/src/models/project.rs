use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub owner_id: ObjectId,
    pub owner_type: String, // "USER" or "TEAM"
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_model_creation() {
        let project = Project {
            id: Some(ObjectId::new()),
            name: "Test Project".to_string(),
            owner_id: ObjectId::new(),
            owner_type: "USER".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        assert_eq!(project.name, "Test Project");
        assert_eq!(project.owner_type, "USER");
        assert!(project.id.is_some());
    }

    #[test]
    fn test_project_serialization() {
        let project = Project {
            id: Some(ObjectId::new()),
            name: "Test Project".to_string(),
            owner_id: ObjectId::new(),
            owner_type: "USER".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&project).unwrap();
        assert!(json.contains("Test Project"));
        assert!(json.contains("USER"));
        assert!(json.contains("owner_id"));
    }
}
