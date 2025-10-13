use bson::oid::ObjectId;
use derive_more::Display;
use time::OffsetDateTime;

use crate::{
    models::project::{OwnerType, Project, ProjectPayload},
    repo::{project::ProjectRepo, team::TeamRepo, user::UserRepo},
};

#[derive(Debug, Display)]
pub enum ProjectServiceError {
    #[display("User not found")]
    UserNotFound,
    #[display("Owner not found: {_0}")]
    OwnerNotFound(OwnerType),
    #[display("Project not found")]
    ProjectNotFound,
    #[display("Access denied: You do not have permission to access this project")]
    AccessDenied,
    #[display("Creator does not match owner")]
    CreatorNotMatchOwner,
    #[display("Creator is not a member of the team")]
    CreatorNotMemberOfTeam,
    #[display("Invalid owner type")]
    InvalidOwnerType,
    #[display("Database error: {_0}")]
    Database(mongodb::error::Error),
}

pub struct ProjectService<P: ProjectRepo, U: UserRepo, T: TeamRepo> {
    pub project_repo: P,
    pub user_repo: U,
    pub team_repo: T,
}

impl<P: ProjectRepo, U: UserRepo, T: TeamRepo> ProjectService<P, U, T> {
    pub async fn create(
        &self,
        creator_id: ObjectId,
        owner_id: ObjectId,
        owner_type: OwnerType,
        name: String,
    ) -> Result<ProjectPayload, ProjectServiceError> {
        // Validate creator exists, creator must be a user
        let creator = match self.user_repo.find_by_id(creator_id).await {
            Ok(Some(user)) => user,
            Ok(None) => return Err(ProjectServiceError::UserNotFound),
            Err(e) => return Err(ProjectServiceError::Database(e)),
        };

        // Validate owner exists
        let owner_id = match owner_type {
            // If owner is a user, ensure the creator is the same as the owner
            OwnerType::User => match self.user_repo.find_by_id(owner_id).await {
                Ok(Some(owner)) => {
                    if creator.id != owner.id {
                        return Err(ProjectServiceError::CreatorNotMatchOwner);
                    }
                    owner.id
                }
                Ok(None) => return Err(ProjectServiceError::OwnerNotFound(OwnerType::User)),
                Err(e) => return Err(ProjectServiceError::Database(e)),
            },
            // If owner is a team, ensure the creator is a member of the team
            // TIPS: Maybe in the future we can add more roles and permissions
            OwnerType::Team => match self.team_repo.find_by_id(owner_id).await {
                Ok(Some(team)) => {
                    if !team.member_ids.contains(&creator.id) {
                        return Err(ProjectServiceError::CreatorNotMemberOfTeam);
                    }
                    team.id
                }
                Ok(None) => return Err(ProjectServiceError::OwnerNotFound(OwnerType::Team)),
                Err(e) => return Err(ProjectServiceError::Database(e)),
            },
        };

        let project = self
            .project_repo
            .create(Project {
                id: ObjectId::new(),
                name,
                owner_id,
                owner_type,
                creator_id: creator.id,
                files: vec![],
                created_at: OffsetDateTime::now_utc(),
                updated_at: OffsetDateTime::now_utc(),
                preview: None,
                pinned_version: None,
            })
            .await
            .map_err(ProjectServiceError::Database)?;

        Ok(project.into())
    }

    pub async fn find_by_id(
        &self,
        project_id: ObjectId,
    ) -> Result<ProjectPayload, ProjectServiceError> {
        match self.project_repo.find_by_id(project_id).await {
            Ok(Some(project)) => Ok(project.into()),
            Ok(None) => Err(ProjectServiceError::ProjectNotFound),
            Err(e) => Err(ProjectServiceError::Database(e)),
        }
    }
}

