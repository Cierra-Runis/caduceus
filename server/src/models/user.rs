use bson::serde_helpers::chrono_datetime_as_bson_datetime;
use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub username: String,
    pub nickname: String,
    pub password: String,
    #[serde(with = "chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserPayload {
    pub id: String,
    pub username: String,
    pub nickname: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserPayload {
    fn from(user: User) -> Self {
        UserPayload {
            id: user.id.to_hex(),
            username: user.username.clone(),
            nickname: user.nickname.clone(),
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User {
            id: ObjectId::new(),
            username: "test_user".to_string(),
            nickname: "Test User".to_string(),
            password: "hashed_password".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(user.username, "test_user");
        assert_eq!(user.nickname, "Test User");
        assert_eq!(user.password, "hashed_password");
    }

    #[test]
    fn test_user_bson_serialization() {
        let user = User {
            id: ObjectId::new(),
            username: "test_user".to_string(),
            nickname: "Test User".to_string(),
            password: "hashed_password".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
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
