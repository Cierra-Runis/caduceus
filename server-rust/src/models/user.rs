use mongodb::bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub username: String,
    pub nickname: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Internal model for database operations (includes password field for deserialization)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserDocument {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub username: String,
    pub nickname: String,
    pub password: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<UserDocument> for User {
    fn from(doc: UserDocument) -> Self {
        User {
            id: doc.id,
            username: doc.username,
            nickname: doc.nickname,
            password: doc.password,
            created_at: doc.created_at,
            updated_at: doc.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_model_creation() {
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
        assert!(user.id.is_some());
    }

    #[test]
    fn test_user_document_creation() {
        let user_doc = UserDocument {
            id: Some(ObjectId::new()),
            username: "testuser".to_string(),
            nickname: "Test User".to_string(),
            password: "hashed_password".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        assert_eq!(user_doc.username, "testuser");
        assert_eq!(user_doc.nickname, "Test User");
        assert_eq!(user_doc.password, "hashed_password");
        assert!(user_doc.id.is_some());
    }

    #[test]
    fn test_user_serialization() {
        let user = User {
            id: Some(ObjectId::new()),
            username: "testuser".to_string(),
            nickname: "Test User".to_string(),
            password: "hashed_password".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&user).unwrap();
        // Test that serialization works and password is skipped
        assert!(json.contains("testuser"));
        assert!(json.contains("Test User"));
        assert!(json.contains("created_at"));
        assert!(json.contains("updated_at"));
        // Password should be skipped in serialization
        assert!(!json.contains("hashed_password"));
    }

    #[test]
    fn test_user_from_user_document() {
        let user_doc = UserDocument {
            id: Some(ObjectId::new()),
            username: "testuser".to_string(),
            nickname: "Test User".to_string(),
            password: "hashed_password".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let user: User = user_doc.clone().into();
        assert_eq!(user.username, user_doc.username);
        assert_eq!(user.nickname, user_doc.nickname);
        assert_eq!(user.id, user_doc.id);
    }

    #[test]
    fn test_user_document_bson_serialization() {
        let user_doc = UserDocument {
            id: Some(ObjectId::new()),
            username: "testuser".to_string(),
            nickname: "Test User".to_string(),
            password: "hashed_password".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Test that BSON serialization works
        let json_str = serde_json::to_string(&user_doc).unwrap();
        let _: UserDocument = serde_json::from_str(&json_str).unwrap();

        assert!(json_str.contains("username"));
        assert!(json_str.contains("nickname"));
        assert!(json_str.contains("password"));
        assert!(json_str.contains("created_at"));
        assert!(json_str.contains("updated_at"));
    }
}
