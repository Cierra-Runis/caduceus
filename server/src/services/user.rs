use actix_web::body::BoxBody;
use actix_web::error::ResponseError;
use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use bcrypt::BcryptError;
use bcrypt::{non_truncating_hash, DEFAULT_COST};
use bson::oid::ObjectId;
use chrono::Utc;
use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::models::user::{User, UserClaims, UserPayload};
use crate::repo::user::UserRepo;

pub struct UserService<R: UserRepo> {
    pub repo: R,
    pub secret: String,
}

#[derive(Debug, Display)]
pub enum UserServiceError {
    #[display("User already exists")]
    UserAlreadyExists,
    #[display("Bcrypt error")]
    Bcrypt(BcryptError),
    #[display("Jwt error")]
    Jwt(jsonwebtoken::errors::Error),
    #[display("Database error")]
    Database(mongodb::error::Error),
    #[display("Internal server error: {details}")]
    Internal { details: String },
}

#[derive(Serialize)]
struct Response {
    message: String,
}

impl ResponseError for UserServiceError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(self.status_code()).json(Response {
            message: self.to_string(),
        })
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            UserServiceError::UserAlreadyExists => StatusCode::CONFLICT,
            UserServiceError::Bcrypt { .. }
            | UserServiceError::Jwt { .. }
            | UserServiceError::Database { .. }
            | UserServiceError::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AuthPayload {
    pub user: UserPayload,
    pub token: String,
}

impl<R: UserRepo> UserService<R> {
    pub async fn register(
        &self,
        username: String,
        password: String,
    ) -> Result<AuthPayload, UserServiceError> {
        match self.repo.find_by_username(&username).await {
            Ok(Some(_)) => return Err(UserServiceError::UserAlreadyExists),
            Ok(None) => {}
            Err(e) => return Err(UserServiceError::Database(e)),
        }

        let hashed_password =
            non_truncating_hash(password, DEFAULT_COST).map_err(UserServiceError::Bcrypt)?;

        let user = self
            .repo
            .create(User {
                id: ObjectId::new(),
                username: username.clone(),
                nickname: username.clone(),
                password: hashed_password,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
            .await
            .map_err(UserServiceError::Database)?;

        let claims = UserClaims::new(user.id.to_hex(), Utc::now(), chrono::Duration::hours(24));
        let token = claims
            .generate(self.secret.clone())
            .map_err(UserServiceError::Jwt)?;

        Ok(AuthPayload {
            user: user.into(),
            token,
        })
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {

    use super::*;
    use crate::repo::user::tests::MockUserRepo;
    use std::sync::Mutex;

    #[tokio::test]
    async fn test_register_user_success() {
        let repo = MockUserRepo {
            users: Mutex::new(vec![]),
        };
        let secret = "test_secret".to_string();
        let service = UserService { repo, secret };

        let result = service
            .register("test_user".to_string(), "test_password".to_string())
            .await;

        assert!(result.is_ok());
        let payload = result.unwrap();
        assert_eq!(payload.user.username, "test_user");
    }

    #[tokio::test]
    async fn test_register_user_already_exists() {
        let repo = MockUserRepo {
            users: Mutex::new(vec![User {
                id: ObjectId::new(),
                username: "existing_user".to_string(),
                nickname: "existing_user".to_string(),
                password: "hashed_password".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }]),
        };
        let secret = "test_secret".to_string();
        let service = UserService { repo, secret };

        let result = service
            .register("existing_user".to_string(), "test_password".to_string())
            .await;

        assert!(matches!(result, Err(UserServiceError::UserAlreadyExists)));
    }

    #[tokio::test]
    async fn test_register_user_bcrypt_error() {
        let repo = MockUserRepo {
            users: Mutex::new(vec![]),
        };
        let secret = "test_secret".to_string();
        let service = UserService { repo, secret };

        let long_password = "a".repeat(1000);
        let result = service
            .register("test_user".to_string(), long_password)
            .await;
        assert!(matches!(
            result,
            Err(UserServiceError::Bcrypt(BcryptError::Truncation(1001)))
        ));
    }
}
