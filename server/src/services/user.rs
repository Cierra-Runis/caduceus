use bcrypt::{non_truncating_hash, DEFAULT_COST};
use bson::oid::ObjectId;

use crate::models::user::{User, UserPayload};
use crate::repo::user::UserRepo;

pub struct UserService<R: UserRepo> {
    pub repo: R,
}

#[derive(Debug)]
pub enum UserServiceError {
    UserAlreadyExists,
    InternalError { details: String },
}

impl<R: UserRepo> UserService<R> {
    pub async fn register(
        &self,
        username: String,
        password: String,
    ) -> Result<UserPayload, UserServiceError> {
        let found_user = self.repo.find_by_username(&username).await;

        match found_user {
            Ok(Some(_)) => return Err(UserServiceError::UserAlreadyExists),
            Err(e) => {
                return Err(UserServiceError::InternalError {
                    details: format!("Database error: {:?}", e),
                })
            }
            _ => {}
        }

        let hashed = non_truncating_hash(password, DEFAULT_COST);
        let password = match hashed {
            Ok(p) => p,
            Err(e) => {
                return Err(UserServiceError::InternalError {
                    details: format!("Password hashing error: {:?}", e),
                })
            }
        };

        let user = User {
            id: ObjectId::new(),
            username: username.clone(),
            nickname: username.clone(),
            password,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let created_user = self.repo.create(user).await;
        match created_user {
            Ok(u) => Ok(UserPayload {
                id: u.id.to_hex(),
                username: u.username,
                nickname: u.nickname,
                created_at: u.created_at,
                updated_at: u.updated_at,
            }),
            Err(e) => Err(UserServiceError::InternalError {
                details: format!("Database error: {:?}", e),
            }),
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_user_success() {
        let repo = crate::repo::user::MockUserRepo {
            users: std::sync::Mutex::new(vec![]),
        };
        let service = UserService { repo };

        let result = service
            .register("test_user".to_string(), "test_password".to_string())
            .await;

        assert!(result.is_ok());
        let user_payload = result.unwrap();
        assert_eq!(user_payload.username, "test_user");
    }

    #[tokio::test]
    async fn test_register_user_already_exists() {
        let repo = crate::repo::user::MockUserRepo {
            users: std::sync::Mutex::new(vec![User {
                id: ObjectId::new(),
                username: "existing_user".to_string(),
                nickname: "existing_user".to_string(),
                password: "hashed_password".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }]),
        };
        let service = UserService { repo };

        let result = service
            .register("existing_user".to_string(), "test_password".to_string())
            .await;

        assert!(matches!(result, Err(UserServiceError::UserAlreadyExists)));
    }
}