impl<P: ProjectRepo, U: UserRepo, T: TeamRepo> ProjectService<P, U, T> {
    pub async fn accessible(
        &self,
        project_id: ObjectId,
        user_id: ObjectId,
    ) -> Result<bool, ProjectServiceError> {
        let project = match self.project_repo.find_by_id(project_id).await {
            Ok(Some(project)) => project,
            Ok(None) => return Err(ProjectServiceError::ProjectNotFound),
            Err(e) => return Err(ProjectServiceError::Database(e)),
        };

        // Check if user is the creator
        if project.creator_id == user_id {
            return Ok(true);
        }

        // Check based on owner type
        match project.owner_type {
            OwnerType::User => {
                // If owner is a user, check if it's the same user
                Ok(project.owner_id == user_id)
            }
            OwnerType::Team => {
                // If owner is a team, check if user is a team member
                match self.team_repo.find_by_id(project.owner_id).await {
                    Ok(Some(team)) => Ok(team.member_ids.contains(&user_id)),
                    Ok(None) => Err(ProjectServiceError::OwnerNotFound(OwnerType::Team)),
                    Err(e) => Err(ProjectServiceError::Database(e)),
                }
            }
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::models::{project::OwnerType, team::Team, user::User};
    use crate::repo::project::tests::MockProjectRepo;
    use crate::repo::team::tests::MockTeamRepo;
    use crate::repo::user::tests::MockUserRepo;
    use bson::oid::ObjectId;
    use std::sync::Mutex;
    use time::OffsetDateTime;

    fn dummy_user(id: ObjectId) -> User {
        User {
            id,
            username: format!("user_{}", id),
            nickname: "nick".to_string(),
            password: "pwd".to_string(),
            avatar_uri: None,
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
        }
    }

    fn dummy_team(id: ObjectId, member_ids: Vec<ObjectId>) -> Team {
        Team {
            id,
            name: format!("team_{}", id),
            avatar_uri: None,
            creator_id: member_ids[0],
            member_ids,
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
        }
    }

    #[tokio::test]
    async fn test_create_project_invalid_creator() {
        let service = ProjectService {
            project_repo: MockProjectRepo::default(),
            user_repo: MockUserRepo::default(),
            team_repo: MockTeamRepo::default(),
        };
        let creator_id = ObjectId::new();
        let owner_id = creator_id;
        let res = service
            .create(creator_id, owner_id, OwnerType::User, "p1".to_string())
            .await;
        assert!(matches!(res, Err(ProjectServiceError::UserNotFound)));
    }

    #[tokio::test]
    async fn test_create_project_user_owner_success() {
        let creator_id = ObjectId::new();
        let user = dummy_user(creator_id);
        let service = ProjectService {
            project_repo: MockProjectRepo::default(),
            user_repo: MockUserRepo {
                users: Mutex::new(vec![user.clone()]),
            },
            team_repo: MockTeamRepo::default(),
        };
        let res = service
            .create(creator_id, creator_id, OwnerType::User, "p3".to_string())
            .await;
        assert!(res.is_ok());
        let payload = res.unwrap();
        assert_eq!(payload.owner_id, creator_id.to_hex());
        assert_eq!(payload.owner_type, OwnerType::User);
    }

    #[tokio::test]
    async fn test_create_project_owner_user_not_found() {
        let creator_id = ObjectId::new();
        let user = dummy_user(creator_id);
        let service = ProjectService {
            project_repo: MockProjectRepo::default(),
            user_repo: MockUserRepo {
                users: Mutex::new(vec![user.clone()]),
            },
            team_repo: MockTeamRepo::default(),
        };
        let owner_id = ObjectId::new();
        let res = service
            .create(creator_id, owner_id, OwnerType::User, "p4".to_string())
            .await;
        assert!(matches!(
            res,
            Err(ProjectServiceError::OwnerNotFound(OwnerType::User))
        ));
    }

    #[tokio::test]
    async fn test_create_project_owner_user_not_match() {
        let creator_id = ObjectId::new();
        let owner_id = ObjectId::new();
        let service = ProjectService {
            project_repo: MockProjectRepo::default(),
            user_repo: MockUserRepo {
                users: Mutex::new(vec![dummy_user(creator_id), dummy_user(owner_id)]),
            },
            team_repo: MockTeamRepo::default(),
        };
        let res = service
            .create(creator_id, owner_id, OwnerType::User, "p5".to_string())
            .await;
        assert!(matches!(
            res,
            Err(ProjectServiceError::CreatorNotMatchOwner)
        ));
    }

    #[tokio::test]
    async fn test_create_project_team_owner_success() {
        let creator_id = ObjectId::new();
        let team_id = ObjectId::new();
        let user = dummy_user(creator_id);
        let team = dummy_team(team_id, vec![creator_id]);
        let service = ProjectService {
            project_repo: MockProjectRepo::default(),
            user_repo: MockUserRepo {
                users: Mutex::new(vec![user.clone()]),
            },
            team_repo: MockTeamRepo {
                teams: Mutex::new(vec![team.clone()]),
            },
        };
        let res = service
            .create(creator_id, team_id, OwnerType::Team, "p7".to_string())
            .await;
        assert!(res.is_ok());
        let payload = res.unwrap();
        assert_eq!(payload.owner_id, team_id.to_hex());
        assert_eq!(payload.owner_type, OwnerType::Team);
    }

    #[tokio::test]
    async fn test_create_project_team_owner_not_member() {
        let creator_id = ObjectId::new();
        let team_id = ObjectId::new();
        let user = dummy_user(creator_id);
        let team = dummy_team(team_id, vec![ObjectId::new()]);
        let service = ProjectService {
            project_repo: MockProjectRepo::default(),
            user_repo: MockUserRepo {
                users: Mutex::new(vec![user.clone()]),
            },
            team_repo: MockTeamRepo {
                teams: Mutex::new(vec![team.clone()]),
            },
        };
        let res = service
            .create(creator_id, team_id, OwnerType::Team, "p8".to_string())
            .await;
        assert!(matches!(
            res,
            Err(ProjectServiceError::CreatorNotMemberOfTeam)
        ));
    }

    #[tokio::test]
    async fn test_create_project_team_owner_not_found() {
        let creator_id = ObjectId::new();
        let team_id = ObjectId::new();
        let user = dummy_user(creator_id);
        let service = ProjectService {
            project_repo: MockProjectRepo::default(),
            user_repo: MockUserRepo {
                users: Mutex::new(vec![user.clone()]),
            },
            team_repo: MockTeamRepo::default(),
        };
        let res = service
            .create(creator_id, team_id, OwnerType::Team, "p9".to_string())
            .await;
        assert!(matches!(
            res,
            Err(ProjectServiceError::OwnerNotFound(OwnerType::Team))
        ));
    }

    #[tokio::test]
    async fn test_check_access_creator() {
        let creator_id = ObjectId::new();
        let owner_id = ObjectId::new();
        let project_id = ObjectId::new();

        let project = Project {
            id: project_id,
            name: "test".to_string(),
            owner_id,
            owner_type: OwnerType::User,
            creator_id,
            files: vec![],
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            preview: None,
            pinned_version: None,
        };

        let service = ProjectService {
            project_repo: MockProjectRepo {
                projects: Mutex::new(vec![project]),
            },
            user_repo: MockUserRepo::default(),
            team_repo: MockTeamRepo::default(),
        };

        let has_access = service.accessible(project_id, creator_id).await.unwrap();
        assert!(has_access);
    }

    #[tokio::test]
    async fn test_check_access_user_owner() {
        let creator_id = ObjectId::new();
        let owner_id = ObjectId::new();
        let project_id = ObjectId::new();

        let project = Project {
            id: project_id,
            name: "test".to_string(),
            owner_id,
            owner_type: OwnerType::User,
            creator_id,
            files: vec![],
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            preview: None,
            pinned_version: None,
        };

        let service = ProjectService {
            project_repo: MockProjectRepo {
                projects: Mutex::new(vec![project]),
            },
            user_repo: MockUserRepo::default(),
            team_repo: MockTeamRepo::default(),
        };

        let has_access = service.accessible(project_id, owner_id).await.unwrap();
        assert!(has_access);
    }

    #[tokio::test]
    async fn test_check_access_team_member() {
        let creator_id = ObjectId::new();
        let team_id = ObjectId::new();
        let member_id = ObjectId::new();
        let project_id = ObjectId::new();

        let project = Project {
            id: project_id,
            name: "test".to_string(),
            owner_id: team_id,
            owner_type: OwnerType::Team,
            creator_id,
            files: vec![],
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            preview: None,
            pinned_version: None,
        };

        let team = dummy_team(team_id, vec![creator_id, member_id]);

        let service = ProjectService {
            project_repo: MockProjectRepo {
                projects: Mutex::new(vec![project]),
            },
            user_repo: MockUserRepo::default(),
            team_repo: MockTeamRepo {
                teams: Mutex::new(vec![team]),
            },
        };

        let has_access = service.accessible(project_id, member_id).await.unwrap();
        assert!(has_access);
    }

    #[tokio::test]
    async fn test_check_access_denied_not_member() {
        let creator_id = ObjectId::new();
        let team_id = ObjectId::new();
        let other_user_id = ObjectId::new();
        let project_id = ObjectId::new();

        let project = Project {
            id: project_id,
            name: "test".to_string(),
            owner_id: team_id,
            owner_type: OwnerType::Team,
            creator_id,
            files: vec![],
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            preview: None,
            pinned_version: None,
        };

        let team = dummy_team(team_id, vec![creator_id]);

        let service = ProjectService {
            project_repo: MockProjectRepo {
                projects: Mutex::new(vec![project]),
            },
            user_repo: MockUserRepo::default(),
            team_repo: MockTeamRepo {
                teams: Mutex::new(vec![team]),
            },
        };

        let has_access = service.accessible(project_id, other_user_id).await.unwrap();
        assert!(!has_access);
    }

    #[tokio::test]
    async fn test_check_access_denied_different_user() {
        let creator_id = ObjectId::new();
        let owner_id = ObjectId::new();
        let other_user_id = ObjectId::new();
        let project_id = ObjectId::new();

        let project = Project {
            id: project_id,
            name: "test".to_string(),
            owner_id,
            owner_type: OwnerType::User,
            creator_id,
            files: vec![],
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            preview: None,
            pinned_version: None,
        };

        let service = ProjectService {
            project_repo: MockProjectRepo {
                projects: Mutex::new(vec![project]),
            },
            user_repo: MockUserRepo::default(),
            team_repo: MockTeamRepo::default(),
        };

        let has_access = service.accessible(project_id, other_user_id).await.unwrap();
        assert!(!has_access);
    }
}
