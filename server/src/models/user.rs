use bson::serde_helpers::chrono_datetime_as_bson_datetime;
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{errors, Algorithm, EncodingKey, Header};
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserClaims {
    pub sub: String,
    pub exp: u64,
    pub iat: u64,
}

impl UserClaims {
    pub fn new(user_id: String, now: DateTime<Utc>, ttl: Duration) -> Self {
        UserClaims {
            sub: user_id,
            exp: (now + ttl).timestamp() as u64,
            iat: now.timestamp() as u64,
        }
    }

    pub fn generate(&self, secret: String) -> Result<String, errors::Error> {
        let header = Header {
            alg: Algorithm::HS512,
            ..Default::default()
        };
        jsonwebtoken::encode(&header, &self, &EncodingKey::from_secret(secret.as_ref()))
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_user_payload_conversion() {
        let user = User {
            id: ObjectId::new(),
            username: "test_user".to_string(),
            nickname: "Test User".to_string(),
            password: "hashed_password".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let payload: UserPayload = user.clone().into();
        assert_eq!(payload.id, user.id.to_hex());
        assert_eq!(payload.username, user.username);
        assert_eq!(payload.nickname, user.nickname);
        assert_eq!(payload.created_at, user.created_at);
        assert_eq!(payload.updated_at, user.updated_at);
    }
}
