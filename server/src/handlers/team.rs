use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::models::team::Team;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTeamRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct TeamResponse {
    pub id: String,
    pub name: String,
    pub creator_id: String,
    pub member_ids: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Team> for TeamResponse {
    fn from(team: Team) -> Self {
        Self {
            id: team.id.map(|id| id.to_hex()).unwrap_or_default(),
            name: team.name,
            creator_id: team.creator_id.to_hex(),
            member_ids: team.member_ids.into_iter().map(|id| id.to_hex()).collect(),
            created_at: team.created_at,
            updated_at: team.updated_at,
        }
    }
}
