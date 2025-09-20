use crate::models::user::{self, User};
use bson::oid::ObjectId;
use mongodb::{bson::doc, error::Result};

#[async_trait::async_trait]
pub trait UserRepo {
    async fn find_by_id(&self, id: ObjectId) -> Result<Option<User>>;
    async fn find_by_username(&self, username: &str) -> Result<Option<User>>;
    async fn create(&self, user: User) -> Result<User>;
}

#[derive(Clone)]
pub struct MongoUserRepo {
    pub collection: mongodb::Collection<User>,
}

#[async_trait::async_trait]
impl UserRepo for MongoUserRepo {
    async fn find_by_id(&self, id: ObjectId) -> Result<Option<User>> {
        let filter = doc! { user::FIELD_ID: id };
        self.collection.find_one(filter).await
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>> {
        let filter = doc! { user::FIELD_USERNAME: username };
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
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod tests {
    use time::OffsetDateTime;

    use crate::config;

    use super::*;
    use std::sync::Mutex;

    pub struct MockUserRepo {
        pub users: Mutex<Vec<User>>,
    }

    #[async_trait::async_trait]
    impl UserRepo for MockUserRepo {
        async fn find_by_id(&self, id: ObjectId) -> Result<Option<User>> {
            let users = self.users.lock().unwrap();
            Ok(users.iter().find(|u| u.id == id).cloned())
        }

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

    #[tokio::test]
    async fn test_mongo_user_repo() {
        let config = config::Config::load("config/test.yaml").unwrap();
        let repo = MongoUserRepo {
            collection: mongodb::Client::with_uri_str(config.mongo_uri)
                .await
                .unwrap()
                .database("test_db")
                .collection("users"),
        };

        let user = User {
            id: ObjectId::new(),
            username: ObjectId::new().to_hex(),
            nickname: "Test User".to_string(),
            password: "hashed_password".to_string(),
            avatar_uri: None,
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
        };

        // Test create
        let created_user = repo.create(user.clone()).await.unwrap();
        assert_eq!(created_user.username, user.username);

        // Test find_by_id
        let found_user = repo.find_by_id(created_user.id).await.unwrap();
        assert!(found_user.is_some());
        assert_eq!(found_user.unwrap().username, user.username);

        // Test find_by_username
        let found_user = repo.find_by_username(&user.username).await.unwrap();
        assert!(found_user.is_some());
        assert_eq!(found_user.unwrap().id, user.id);
    }
}
