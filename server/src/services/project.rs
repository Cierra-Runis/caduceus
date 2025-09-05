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
        let creator = match self.user_repo.find_by_id(creator_id).await {
            Ok(Some(user)) => user,
            Ok(None) => return Err(ProjectServiceError::UserNotFound),
            Err(e) => return Err(ProjectServiceError::Database(e)),
        };

        let owner_id = match owner_type {
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
                created_at: OffsetDateTime::now_utc(),
                updated_at: OffsetDateTime::now_utc(),
            })
            .await
            .map_err(ProjectServiceError::Database)?;

        Ok(project.into())
    }
}
