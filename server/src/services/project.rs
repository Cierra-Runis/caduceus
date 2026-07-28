use std::collections::HashMap;

use bson::oid::ObjectId;
use derive_more::Display;
use time::OffsetDateTime;

use crate::{
    models::{
        project::{
            DEFAULT_MAIN_TYP, OwnerType, Project, ProjectDetailPayload, ProjectPayload,
            ProjectSettings,
        },
        tree::{NodeContent, ProjectionEntry},
    },
    repo::{project::ProjectRepo, team::TeamRepo, user::UserRepo},
    storage::ProjectStore,
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
    #[display("Storage error")]
    Storage,
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
        store: &ProjectStore,
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

        // Seed an entry file so the project is editable/compilable immediately.
        // Its bytes go to a blob (the sole text store); the tree references it.
        let project_id = ObjectId::new();
        let entry_id = ObjectId::new();
        let now = OffsetDateTime::now_utc();
        let blob = store
            .put_blob(&project_id.to_hex(), DEFAULT_MAIN_TYP.as_bytes())
            .await
            .map_err(|_| ProjectServiceError::Storage)?;
        let mut tree = HashMap::new();
        tree.insert(
            entry_id.to_hex(),
            ProjectionEntry {
                parent: None,
                name: "main.typ".to_string(),
                path: "main.typ".to_string(),
                content: NodeContent::File { blob },
            },
        );

        let project = self
            .project_repo
            .create(Project {
                id: project_id,
                name,
                owner_id,
                owner_type,
                creator_id: creator.id,
                created_at: now,
                updated_at: now,
                entry: Some(entry_id),
                pinned_version: None,
                settings: ProjectSettings::default(),
                tree,
            })
            .await
            .map_err(ProjectServiceError::Database)?;

        Ok(project.into())
    }

    pub async fn find_by_id(
        &self,
        project_id: ObjectId,
    ) -> Result<ProjectDetailPayload, ProjectServiceError> {
        match self.project_repo.find_by_id(project_id).await {
            Ok(Some(project)) => Ok(project.into()),
            Ok(None) => Err(ProjectServiceError::ProjectNotFound),
            Err(e) => Err(ProjectServiceError::Database(e)),
        }
    }

    /// Update a project's metadata: rename it and/or move it between owners
    /// (personal space ↔ team). The caller must have access to the project,
    /// and the *target* owner is validated with the same rules as `create` —
    /// a user owner must be the requester themselves, a team owner must be a
    /// team the requester belongs to — so a project can never be pushed into
    /// someone else's space.
    pub async fn update(
        &self,
        project_id: ObjectId,
        user_id: ObjectId,
        name: String,
        owner_id: ObjectId,
        owner_type: OwnerType,
    ) -> Result<ProjectPayload, ProjectServiceError> {
        match self.accessible(project_id, user_id).await {
            Ok(true) => {}
            Ok(false) => return Err(ProjectServiceError::AccessDenied),
            Err(e) => return Err(e),
        };

        let owner_id = match owner_type {
            OwnerType::User => match self.user_repo.find_by_id(owner_id).await {
                Ok(Some(owner)) => {
                    if owner.id != user_id {
                        return Err(ProjectServiceError::CreatorNotMatchOwner);
                    }
                    owner.id
                }
                Ok(None) => return Err(ProjectServiceError::OwnerNotFound(OwnerType::User)),
                Err(e) => return Err(ProjectServiceError::Database(e)),
            },
            OwnerType::Team => match self.team_repo.find_by_id(owner_id).await {
                Ok(Some(team)) => {
                    if !team.member_ids.contains(&user_id) {
                        return Err(ProjectServiceError::CreatorNotMemberOfTeam);
                    }
                    team.id
                }
                Ok(None) => return Err(ProjectServiceError::OwnerNotFound(OwnerType::Team)),
                Err(e) => return Err(ProjectServiceError::Database(e)),
            },
        };

        match self
            .project_repo
            .update_metadata(project_id, name, owner_id, owner_type)
            .await
        {
            Ok(Some(project)) => Ok(project.into()),
            Ok(None) => Err(ProjectServiceError::ProjectNotFound),
            Err(e) => Err(ProjectServiceError::Database(e)),
        }
    }

    /// Update a project's editor settings (auto-save policy, …). Any
    /// collaborator with access can change them — they are project-level and
    /// shared. Returns the stored settings.
    pub async fn update_settings(
        &self,
        project_id: ObjectId,
        user_id: ObjectId,
        settings: ProjectSettings,
    ) -> Result<ProjectSettings, ProjectServiceError> {
        match self.accessible(project_id, user_id).await {
            Ok(true) => {}
            Ok(false) => return Err(ProjectServiceError::AccessDenied),
            Err(e) => return Err(e),
        };

        match self.project_repo.update_settings(project_id, settings).await {
            Ok(Some(project)) => Ok(project.settings),
            Ok(None) => Err(ProjectServiceError::ProjectNotFound),
            Err(e) => Err(ProjectServiceError::Database(e)),
        }
    }

    /// Clone a project the caller can access into a brand-new, independent
    /// project owned the same way (same `owner_id`/`owner_type`), with the
    /// requester recorded as the new project's `creator_id`. Every node gets a
    /// fresh id (parents remapped through the swap, `entry` too), and each
    /// file's bytes are copied into the new project's blob namespace — so the
    /// copy shares nothing with the source.
    pub async fn duplicate(
        &self,
        project_id: ObjectId,
        user_id: ObjectId,
        store: &ProjectStore,
    ) -> Result<ProjectPayload, ProjectServiceError> {
        match self.accessible(project_id, user_id).await {
            Ok(true) => {}
            Ok(false) => return Err(ProjectServiceError::AccessDenied),
            Err(e) => return Err(e),
        };

        let source = match self.project_repo.find_by_id(project_id).await {
            Ok(Some(project)) => project,
            Ok(None) => return Err(ProjectServiceError::ProjectNotFound),
            Err(e) => return Err(ProjectServiceError::Database(e)),
        };

        let now = OffsetDateTime::now_utc();
        let new_project_id = ObjectId::new();
        let src_hex = project_id.to_hex();
        let dst_hex = new_project_id.to_hex();

        // Fresh id per node, decided up front so parents can be remapped.
        let mut id_map: HashMap<String, String> = HashMap::with_capacity(source.tree.len());
        for old_id in source.tree.keys() {
            id_map.insert(old_id.clone(), ObjectId::new().to_hex());
        }

        let mut tree = HashMap::with_capacity(source.tree.len());
        for (old_id, ProjectionEntry { parent, name, path, content }) in source.tree {
            let new_id = id_map[&old_id].clone();
            let parent = parent.map(|p| id_map.get(&p).cloned().unwrap_or(p));
            // Copy a file's bytes into the new project's namespace (same sha,
            // content-addressed) so the duplicate references its own blobs.
            if let NodeContent::File { blob } = &content
                && let Some(bytes) = store
                    .get_blob(&src_hex, &blob.sha256)
                    .await
                    .map_err(|_| ProjectServiceError::Storage)?
                {
                    store
                        .put_blob(&dst_hex, &bytes)
                        .await
                        .map_err(|_| ProjectServiceError::Storage)?;
                }
            tree.insert(new_id, ProjectionEntry { parent, name, path, content });
        }

        let entry = source
            .entry
            .and_then(|old| id_map.get(&old.to_hex()).cloned())
            .and_then(|hex| ObjectId::parse_str(hex).ok());

        let project = self
            .project_repo
            .create(Project {
                id: new_project_id,
                name: format!("{} copy", source.name),
                owner_id: source.owner_id,
                owner_type: source.owner_type,
                creator_id: user_id,
                created_at: now,
                updated_at: now,
                entry,
                pinned_version: source.pinned_version,
                settings: source.settings,
                tree,
            })
            .await
            .map_err(ProjectServiceError::Database)?;

        Ok(project.into())
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
    use crate::storage::InMemoryObjectStore;
    use bson::oid::ObjectId;
    use std::sync::{Arc, Mutex};
    use time::OffsetDateTime;

    /// An in-memory project store for create/duplicate (which seed/copy blobs).
    fn store() -> ProjectStore {
        ProjectStore::new(Arc::new(InMemoryObjectStore::new()))
    }

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
            .create(creator_id, owner_id, OwnerType::User, "p1".to_string(), &store())
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
            .create(creator_id, creator_id, OwnerType::User, "p3".to_string(), &store())
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
            .create(creator_id, owner_id, OwnerType::User, "p4".to_string(), &store())
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
            .create(creator_id, owner_id, OwnerType::User, "p5".to_string(), &store())
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
            .create(creator_id, team_id, OwnerType::Team, "p7".to_string(), &store())
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
            .create(creator_id, team_id, OwnerType::Team, "p8".to_string(), &store())
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
            .create(creator_id, team_id, OwnerType::Team, "p9".to_string(), &store())
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
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            entry: None,
            pinned_version: None,
            settings: ProjectSettings::default(),
            tree: Default::default(),
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
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            entry: None,
            pinned_version: None,
            settings: ProjectSettings::default(),
            tree: Default::default(),
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
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            entry: None,
            pinned_version: None,
            settings: ProjectSettings::default(),
            tree: Default::default(),
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
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            entry: None,
            pinned_version: None,
            settings: ProjectSettings::default(),
            tree: Default::default(),
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
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            entry: None,
            pinned_version: None,
            settings: ProjectSettings::default(),
            tree: Default::default(),
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

    fn project_with_file(project_id: ObjectId, owner_id: ObjectId, file_id: ObjectId) -> Project {
        let mut tree = HashMap::new();
        tree.insert(
            file_id.to_hex(),
            ProjectionEntry {
                parent: None,
                name: "main.typ".to_string(),
                path: "main.typ".to_string(),
                content: NodeContent::File {
                    blob: crate::storage::Blob {
                        sha256: "a".repeat(64),
                        size: 3,
                    },
                },
            },
        );
        Project {
            id: project_id,
            name: "test".to_string(),
            owner_id,
            owner_type: OwnerType::User,
            creator_id: owner_id,
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            entry: Some(file_id),
            pinned_version: None,
            settings: ProjectSettings::default(),
            tree,
        }
    }

    #[tokio::test]
    async fn test_update_project_rename_success() {
        let owner_id = ObjectId::new();
        let project_id = ObjectId::new();
        let file_id = ObjectId::new();
        let service = ProjectService {
            project_repo: MockProjectRepo {
                projects: Mutex::new(vec![project_with_file(project_id, owner_id, file_id)]),
            },
            user_repo: MockUserRepo {
                users: Mutex::new(vec![dummy_user(owner_id)]),
            },
            team_repo: MockTeamRepo::default(),
        };

        let payload = service
            .update(
                project_id,
                owner_id,
                "renamed".to_string(),
                owner_id,
                OwnerType::User,
            )
            .await
            .unwrap();

        assert_eq!(payload.name, "renamed");
        assert_eq!(payload.owner_id, owner_id.to_hex());
        assert_eq!(payload.owner_type, OwnerType::User);
    }

    #[tokio::test]
    async fn test_update_project_move_to_team() {
        let owner_id = ObjectId::new();
        let team_id = ObjectId::new();
        let project_id = ObjectId::new();
        let file_id = ObjectId::new();
        let service = ProjectService {
            project_repo: MockProjectRepo {
                projects: Mutex::new(vec![project_with_file(project_id, owner_id, file_id)]),
            },
            user_repo: MockUserRepo {
                users: Mutex::new(vec![dummy_user(owner_id)]),
            },
            team_repo: MockTeamRepo {
                teams: Mutex::new(vec![dummy_team(team_id, vec![owner_id])]),
            },
        };

        let payload = service
            .update(
                project_id,
                owner_id,
                "moved".to_string(),
                team_id,
                OwnerType::Team,
            )
            .await
            .unwrap();

        assert_eq!(payload.owner_id, team_id.to_hex());
        assert_eq!(payload.owner_type, OwnerType::Team);
    }

    #[tokio::test]
    async fn test_update_project_access_denied() {
        let owner_id = ObjectId::new();
        let other_user_id = ObjectId::new();
        let project_id = ObjectId::new();
        let file_id = ObjectId::new();
        let service = ProjectService {
            project_repo: MockProjectRepo {
                projects: Mutex::new(vec![project_with_file(project_id, owner_id, file_id)]),
            },
            user_repo: MockUserRepo::default(),
            team_repo: MockTeamRepo::default(),
        };

        let res = service
            .update(
                project_id,
                other_user_id,
                "x".to_string(),
                other_user_id,
                OwnerType::User,
            )
            .await;
        assert!(matches!(res, Err(ProjectServiceError::AccessDenied)));
    }

    #[tokio::test]
    async fn test_update_project_not_found() {
        let service = ProjectService {
            project_repo: MockProjectRepo::default(),
            user_repo: MockUserRepo::default(),
            team_repo: MockTeamRepo::default(),
        };

        let user_id = ObjectId::new();
        let res = service
            .update(
                ObjectId::new(),
                user_id,
                "x".to_string(),
                user_id,
                OwnerType::User,
            )
            .await;
        assert!(matches!(res, Err(ProjectServiceError::ProjectNotFound)));
    }

    #[tokio::test]
    async fn test_update_project_target_user_not_self() {
        let owner_id = ObjectId::new();
        let other_user_id = ObjectId::new();
        let project_id = ObjectId::new();
        let file_id = ObjectId::new();
        let service = ProjectService {
            project_repo: MockProjectRepo {
                projects: Mutex::new(vec![project_with_file(project_id, owner_id, file_id)]),
            },
            user_repo: MockUserRepo {
                users: Mutex::new(vec![dummy_user(owner_id), dummy_user(other_user_id)]),
            },
            team_repo: MockTeamRepo::default(),
        };

        // The owner tries to hand the project to another user directly.
        let res = service
            .update(
                project_id,
                owner_id,
                "x".to_string(),
                other_user_id,
                OwnerType::User,
            )
            .await;
        assert!(matches!(
            res,
            Err(ProjectServiceError::CreatorNotMatchOwner)
        ));
    }

    #[tokio::test]
    async fn test_update_project_target_team_not_member() {
        let owner_id = ObjectId::new();
        let team_id = ObjectId::new();
        let project_id = ObjectId::new();
        let file_id = ObjectId::new();
        let service = ProjectService {
            project_repo: MockProjectRepo {
                projects: Mutex::new(vec![project_with_file(project_id, owner_id, file_id)]),
            },
            user_repo: MockUserRepo {
                users: Mutex::new(vec![dummy_user(owner_id)]),
            },
            team_repo: MockTeamRepo {
                teams: Mutex::new(vec![dummy_team(team_id, vec![ObjectId::new()])]),
            },
        };

        let res = service
            .update(
                project_id,
                owner_id,
                "x".to_string(),
                team_id,
                OwnerType::Team,
            )
            .await;
        assert!(matches!(
            res,
            Err(ProjectServiceError::CreatorNotMemberOfTeam)
        ));
    }

    #[tokio::test]
    async fn test_update_project_target_owner_not_found() {
        let owner_id = ObjectId::new();
        let project_id = ObjectId::new();
        let file_id = ObjectId::new();
        let service = ProjectService {
            project_repo: MockProjectRepo {
                projects: Mutex::new(vec![project_with_file(project_id, owner_id, file_id)]),
            },
            user_repo: MockUserRepo::default(),
            team_repo: MockTeamRepo::default(),
        };

        let res = service
            .update(
                project_id,
                owner_id,
                "x".to_string(),
                ObjectId::new(),
                OwnerType::Team,
            )
            .await;
        assert!(matches!(
            res,
            Err(ProjectServiceError::OwnerNotFound(OwnerType::Team))
        ));
    }

    #[tokio::test]
    async fn test_update_settings_success() {
        use crate::models::project::AutoSavePolicy;

        let owner_id = ObjectId::new();
        let project_id = ObjectId::new();
        let file_id = ObjectId::new();
        let service = ProjectService {
            project_repo: MockProjectRepo {
                projects: Mutex::new(vec![project_with_file(project_id, owner_id, file_id)]),
            },
            user_repo: MockUserRepo::default(),
            team_repo: MockTeamRepo::default(),
        };

        let settings = ProjectSettings {
            auto_save: AutoSavePolicy::AfterDelay,
            auto_save_delay: 500,
        };
        let stored = service
            .update_settings(project_id, owner_id, settings)
            .await
            .unwrap();

        assert_eq!(stored.auto_save, AutoSavePolicy::AfterDelay);
        assert_eq!(stored.auto_save_delay, 500);
    }

    #[tokio::test]
    async fn test_update_settings_access_denied() {
        let owner_id = ObjectId::new();
        let other_user_id = ObjectId::new();
        let project_id = ObjectId::new();
        let file_id = ObjectId::new();
        let service = ProjectService {
            project_repo: MockProjectRepo {
                projects: Mutex::new(vec![project_with_file(project_id, owner_id, file_id)]),
            },
            user_repo: MockUserRepo::default(),
            team_repo: MockTeamRepo::default(),
        };

        let res = service
            .update_settings(project_id, other_user_id, ProjectSettings::default())
            .await;
        assert!(matches!(res, Err(ProjectServiceError::AccessDenied)));
    }

    #[tokio::test]
    async fn test_duplicate_project_success() {
        let creator_id = ObjectId::new();
        let project_id = ObjectId::new();
        let file_id = ObjectId::new();
        let service = ProjectService {
            project_repo: MockProjectRepo {
                projects: Mutex::new(vec![project_with_file(project_id, creator_id, file_id)]),
            },
            user_repo: MockUserRepo::default(),
            team_repo: MockTeamRepo::default(),
        };

        let payload = service.duplicate(project_id, creator_id, &store()).await.unwrap();

        assert_eq!(payload.name, "test copy");
        assert_eq!(payload.owner_id, creator_id.to_hex());
        assert_eq!(payload.creator_id, creator_id.to_hex());
        assert_ne!(payload.id, project_id.to_hex());

        // `entry` is remapped to the duplicated file's fresh id — the copy does
        // not alias the source's node ids.
        let new_entry = payload.entry.clone().expect("duplicate keeps an entry");
        assert_ne!(new_entry, file_id.to_hex());
    }

    #[tokio::test]
    async fn test_duplicate_project_team_owned_sets_requester_as_creator() {
        let original_creator_id = ObjectId::new();
        let team_id = ObjectId::new();
        let member_id = ObjectId::new();
        let project_id = ObjectId::new();

        let project = Project {
            id: project_id,
            name: "team project".to_string(),
            owner_id: team_id,
            owner_type: OwnerType::Team,
            creator_id: original_creator_id,
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            entry: None,
            pinned_version: None,
            settings: ProjectSettings::default(),
            tree: Default::default(),
        };

        let team = dummy_team(team_id, vec![original_creator_id, member_id]);

        let service = ProjectService {
            project_repo: MockProjectRepo {
                projects: Mutex::new(vec![project]),
            },
            user_repo: MockUserRepo::default(),
            team_repo: MockTeamRepo {
                teams: Mutex::new(vec![team]),
            },
        };

        // A team member other than the original creator duplicates the
        // project: ownership stays with the team, but the duplicate's creator
        // is the requester, not the original creator.
        let payload = service.duplicate(project_id, member_id, &store()).await.unwrap();
        assert_eq!(payload.owner_id, team_id.to_hex());
        assert_eq!(payload.owner_type, OwnerType::Team);
        assert_eq!(payload.creator_id, member_id.to_hex());
        assert_ne!(payload.creator_id, original_creator_id.to_hex());
    }

    #[tokio::test]
    async fn test_duplicate_project_not_found() {
        let service = ProjectService {
            project_repo: MockProjectRepo::default(),
            user_repo: MockUserRepo::default(),
            team_repo: MockTeamRepo::default(),
        };

        let res = service.duplicate(ObjectId::new(), ObjectId::new(), &store()).await;
        assert!(matches!(res, Err(ProjectServiceError::ProjectNotFound)));
    }

    #[tokio::test]
    async fn test_duplicate_project_access_denied() {
        let owner_id = ObjectId::new();
        let other_user_id = ObjectId::new();
        let project_id = ObjectId::new();
        let file_id = ObjectId::new();
        let service = ProjectService {
            project_repo: MockProjectRepo {
                projects: Mutex::new(vec![project_with_file(project_id, owner_id, file_id)]),
            },
            user_repo: MockUserRepo::default(),
            team_repo: MockTeamRepo::default(),
        };

        let res = service.duplicate(project_id, other_user_id, &store()).await;
        assert!(matches!(res, Err(ProjectServiceError::AccessDenied)));
    }
}
