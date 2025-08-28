use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub owner_id: ObjectId,
    pub owner_type: String, // "USER" or "TEAM"
    pub created_at: DateTime<Utc>,
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

        // Test that BSON serialization works for MongoDB
        let json_str = serde_json::to_string(&project).unwrap();
        let _: Project = serde_json::from_str(&json_str).unwrap();
    }
}
