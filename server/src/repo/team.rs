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

    #[tokio::test]
    async fn test_mongo_team_repo() {
        let config = config::Config::load("config/test.yaml").unwrap();
        let repo = MongoTeamRepo {
            collection: mongodb::Client::with_uri_str(config.mongo_uri)
                .await
                .unwrap()
                .database("test_db")
                .collection::<Team>("teams"),
        };

        let team = Team {
            id: ObjectId::new(),
            name: "Test Team".to_string(),
            avatar_uri: None,
            creator_id: ObjectId::new(),
            member_ids: vec![],
            created_at: time::OffsetDateTime::now_utc(),
            updated_at: time::OffsetDateTime::now_utc(),
        };

        // Test create
        let created_team = repo.create(team.clone()).await.unwrap();
        assert_eq!(created_team.name, team.name);

        // Test find_by_id
        let found_team = repo.find_by_id(created_team.id).await.unwrap();
        assert!(found_team.is_some());
        assert_eq!(found_team.unwrap().name, team.name);

        // Test list_by_member_id
        let member_id = ObjectId::new();
        let found_teams = repo.list_by_member_id(member_id).await.unwrap();
        assert!(found_teams.is_empty());
    }
}
