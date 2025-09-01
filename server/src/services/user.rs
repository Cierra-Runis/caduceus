use bcrypt::{non_truncating_hash, DEFAULT_COST};
use bson::oid::ObjectId;

use crate::models::user::{User, UserPayload};
use crate::repo::user::UserRepo;

pub struct UserService<R: UserRepo> {
    pub repo: R,
}

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
