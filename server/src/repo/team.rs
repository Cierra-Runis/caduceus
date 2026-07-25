use bson::{doc, oid::ObjectId};
use futures_util::TryStreamExt;
use mongodb::error::Result;

use crate::models::team::Team;

#[async_trait::async_trait]
pub trait TeamRepo {
    async fn create(&self, team: Team) -> Result<Team>;
    async fn find_by_id(&self, id: ObjectId) -> Result<Option<Team>>;
    async fn list_by_member_id(&self, member_id: ObjectId) -> Result<Vec<Team>>;
}

#[derive(Clone)]
pub struct MongoTeamRepo {
    pub collection: mongodb::Collection<Team>,
}

#[async_trait::async_trait]
impl TeamRepo for MongoTeamRepo {
    async fn create(&self, team: Team) -> Result<Team> {
        let result = self.collection.insert_one(&team).await;
        match result {
            Ok(_) => Ok(team),
            Err(e) => Err(e),
        }
    }

    async fn find_by_id(&self, id: ObjectId) -> Result<Option<Team>> {
        let filter = doc! { "_id": id };
        self.collection.find_one(filter).await
    }

    async fn list_by_member_id(&self, member_id: ObjectId) -> Result<Vec<Team>> {
        let filter = doc! { "member_ids": member_id };
        let cursor = self.collection.find(filter).await?;
        let teams: Vec<Team> = cursor.try_collect().await?;
        Ok(teams)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod tests {
    use super::*;
    use crate::config;
    use std::sync::Mutex;

    #[derive(Default)]
    pub struct MockTeamRepo {
        pub teams: Mutex<Vec<Team>>,
    }

    #[async_trait::async_trait]
    impl TeamRepo for MockTeamRepo {
        async fn create(&self, team: Team) -> Result<Team> {
            let mut teams = self.teams.lock().unwrap();
            teams.push(team.clone());
            Ok(team)
        }

        async fn find_by_id(&self, id: ObjectId) -> Result<Option<Team>> {
            let teams = self.teams.lock().unwrap();
            for team in teams.iter() {
                if team.id == id {
                    return Ok(Some(team.clone()));
                }
            }
            Ok(None)
        }

        async fn list_by_member_id(&self, member_id: ObjectId) -> Result<Vec<Team>> {
            let teams = self.teams.lock().unwrap();
            let filtered_teams: Vec<Team> = teams
                .iter()
                .filter(|team| team.member_ids.contains(&member_id))
                .cloned()
                .collect();
            Ok(filtered_teams)
        }
    }

    async fn test_repo() -> MongoTeamRepo {
        let config = config::Config::load("config/test.yaml").unwrap();
        let client = mongodb::Client::with_uri_str(config.mongo_uri)
            .await
            .unwrap();
        MongoTeamRepo {
            collection: client
                .database(&config.db_name)
                .collection::<Team>("teams"),
        }
    }

    fn new_team(member_ids: Vec<ObjectId>) -> Team {
        Team {
            id: ObjectId::new(),
            name: format!("Test Team {}", ObjectId::new().to_hex()),
            avatar_uri: None,
            creator_id: ObjectId::new(),
            member_ids,
            created_at: time::OffsetDateTime::now_utc(),
            updated_at: time::OffsetDateTime::now_utc(),
        }
    }

    async fn cleanup(repo: &MongoTeamRepo, id: ObjectId) {
        let _ = repo.collection.delete_one(doc! { "_id": id }).await;
    }

    #[tokio::test]
    #[ignore = "requires a live MongoDB (provisioned in CI; run locally with cargo test -- --ignored)"]
    async fn test_create_and_find_by_id() {
        let repo = test_repo().await;
        let team = new_team(vec![]);

        let created = repo.create(team.clone()).await.unwrap();
        assert_eq!(created.id, team.id);
        assert_eq!(created.name, team.name);

        let found = repo.find_by_id(team.id).await.unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.id, team.id);
        assert_eq!(found.name, team.name);
        assert_eq!(found.creator_id, team.creator_id);

        cleanup(&repo, team.id).await;
    }

    #[tokio::test]
    #[ignore = "requires a live MongoDB (provisioned in CI; run locally with cargo test -- --ignored)"]
    async fn test_find_by_id_not_found() {
        let repo = test_repo().await;
        let found = repo.find_by_id(ObjectId::new()).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    #[ignore = "requires a live MongoDB (provisioned in CI; run locally with cargo test -- --ignored)"]
    async fn test_list_by_member_id_includes_matching_excludes_others() {
        let repo = test_repo().await;
        let member_id = ObjectId::new();
        let other_member_id = ObjectId::new();

        let with_member = new_team(vec![member_id, other_member_id]);
        let without_member = new_team(vec![other_member_id]);

        repo.create(with_member.clone()).await.unwrap();
        repo.create(without_member.clone()).await.unwrap();

        let found = repo.list_by_member_id(member_id).await.unwrap();
        assert!(found.iter().any(|t| t.id == with_member.id));
        assert!(!found.iter().any(|t| t.id == without_member.id));

        cleanup(&repo, with_member.id).await;
        cleanup(&repo, without_member.id).await;
    }

    #[tokio::test]
    #[ignore = "requires a live MongoDB (provisioned in CI; run locally with cargo test -- --ignored)"]
    async fn test_list_by_member_id_empty_when_no_match() {
        let repo = test_repo().await;
        let found = repo.list_by_member_id(ObjectId::new()).await.unwrap();
        assert!(found.is_empty());
    }
}
