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
    #[display("User not found")]
    UserNotFound,
    #[display("Password not matched")]
    PasswordNotMatched,
    #[display("User already exists")]
    UserAlreadyExists,
    #[display("Bcrypt error: {_0}")]
    Bcrypt(BcryptError),
    #[display("Jwt error: {_0}")]
    Jwt(jsonwebtoken::errors::Error),
    #[display("Database error: {_0}")]
    Database(mongodb::error::Error),
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

    pub async fn login(
        &self,
        username: String,
        password: String,
    ) -> Result<AuthPayload, UserServiceError> {
        let user = match self.repo.find_by_username(&username).await {
            Ok(Some(user)) => user,
            Ok(None) => return Err(UserServiceError::UserNotFound),
            Err(e) => return Err(UserServiceError::Database(e)),
        };

        if !bcrypt::verify(password, &user.password).map_err(UserServiceError::Bcrypt)? {
            return Err(UserServiceError::PasswordNotMatched);
        }

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

    #[tokio::test]
    async fn test_login_user_success() {
        let repo = MockUserRepo {
            users: Mutex::new(vec![User {
                id: ObjectId::new(),
                username: "test_user".to_string(),
                nickname: "test_user".to_string(),
                password: bcrypt::hash("test_password", DEFAULT_COST).unwrap(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }]),
        };
        let secret = "test_secret".to_string();
        let service = UserService { repo, secret };

        let result = service
            .login("test_user".to_string(), "test_password".to_string())
            .await;

        assert!(result.is_ok());
        let payload = result.unwrap();
        assert_eq!(payload.user.username, "test_user");
    }

    #[tokio::test]
    async fn test_login_user_not_found() {
        let repo = MockUserRepo {
            users: Mutex::new(vec![]),
        };
        let secret = "test_secret".to_string();
        let service = UserService { repo, secret };

        let result = service
            .login("nonexistent_user".to_string(), "test_password".to_string())
            .await;

        assert!(matches!(result, Err(UserServiceError::UserNotFound)));
    }

    #[tokio::test]
    async fn test_login_user_password_not_matched() {
        let repo = MockUserRepo {
            users: Mutex::new(vec![User {
                id: ObjectId::new(),
                username: "test_user".to_string(),
                nickname: "test_user".to_string(),
                password: bcrypt::hash("correct_password", DEFAULT_COST).unwrap(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }]),
        };
        let secret = "test_secret".to_string();
        let service = UserService { repo, secret };

        let result = service
            .login("test_user".to_string(), "wrong_password".to_string())
            .await;

        assert!(matches!(result, Err(UserServiceError::PasswordNotMatched)));
    }
}
