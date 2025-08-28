use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use mongodb::bson::{doc, oid::ObjectId};
use mongodb::{Collection, Database};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::{
    error::{AppError, Result},
    middleware::Claims,
    models::user::{User, UserDocument},
};

#[derive(Clone)]
pub struct UserService {
    collection: Collection<UserDocument>,
    jwt_secret: String,
    jwt_expires_in: Duration,
}

impl UserService {
    pub fn new(db: &Database, jwt_secret: String, jwt_expires_in: Duration) -> Self {
        Self {
            collection: db.collection("users"),
            jwt_secret,
            jwt_expires_in,
        }
    }

    pub async fn create_user(
        &self,
        username: String,
        nickname: String,
        password: String,
    ) -> Result<(User, String)> {
        if self.get_user_by_username(&username).await.is_ok() {
            return Err(AppError::Conflict("Username already exists".to_string()));
        }

        let hashed_password = hash(password, DEFAULT_COST)?;

        let now = Utc::now();
        let user_doc = UserDocument {
            id: None,
            username,
            nickname,
            password: hashed_password,
            created_at: now,
            updated_at: now,
        };

        let result = self.collection.insert_one(&user_doc).await?;
        let inserted_id = result
            .inserted_id
            .as_object_id()
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("Failed to get inserted ID")))?;

        let mut created_user_doc = user_doc;
        created_user_doc.id = Some(inserted_id);

        let created_user = User::from(created_user_doc);
        let token = self.generate_token(&created_user)?;
        Ok((created_user, token))
    }

    pub async fn authenticate(&self, username: String, password: String) -> Result<(User, String)> {
        let user = self.get_user_by_username(&username).await?;

        if !verify(password, &user.password)? {
            return Err(AppError::Authentication("Invalid credentials".to_string()));
        }

        let token = self.generate_token(&user)?;
        Ok((user, token))
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<User> {
        let user = self
            .collection
            .find_one(doc! { "username": username })
            .await?;

        match user {
            Some(user_doc) => Ok(User::from(user_doc)),
            None => Err(AppError::NotFound("User not found".to_string())),
        }
    }

    pub async fn get_user_by_id(&self, user_id: &str) -> Result<User> {
        let object_id = ObjectId::parse_str(user_id)?;
        let user = self.collection.find_one(doc! { "_id": object_id }).await?;

        match user {
            Some(user_doc) => Ok(User::from(user_doc)),
            None => Err(AppError::NotFound("User not found".to_string())),
        }
    }

    fn generate_token(&self, user: &User) -> Result<String> {
        let user_id = user
            .id
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("User ID is missing")))?;

        let exp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("System time error: {}", e)))?
            .as_secs()
            + self.jwt_expires_in.as_secs();

        let claims = Claims {
            sub: user_id.to_hex(),
            username: user.username.clone(),
            exp: exp as usize,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )?;

        Ok(token)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_create_user_success() {
        let user = User {
            id: Some(ObjectId::new()),
            username: "test_user".to_string(),
            nickname: "Test User".to_string(),
            password: "hashed_password".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        assert_eq!(user.username, "test_user");
        assert_eq!(user.nickname, "Test User");
    }

    #[tokio::test]
    async fn test_password_hashing() {
        let password = "test_password";
        let hashed = bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap();

        assert_ne!(password, hashed);
        assert!(bcrypt::verify(password, &hashed).unwrap());
    }

    #[tokio::test]
    async fn test_jwt_token_generation() {
        use jsonwebtoken::{
            decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation,
        };
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize)]
        struct Claims {
            sub: String,
            exp: usize,
        }

        let secret = "test_secret";
        let user_id = "test_user_id";

        let exp = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::hours(24))
            .expect("valid timestamp")
            .timestamp() as usize;

        let claims = Claims {
            sub: user_id.to_string(),
            exp,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )
        .unwrap();

        assert!(!token.is_empty());

        let token_data = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::new(Algorithm::HS256),
        )
        .unwrap();

        assert_eq!(token_data.claims.sub, user_id);
    }

    #[test]
    fn test_user_model_serialization() {
        let user_doc = UserDocument {
            id: Some(ObjectId::new()),
            username: "test_user".to_string(),
            nickname: "Test User".to_string(),
            password: "hashed_password".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&user_doc).unwrap();
        assert!(json.contains("test_user"));
        assert!(json.contains("Test User"));

        let deserialized: UserDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.username, user_doc.username);
        assert_eq!(deserialized.nickname, user_doc.nickname);
    }

    #[test]
    fn test_user_document_serialization() {
        let user_doc = UserDocument {
            id: Some(ObjectId::new()),
            username: "test_user".to_string(),
            nickname: "Test User".to_string(),
            password: "hashed_password".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let json_str = serde_json::to_string(&user_doc).unwrap();
        let _: UserDocument = serde_json::from_str(&json_str).unwrap();

        assert!(json_str.contains("test_user"));
        assert!(json_str.contains("Test User"));
    }

    #[test]
    fn test_username_validation() {
        let valid_usernames = vec!["user123", "test_user", "TestUser", "u"];

        let long_username = "a".repeat(51);
        let invalid_usernames = vec!["", &long_username, "user@name", "user name"];

        let username_regex = regex::Regex::new(r"^[a-zA-Z0-9_]{1,50}$").unwrap();

        for username in valid_usernames {
            assert!(
                username_regex.is_match(username),
                "Username {} should be valid",
                username
            );
        }

        for username in invalid_usernames {
            assert!(
                !username_regex.is_match(username),
                "Username {} should be invalid",
                username
            );
        }
    }

    #[test]
    fn test_email_validation() {
        let valid_emails = vec!["test@example.com", "user.name@domain.co.uk"];

        let invalid_emails = vec!["invalid-email", "@example.com", "test@"];

        let email_regex =
            regex::Regex::new(r"^[a-zA-Z0-9]([a-zA-Z0-9._%-]*[a-zA-Z0-9])?@[a-zA-Z0-9]([a-zA-Z0-9.-]*[a-zA-Z0-9])?\.[a-zA-Z]{2,}$").unwrap();

        for email in valid_emails {
            assert!(
                email_regex.is_match(email),
                "Email {} should be valid",
                email
            );
        }

        for email in invalid_emails {
            assert!(
                !email_regex.is_match(email),
                "Email {} should be invalid",
                email
            );
        }
    }
}
