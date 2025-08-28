use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::models::team::Team;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTeamRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    // pub member_ids: Vec<String>,
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

// pub async fn create_team(
//     State(state): State<AppState>,
//     Extension(claims): Extension<Claims>,
//     Json(payload): Json<CreateTeamRequest>,
// ) -> Result<Json<Response<TeamResponse>>> {
//     // Validate input
//     payload
//         .validate()
//         .map_err(|e| crate::error::AppError::Validation(format!("Validation error: {}", e)))?;

//     let team_service = TeamService::new(&state.database.db);

//     let creator_id = ObjectId::parse_str(&claims.sub)?;

//     // Parse member IDs
//     let mut member_ids = Vec::new();
//     for member_id_str in payload.member_ids {
//         member_ids.push(ObjectId::parse_str(&member_id_str)?);
//     }

//     let team = team_service
//         .create_team(payload.name, creator_id, member_ids)
//         .await?;

//     let response = Response {
//         success: true,
//         data: Some(team.into()),
//         message: Some("Team created successfully".to_string()),
//     };

//     Ok(Json(response))
// }
