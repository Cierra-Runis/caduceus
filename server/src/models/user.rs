use bson::serde_helpers::serialize_object_id_as_hex_string;
use bson::serde_helpers::time_0_3_offsetdatetime_as_bson_datetime;
use jsonwebtoken::{errors, Algorithm, EncodingKey, Header};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use time::serde::rfc3339;
use time::{Duration, OffsetDateTime};

pub const FIELD_ID: &str = "_id";
pub const FIELD_USERNAME: &str = "username";
pub const FIELD_NICKNAME: &str = "nickname";
pub const FIELD_PASSWORD: &str = "password";
pub const FIELD_CREATED_AT: &str = "created_at";
pub const FIELD_UPDATED_AT: &str = "updated_at";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub username: String,
    pub nickname: String,
    pub password: String,
    pub avatar_uri: Option<String>,
    #[serde(with = "time_0_3_offsetdatetime_as_bson_datetime")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time_0_3_offsetdatetime_as_bson_datetime")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UserPayload {
    pub id: String,
    pub username: String,
    pub nickname: String,
    pub avatar_uri: Option<String>,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl From<User> for UserPayload {
    fn from(user: User) -> Self {
        UserPayload {
            id: user.id.to_hex(),
            username: user.username.clone(),
            nickname: user.nickname.clone(),
            avatar_uri: user.avatar_uri.clone(),
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserClaims {
    #[serde(serialize_with = "serialize_object_id_as_hex_string")]
    pub sub: ObjectId,
    pub exp: i64,
    pub iat: i64,
}

impl UserClaims {
    pub fn new(user_id: ObjectId, now: OffsetDateTime, ttl: Duration) -> Self {
        UserClaims {
            sub: user_id,
            exp: now.saturating_add(ttl).unix_timestamp(),
            iat: now.unix_timestamp(),
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
            avatar_uri: None,
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
        };

        let payload: UserPayload = user.clone().into();
        assert_eq!(payload.id, user.id.to_hex());
        assert_eq!(payload.username, user.username);
        assert_eq!(payload.nickname, user.nickname);
        assert_eq!(payload.avatar_uri, user.avatar_uri);
        assert_eq!(payload.created_at, user.created_at);
        assert_eq!(payload.updated_at, user.updated_at);
    }
}
