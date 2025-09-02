use bson::oid::ObjectId;
use derive_more::Display;
use serde::Serialize;

use crate::{
    models::team::Team,
    repo::{team::TeamRepo, user::UserRepo},
};

pub struct TeamService<R: TeamRepo, U: UserRepo> {
    pub user_repo: U,
    pub team_repo: R,
}

#[derive(Debug, Display)]
pub enum TeamServiceError {
    #[display("Invalid user ID")]
    InvalidUserId,
    #[display("User not found")]
    UserNotFound,
    #[display("Database error: {_0}")]
    Database(mongodb::error::Error),
}

#[derive(Serialize)]
pub struct TeamPayload {
    pub id: String,
    pub name: String,
}

impl<R: TeamRepo, U: UserRepo> TeamService<R, U> {
    pub async fn create(
        &self,
        creator_id: String,
        name: String,
    ) -> Result<TeamPayload, TeamServiceError> {
        let id = ObjectId::parse_str(&creator_id).map_err(|_| TeamServiceError::InvalidUserId)?;
        match self.user_repo.find_by_id(id).await {
            Ok(Some(_)) => {}
            Ok(None) => return Err(TeamServiceError::UserNotFound),
            Err(e) => return Err(TeamServiceError::Database(e)),
        };

        let team = self
            .team_repo
            .create(Team {
                id: ObjectId::new(),
                name: name.clone(),
                creator_id: id,
                member_ids: vec![id],
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            })
            .await
            .map_err(TeamServiceError::Database)?;

        Ok(TeamPayload {
            id: team.id.to_hex(),
            name: team.name,
        })
    }
}
