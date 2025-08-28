use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub username: String,
    pub nickname: String,
    pub password: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User {
            id: Some(ObjectId::new()),
            username: "testuser".to_string(),
            nickname: "Test User".to_string(),
            password: "hashed_password".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        assert_eq!(user.username, "testuser");
        assert_eq!(user.nickname, "Test User");
        assert_eq!(user.password, "hashed_password");
        assert!(user.id.is_some());
    }

    #[test]
    fn test_user_bson_serialization() {
        let user = User {
            id: Some(ObjectId::new()),
            username: "testuser".to_string(),
            nickname: "Test User".to_string(),
            password: "hashed_password".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let json_str = serde_json::to_string(&user).unwrap();
        let _: User = serde_json::from_str(&json_str).unwrap();

        assert!(json_str.contains("username"));
        assert!(json_str.contains("nickname"));
        assert!(json_str.contains("password"));
        assert!(json_str.contains("created_at"));
        assert!(json_str.contains("updated_at"));
    }
}
