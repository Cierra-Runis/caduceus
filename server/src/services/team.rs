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
    #[display("Access denied: You are not a member of this team")]
    AccessDenied,
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

    /// List a team's projects. Only members may see them — team projects are
    /// not public, so a valid login alone is not enough.
    pub async fn list_projects(
        &self,
        team_id: ObjectId,
        user_id: ObjectId,
    ) -> Result<Vec<ProjectPayload>, TeamServiceError> {
        match self.team_repo.find_by_id(team_id).await {
            Ok(Some(team)) => {
                if !team.member_ids.contains(&user_id) {
                    return Err(TeamServiceError::AccessDenied);
                }
            }
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
    use crate::models::project::Project;
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

    fn team_with_members(id: ObjectId, member_ids: Vec<ObjectId>) -> Team {
        Team {
            id,
            name: "Test Team".to_string(),
            avatar_uri: None,
            creator_id: member_ids[0],
            member_ids,
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
        }
    }

    fn team_project(owner_id: ObjectId, creator_id: ObjectId) -> Project {
        Project {
            id: ObjectId::new(),
            name: "Team Project".to_string(),
            owner_id,
            owner_type: OwnerType::Team,
            creator_id,
            files: vec![],
            directories: vec![],
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            entry: None,
            pinned_version: None,
        }
    }

    #[tokio::test]
    async fn test_list_projects_member_success() {
        let member_id = ObjectId::new();
        let team_id = ObjectId::new();

        let service = TeamService {
            user_repo: MockUserRepo::default(),
            team_repo: MockTeamRepo {
                teams: Mutex::new(vec![team_with_members(team_id, vec![member_id])]),
            },
            project_repo: MockProjectRepo {
                projects: Mutex::new(vec![team_project(team_id, member_id)]),
            },
        };

        let projects = service.list_projects(team_id, member_id).await.unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "Team Project");
    }

    #[tokio::test]
    async fn test_list_projects_denied_for_non_member() {
        let member_id = ObjectId::new();
        let outsider_id = ObjectId::new();
        let team_id = ObjectId::new();

        let service = TeamService {
            user_repo: MockUserRepo::default(),
            team_repo: MockTeamRepo {
                teams: Mutex::new(vec![team_with_members(team_id, vec![member_id])]),
            },
            project_repo: MockProjectRepo {
                projects: Mutex::new(vec![team_project(team_id, member_id)]),
            },
        };

        let result = service.list_projects(team_id, outsider_id).await;
        assert!(matches!(result, Err(TeamServiceError::AccessDenied)));
    }

    #[tokio::test]
    async fn test_list_projects_team_not_found() {
        let service = TeamService {
            user_repo: MockUserRepo::default(),
            team_repo: MockTeamRepo::default(),
            project_repo: MockProjectRepo::default(),
        };

        let result = service
            .list_projects(ObjectId::new(), ObjectId::new())
            .await;
        assert!(matches!(result, Err(TeamServiceError::TeamNotFound)));
    }
}
