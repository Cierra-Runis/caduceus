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

    #[derive(Default)]
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

    async fn test_repo() -> MongoUserRepo {
        let config = config::Config::load("config/test.yaml").unwrap();
        let client = mongodb::Client::with_uri_str(config.mongo_uri)
            .await
            .unwrap();
        MongoUserRepo {
            collection: client.database(&config.db_name).collection("users"),
        }
    }

    fn new_user() -> User {
        User {
            id: ObjectId::new(),
            username: format!("test_user_{}", ObjectId::new().to_hex()),
            nickname: "Test User".to_string(),
            password: "hashed_password".to_string(),
            avatar_uri: None,
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
        }
    }

    async fn cleanup(repo: &MongoUserRepo, id: ObjectId) {
        let _ = repo
            .collection
            .delete_one(doc! { user::FIELD_ID: id })
            .await;
    }

    #[tokio::test]
    async fn test_create_and_find_by_id() {
        let repo = test_repo().await;
        let user = new_user();

        let created = repo.create(user.clone()).await.unwrap();
        assert_eq!(created.id, user.id);
        assert_eq!(created.username, user.username);

        let found = repo.find_by_id(user.id).await.unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.id, user.id);
        assert_eq!(found.username, user.username);
        assert_eq!(found.nickname, user.nickname);
        assert_eq!(found.password, user.password);

        cleanup(&repo, user.id).await;
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let repo = test_repo().await;
        let found = repo.find_by_id(ObjectId::new()).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_create_and_find_by_username() {
        let repo = test_repo().await;
        let user = new_user();
        repo.create(user.clone()).await.unwrap();

        let found = repo.find_by_username(&user.username).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, user.id);

        cleanup(&repo, user.id).await;
    }

    #[tokio::test]
    async fn test_find_by_username_not_found() {
        let repo = test_repo().await;
        let found = repo
            .find_by_username(&format!("nonexistent_{}", ObjectId::new().to_hex()))
            .await
            .unwrap();
        assert!(found.is_none());
    }
}
