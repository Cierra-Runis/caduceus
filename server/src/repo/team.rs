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
    use std::sync::Mutex;

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
}
