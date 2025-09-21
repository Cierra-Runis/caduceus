use bson::oid::ObjectId;
use derive_more::Display;
use time::OffsetDateTime;

use crate::{
    models::{
        project::{OwnerType, ProjectPayload},
        team::{Team, TeamPayload},
    },
    repo::{project::ProjectRepo, team::TeamRepo, user::UserRepo},
};

pub struct TeamService<R: TeamRepo, U: UserRepo, P: ProjectRepo> {
    pub user_repo: U,
    pub team_repo: R,
    pub project_repo: P,
}

#[derive(Debug, Display)]
pub enum TeamServiceError {
    #[display("User not found")]
    UserNotFound,
    #[display("Database error: {_0}")]
    Database(mongodb::error::Error),
    #[display("Team not found")]
    TeamNotFound,
}

impl<R: TeamRepo, U: UserRepo, P: ProjectRepo> TeamService<R, U, P> {
    pub async fn create(
        &self,
        creator_id: ObjectId,
        name: String,
    ) -> Result<TeamPayload, TeamServiceError> {
        let creator = match self.user_repo.find_by_id(creator_id).await {
            Ok(Some(user)) => user,
            Ok(None) => return Err(TeamServiceError::UserNotFound),
            Err(e) => return Err(TeamServiceError::Database(e)),
        };

        let team = self
            .team_repo
            .create(Team {
                id: ObjectId::new(),
                name: name.clone(),
                avatar_uri: None,
                creator_id: creator.id,
                member_ids: vec![creator.id],
                created_at: OffsetDateTime::now_utc(),
                updated_at: OffsetDateTime::now_utc(),
            })
            .await
            .map_err(TeamServiceError::Database)?;

        Ok(team.into())
    }

    pub async fn list_projects(
        &self,
        team_id: ObjectId,
    ) -> Result<Vec<ProjectPayload>, TeamServiceError> {
        match self.team_repo.find_by_id(team_id).await {
            Ok(Some(_)) => {}
            Ok(None) => return Err(TeamServiceError::TeamNotFound),
            Err(e) => return Err(TeamServiceError::Database(e))?,
        };

        let projects = self
            .project_repo
            .find_by_owner(team_id, OwnerType::Team)
            .await
            .map_err(TeamServiceError::Database)?;

        let payloads = projects.into_iter().map(|p| p.into()).collect();

        Ok(payloads)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::models::user::User;
    use crate::repo::project::tests::MockProjectRepo;
    use crate::repo::{team::tests::MockTeamRepo, user::tests::MockUserRepo};
    use std::sync::Mutex;

    #[tokio::test]
    async fn test_create_success() {
        let service = TeamService {
            user_repo: MockUserRepo {
                users: Mutex::new(vec![User {
                    id: ObjectId::parse_str("64b64c4f2f9b256e1c8e4d3a").unwrap(),
                    username: "test_user".to_string(),
                    nickname: "Test User".to_string(),
                    password: "hashed_password".to_string(),
                    avatar_uri: None,
                    created_at: OffsetDateTime::now_utc(),
                    updated_at: OffsetDateTime::now_utc(),
                }]),
            },
            team_repo: MockTeamRepo::default(),
            project_repo: MockProjectRepo::default(),
        };

        let result = service
            .create(
                ObjectId::parse_str("64b64c4f2f9b256e1c8e4d3a").unwrap(),
                "Test Team".to_string(),
            )
            .await;
        assert!(result.is_ok());
        let payload = result.unwrap();
        assert_eq!(payload.name, "Test Team");
    }

    #[tokio::test]
    async fn test_create_user_not_found() {
        let service = TeamService {
            user_repo: MockUserRepo::default(),
            team_repo: MockTeamRepo::default(),
            project_repo: MockProjectRepo::default(),
        };

        let result = service
            .create(
                ObjectId::parse_str("64b64c4f2f9b256e1c8e4d3a").unwrap(),
                "Test Team".to_string(),
            )
            .await;

        assert!(matches!(result, Err(TeamServiceError::UserNotFound)));
    }
}
