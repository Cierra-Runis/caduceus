use crate::models::user::User;
use mongodb::{bson::doc, error::Result};

#[async_trait::async_trait]
pub trait UserRepo {
    async fn find_by_username(&self, username: &str) -> Result<Option<User>>;
    async fn create(&self, user: User) -> Result<User>;
}

pub struct MongoUserRepo {
    pub collection: mongodb::Collection<User>,
}

#[async_trait::async_trait]
impl UserRepo for MongoUserRepo {
    async fn find_by_username(&self, username: &str) -> Result<Option<User>> {
        let filter = doc! { "username": username };
        self.collection.find_one(filter).await
    }

    async fn create(&self, user: User) -> Result<User> {
        let result = self.collection.insert_one(&user).await;
        match result {
            Ok(_) => Ok(user),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
pub struct MockUserRepo {
    pub users: std::sync::Mutex<Vec<User>>,
}

#[cfg(test)]
#[async_trait::async_trait]
impl UserRepo for MockUserRepo {
    async fn find_by_username(&self, username: &str) -> Result<Option<User>> {
        let users = self.users.lock().unwrap();
        Ok(users.iter().find(|u| u.username == username).cloned())
    }

    async fn create(&self, user: User) -> Result<User> {
        let mut users = self.users.lock().unwrap();
        users.push(user.clone());
        Ok(user)
    }
}
