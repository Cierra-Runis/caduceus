use mongodb::error::Result;

use crate::models::team::Team;

#[async_trait::async_trait]
pub trait TeamRepo {
    async fn create(&self, team: Team) -> Result<Team>;
}

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
    }
}
