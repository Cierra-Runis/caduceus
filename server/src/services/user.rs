use bson::oid::ObjectId;

use crate::models::user::User;
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
    ) -> Result<User, UserServiceError> {
        let found_user = self.repo.find_user_by_username(&username).await;

        match found_user {
            Ok(Some(_)) => return Err(UserServiceError::UserAlreadyExists),
            Err(e) => {
                return Err(UserServiceError::InternalError {
                    details: format!("Database error: {:?}", e),
                })
            }
            _ => {}
        }

        let user = User {
            id: ObjectId::new(),
            username: username.clone(),
            nickname: username.clone(),
            password,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.repo
            .create_user(user)
            .await
            .map_err(|e| UserServiceError::InternalError {
                details: format!("Failed to create user: {:?}", e),
            })
    }
}
