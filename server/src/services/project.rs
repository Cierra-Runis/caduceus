use std::collections::{HashMap, HashSet};

use bson::oid::ObjectId;
use derive_more::Display;
use time::OffsetDateTime;

use crate::{
    models::{
        path::{self, PathError},
        project::{
            FileContent, OwnerType, Project, ProjectDetailPayload, ProjectFile, ProjectFilePayload,
            ProjectPayload, UpdateFilePayload,
        },
    },
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
    #[display("Invalid path: {_0}")]
    InvalidPath(PathError),
    #[display("Path already exists: {_0}")]
    PathConflict(String),
    #[display("File not found")]
    FileNotFound,
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

        // Seed an entry file so the project is editable/compilable immediately.
        let entry_file = ProjectFile::default();
        let now = entry_file.updated_at;
        let entry_id = entry_file.id;

        let project = self
            .project_repo
            .create(Project {
                id: ObjectId::new(),
                name,
                owner_id,
                owner_type,
                creator_id: creator.id,
                files: vec![entry_file],
                directories: vec![],
                created_at: now,
                updated_at: now,
                entry: Some(entry_id),
                pinned_version: None,
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

    /// Persist a text edit to a single file. Caller must have access. Returns
    /// the file's new version/timestamp. Whole-buffer save (not a delta) — this
    /// is the at-rest store, orthogonal to how edits are *synced* between
    /// collaborators (that becomes CRDT in M5).
    pub async fn update_file(
        &self,
        project_id: ObjectId,
        user_id: ObjectId,
        file_id: ObjectId,
        text: String,
    ) -> Result<UpdateFilePayload, ProjectServiceError> {
        match self.accessible(project_id, user_id).await {
            Ok(true) => {}
            Ok(false) => return Err(ProjectServiceError::AccessDenied),
            Err(e) => return Err(e),
        };

        let size = text.len() as i64;
        let content = FileContent::Text { text };

        match self
            .project_repo
            .update_file_content(project_id, file_id, content, size)
            .await
        {
            Ok(Some(project)) => project
                .files
                .into_iter()
                .find(|file| file.id == file_id)
                .map(UpdateFilePayload::from)
                .ok_or(ProjectServiceError::ProjectNotFound),
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

    /// Clone a project the caller can access into a brand-new, independent
    /// project owned the same way (same `owner_id`/`owner_type`), with the
    /// requester recorded as the new project's `creator_id`. Every file gets a
    /// fresh id — the copy must not alias the source's file rows — and `entry`
    /// is remapped through that id swap so the duplicate still opens on the
    /// same logical file.
    pub async fn duplicate(
        &self,
        project_id: ObjectId,
        user_id: ObjectId,
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

        let mut id_map = HashMap::with_capacity(source.files.len());
        let files: Vec<ProjectFile> = source
            .files
            .into_iter()
            .map(|file| {
                let new_id = ObjectId::new();
                id_map.insert(file.id, new_id);
                ProjectFile {
                    id: new_id,
                    updated_at: now,
                    ..file
                }
            })
            .collect();
        let entry = source.entry.and_then(|old_id| id_map.get(&old_id).copied());

        let project = self
            .project_repo
            .create(Project {
                id: ObjectId::new(),
                name: format!("{} copy", source.name),
                owner_id: source.owner_id,
                owner_type: source.owner_type,
                creator_id: user_id,
                files,
                directories: source.directories,
                created_at: now,
                updated_at: now,
                entry,
                pinned_version: source.pinned_version,
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

/// File-tree structural operations (create / rename / delete / upload). These
/// are pure metadata + validation over the [`Project`] document; the bytes of a
/// binary asset live in object storage and are orchestrated by the handler, so
/// this layer stays fully unit-testable against the mock repos.
impl<P: ProjectRepo, U: UserRepo, T: TeamRepo> ProjectService<P, U, T> {
    /// Create an empty text file at `path`.
    pub async fn create_file(
        &self,
        project_id: ObjectId,
        user_id: ObjectId,
        path: String,
    ) -> Result<ProjectFilePayload, ProjectServiceError> {
        let project = self.accessible_project(project_id, user_id).await?;
        let path = path::normalize_path(&path).map_err(ProjectServiceError::InvalidPath)?;
        validate_new_paths(&project, std::slice::from_ref(&path))?;

        let file = ProjectFile {
            id: ObjectId::new(),
            path,
            content: FileContent::Text {
                text: String::new(),
            },
            size: 0,
            version: 0,
            updated_at: OffsetDateTime::now_utc(),
        };
        let file_id = file.id;

        match self.project_repo.add_files(project_id, vec![file]).await {
            Ok(Some(project)) => project
                .files
                .into_iter()
                .find(|f| f.id == file_id)
                .map(ProjectFilePayload::from)
                .ok_or(ProjectServiceError::ProjectNotFound),
            Ok(None) => Err(ProjectServiceError::ProjectNotFound),
            Err(e) => Err(ProjectServiceError::Database(e)),
        }
    }

    /// Create an explicit (empty) directory at `path`.
    pub async fn create_directory(
        &self,
        project_id: ObjectId,
        user_id: ObjectId,
        path: String,
    ) -> Result<ProjectPayload, ProjectServiceError> {
        let project = self.accessible_project(project_id, user_id).await?;
        let path = path::normalize_directory(&path).map_err(ProjectServiceError::InvalidPath)?;

        let (files, dirs) = occupied_paths(&project);
        check_directory_free(&files, &dirs, &path)?;

        match self.project_repo.add_directory(project_id, path).await {
            Ok(Some(project)) => Ok(project.into()),
            Ok(None) => Err(ProjectServiceError::ProjectNotFound),
            Err(e) => Err(ProjectServiceError::Database(e)),
        }
    }

    /// Rename or move a single file to `new_path`.
    pub async fn rename_file(
        &self,
        project_id: ObjectId,
        user_id: ObjectId,
        file_id: ObjectId,
        new_path: String,
    ) -> Result<ProjectFilePayload, ProjectServiceError> {
        let project = self.accessible_project(project_id, user_id).await?;
        let new_path =
            path::normalize_path(&new_path).map_err(ProjectServiceError::InvalidPath)?;

        // The file must exist; renaming it to its own current path is a no-op
        // that should still succeed.
        let current = project
            .files
            .iter()
            .find(|f| f.id == file_id)
            .ok_or(ProjectServiceError::FileNotFound)?;
        if current.path != new_path {
            // Validate against the tree with the file being renamed removed, so
            // it never "collides with itself".
            let without_self: Vec<ProjectFile> = project
                .files
                .iter()
                .filter(|f| f.id != file_id)
                .cloned()
                .collect();
            let scratch = Project {
                files: without_self,
                ..project.clone()
            };
            validate_new_paths(&scratch, std::slice::from_ref(&new_path))?;
        }

        match self
            .project_repo
            .rename_file(project_id, file_id, new_path)
            .await
        {
            Ok(Some(project)) => project
                .files
                .into_iter()
                .find(|f| f.id == file_id)
                .map(ProjectFilePayload::from)
                .ok_or(ProjectServiceError::FileNotFound),
            Ok(None) => Err(ProjectServiceError::FileNotFound),
            Err(e) => Err(ProjectServiceError::Database(e)),
        }
    }

    /// Delete one file. Returns the removed file so the handler can free its
    /// backing object-storage bytes when it is a binary asset. Clears the
    /// compile entry if this file was it.
    pub async fn delete_file(
        &self,
        project_id: ObjectId,
        user_id: ObjectId,
        file_id: ObjectId,
    ) -> Result<ProjectFile, ProjectServiceError> {
        let project = self.accessible_project(project_id, user_id).await?;
        let removed = project
            .files
            .iter()
            .find(|f| f.id == file_id)
            .cloned()
            .ok_or(ProjectServiceError::FileNotFound)?;

        match self.project_repo.delete_file(project_id, file_id).await {
            Ok(Some(_)) => {}
            Ok(None) => return Err(ProjectServiceError::ProjectNotFound),
            Err(e) => return Err(ProjectServiceError::Database(e)),
        }

        if project.entry == Some(file_id) {
            self.clear_entry(project_id).await?;
        }
        Ok(removed)
    }

    /// Delete a directory and everything under it. Returns the storage keys of
    /// every binary asset removed, so the handler can free their bytes. Clears
    /// the compile entry if it pointed inside the deleted subtree.
    pub async fn delete_directory(
        &self,
        project_id: ObjectId,
        user_id: ObjectId,
        directory: String,
    ) -> Result<Vec<ObjectId>, ProjectServiceError> {
        let project = self.accessible_project(project_id, user_id).await?;
        let directory =
            path::normalize_directory(&directory).map_err(ProjectServiceError::InvalidPath)?;

        let removed_storage_keys: Vec<ObjectId> = project
            .files
            .iter()
            .filter(|f| path::is_within(&f.path, &directory))
            .filter_map(|f| match &f.content {
                FileContent::Binary { storage_key } => Some(*storage_key),
                FileContent::Text { .. } => None,
            })
            .collect();
        let entry_removed = project
            .entry
            .and_then(|id| project.files.iter().find(|f| f.id == id))
            .is_some_and(|f| path::is_within(&f.path, &directory));

        match self
            .project_repo
            .delete_directory(project_id, directory)
            .await
        {
            Ok(Some(_)) => {}
            Ok(None) => return Err(ProjectServiceError::ProjectNotFound),
            Err(e) => return Err(ProjectServiceError::Database(e)),
        }

        if entry_removed {
            self.clear_entry(project_id).await?;
        }
        Ok(removed_storage_keys)
    }

    /// Resolve a binary file for download: access-check, then return its
    /// `(storage_key, path)`. `Ok(None)` means the file exists but is text (no
    /// separate bytes to serve); `Err(FileNotFound)` means no such file.
    pub async fn file_for_download(
        &self,
        project_id: ObjectId,
        user_id: ObjectId,
        file_id: ObjectId,
    ) -> Result<Option<(ObjectId, String)>, ProjectServiceError> {
        let project = self.accessible_project(project_id, user_id).await?;
        let file = project
            .files
            .iter()
            .find(|f| f.id == file_id)
            .ok_or(ProjectServiceError::FileNotFound)?;
        match &file.content {
            FileContent::Binary { storage_key } => Ok(Some((*storage_key, file.path.clone()))),
            FileContent::Text { .. } => Ok(None),
        }
    }

    /// Set the compile entry to `file_id`. The file must exist.
    pub async fn set_entry(
        &self,
        project_id: ObjectId,
        user_id: ObjectId,
        file_id: ObjectId,
    ) -> Result<ProjectPayload, ProjectServiceError> {
        let project = self.accessible_project(project_id, user_id).await?;
        if !project.files.iter().any(|f| f.id == file_id) {
            return Err(ProjectServiceError::FileNotFound);
        }
        match self.project_repo.set_entry(project_id, Some(file_id)).await {
            Ok(Some(project)) => Ok(project.into()),
            Ok(None) => Err(ProjectServiceError::ProjectNotFound),
            Err(e) => Err(ProjectServiceError::Database(e)),
        }
    }

    /// Verify that `paths` can all be added as new files, *before* the handler
    /// streams any bytes to object storage — so a rejected upload never leaves
    /// orphaned objects behind. Also enforces access.
    pub async fn check_can_add_files(
        &self,
        project_id: ObjectId,
        user_id: ObjectId,
        paths: &[String],
    ) -> Result<(), ProjectServiceError> {
        let project = self.accessible_project(project_id, user_id).await?;
        validate_new_paths(&project, paths)?;
        Ok(())
    }

    /// Record uploaded files in the project tree. Each [`UploadedFile`] carries
    /// its already-decided content — inline `Text` for a UTF-8 source file
    /// (editable and collaborative, just like a hand-created file) or `Binary`
    /// pointing at bytes the handler already wrote to object storage.
    /// Re-validates paths as a last line of defense.
    pub async fn add_uploaded_files(
        &self,
        project_id: ObjectId,
        user_id: ObjectId,
        uploads: Vec<UploadedFile>,
    ) -> Result<Vec<ProjectFilePayload>, ProjectServiceError> {
        let project = self.accessible_project(project_id, user_id).await?;
        let paths: Vec<String> = uploads.iter().map(|u| u.path.clone()).collect();
        validate_new_paths(&project, &paths)?;

        let new_ids: Vec<ObjectId>;
        let files: Vec<ProjectFile> = {
            let mut ids = Vec::with_capacity(uploads.len());
            let files = uploads
                .into_iter()
                .map(|u| {
                    let id = ObjectId::new();
                    ids.push(id);
                    ProjectFile {
                        id,
                        path: u.path,
                        content: u.content,
                        size: u.size,
                        version: 0,
                        updated_at: OffsetDateTime::now_utc(),
                    }
                })
                .collect();
            new_ids = ids;
            files
        };

        match self.project_repo.add_files(project_id, files).await {
            Ok(Some(project)) => {
                let added: HashSet<ObjectId> = new_ids.into_iter().collect();
                Ok(project
                    .files
                    .into_iter()
                    .filter(|f| added.contains(&f.id))
                    .map(ProjectFilePayload::from)
                    .collect())
            }
            Ok(None) => Err(ProjectServiceError::ProjectNotFound),
            Err(e) => Err(ProjectServiceError::Database(e)),
        }
    }

    /// Access-check and fetch the project document in one step.
    async fn accessible_project(
        &self,
        project_id: ObjectId,
        user_id: ObjectId,
    ) -> Result<Project, ProjectServiceError> {
        match self.accessible(project_id, user_id).await {
            Ok(true) => {}
            Ok(false) => return Err(ProjectServiceError::AccessDenied),
            Err(e) => return Err(e),
        };
        match self.project_repo.find_by_id(project_id).await {
            Ok(Some(project)) => Ok(project),
            Ok(None) => Err(ProjectServiceError::ProjectNotFound),
            Err(e) => Err(ProjectServiceError::Database(e)),
        }
    }

    async fn clear_entry(&self, project_id: ObjectId) -> Result<(), ProjectServiceError> {
        self.project_repo
            .set_entry(project_id, None)
            .await
            .map_err(ProjectServiceError::Database)?;
        Ok(())
    }
}

/// An uploaded file awaiting a metadata row in the project tree. `content` is
/// already resolved by the handler: inline [`FileContent::Text`] for a UTF-8
/// source file, or [`FileContent::Binary`] whose bytes are already in object
/// storage.
pub struct UploadedFile {
    pub path: String,
    pub content: FileContent,
    pub size: i64,
}

/// The set of file paths, and the set of directory paths (explicit ones plus
/// every directory implied by a file or explicit directory), currently
/// occupied in `project`.
fn occupied_paths(project: &Project) -> (HashSet<String>, HashSet<String>) {
    let mut files = HashSet::new();
    let mut dirs = HashSet::new();
    for file in &project.files {
        files.insert(file.path.clone());
        for ancestor in path::ancestor_directories(&file.path) {
            dirs.insert(ancestor);
        }
    }
    for dir in &project.directories {
        dirs.insert(dir.clone());
        for ancestor in path::ancestor_directories(dir) {
            dirs.insert(ancestor);
        }
    }
    (files, dirs)
}

/// Validate that every path in `new_paths` can be added to `project` as a new
/// file — legal shape, no collision with an existing file or directory, no
/// file-vs-directory clash, and no clash among the batch itself. Paths in
/// `new_paths` are assumed already normalized.
fn validate_new_paths(project: &Project, new_paths: &[String]) -> Result<(), ProjectServiceError> {
    let (mut files, mut dirs) = occupied_paths(project);
    for path in new_paths {
        check_file_free(&files, &dirs, path)?;
        // Fold this candidate into the working set so a later path in the same
        // batch sees it (catches an in-batch duplicate or file-under-file).
        files.insert(path.clone());
        for ancestor in path::ancestor_directories(path) {
            dirs.insert(ancestor);
        }
    }
    Ok(())
}

/// A new *file* at `path` is free iff nothing already occupies that exact name
/// (as a file or a directory) and no ancestor segment is itself a file.
fn check_file_free(
    files: &HashSet<String>,
    dirs: &HashSet<String>,
    path: &str,
) -> Result<(), ProjectServiceError> {
    if files.contains(path) || dirs.contains(path) {
        return Err(ProjectServiceError::PathConflict(path.to_string()));
    }
    for ancestor in path::ancestor_directories(path) {
        if files.contains(&ancestor) {
            return Err(ProjectServiceError::PathConflict(ancestor));
        }
    }
    Ok(())
}

/// A new *directory* at `path` is free iff nothing already occupies that exact
/// name (as a file or an already-existing directory) and no ancestor segment
/// is a file.
fn check_directory_free(
    files: &HashSet<String>,
    dirs: &HashSet<String>,
    path: &str,
) -> Result<(), ProjectServiceError> {
    if files.contains(path) || dirs.contains(path) {
        return Err(ProjectServiceError::PathConflict(path.to_string()));
    }
    for ancestor in path::ancestor_directories(path) {
        if files.contains(&ancestor) {
            return Err(ProjectServiceError::PathConflict(ancestor));
        }
    }
    Ok(())
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
            directories: vec![],
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            entry: None,
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
            directories: vec![],
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            entry: None,
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
            directories: vec![],
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            entry: None,
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
            directories: vec![],
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            entry: None,
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
            directories: vec![],
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            entry: None,
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

    fn project_with_file(project_id: ObjectId, owner_id: ObjectId, file_id: ObjectId) -> Project {
        Project {
            id: project_id,
            name: "test".to_string(),
            owner_id,
            owner_type: OwnerType::User,
            creator_id: owner_id,
            files: vec![ProjectFile {
                id: file_id,
                path: "main.typ".to_string(),
                content: FileContent::Text {
                    text: "old".to_string(),
                },
                size: 3,
                version: 1,
                updated_at: OffsetDateTime::now_utc(),
            }],
            directories: vec![],
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            entry: Some(file_id),
            pinned_version: None,
        }
    }

    #[tokio::test]
    async fn test_update_file_success() {
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

        let payload = service
            .update_file(project_id, owner_id, file_id, "new body".to_string())
            .await
            .unwrap();

        assert_eq!(payload.id, file_id.to_hex());
        assert_eq!(payload.version, 2);
    }

    #[tokio::test]
    async fn test_update_file_access_denied() {
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
            .update_file(project_id, other_user_id, file_id, "x".to_string())
            .await;
        assert!(matches!(res, Err(ProjectServiceError::AccessDenied)));
    }

    #[tokio::test]
    async fn test_update_file_not_found() {
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

        // Access passes (owner) but the file id does not exist.
        let res = service
            .update_file(project_id, owner_id, ObjectId::new(), "x".to_string())
            .await;
        assert!(matches!(res, Err(ProjectServiceError::ProjectNotFound)));
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

        let payload = service.duplicate(project_id, creator_id).await.unwrap();

        assert_eq!(payload.name, "test copy");
        assert_eq!(payload.owner_id, creator_id.to_hex());
        assert_eq!(payload.creator_id, creator_id.to_hex());
        assert_ne!(payload.id, project_id.to_hex());
        assert_eq!(payload.files.len(), 1);

        // The duplicated file gets a fresh id, distinct from the source
        // file's, and `entry` is remapped to point at the new one.
        let new_file_id = payload.files[0].id.clone();
        assert_ne!(new_file_id, file_id.to_hex());
        assert_eq!(payload.entry, Some(new_file_id));
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
            files: vec![],
            directories: vec![],
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            entry: None,
            pinned_version: None,
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
        let payload = service.duplicate(project_id, member_id).await.unwrap();
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

        let res = service.duplicate(ObjectId::new(), ObjectId::new()).await;
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

        let res = service.duplicate(project_id, other_user_id).await;
        assert!(matches!(res, Err(ProjectServiceError::AccessDenied)));
    }

    // --- File-tree structural operations -------------------------------------

    use crate::models::project::{FileKind, ProjectFile};

    fn service_with(project: Project) -> ProjectService<MockProjectRepo, MockUserRepo, MockTeamRepo> {
        ProjectService {
            project_repo: MockProjectRepo {
                projects: Mutex::new(vec![project]),
            },
            user_repo: MockUserRepo::default(),
            team_repo: MockTeamRepo::default(),
        }
    }

    fn binary_file(id: ObjectId, path: &str, storage_key: ObjectId) -> ProjectFile {
        ProjectFile {
            id,
            path: path.to_string(),
            content: FileContent::Binary { storage_key },
            size: 10,
            version: 0,
            updated_at: OffsetDateTime::now_utc(),
        }
    }

    #[tokio::test]
    async fn test_create_file_success() {
        let owner = ObjectId::new();
        let pid = ObjectId::new();
        let fid = ObjectId::new();
        let service = service_with(project_with_file(pid, owner, fid));

        let payload = service
            .create_file(pid, owner, "chapters/intro.typ".to_string())
            .await
            .unwrap();
        assert_eq!(payload.path, "chapters/intro.typ");
        assert_eq!(payload.kind, FileKind::Text);
        assert_eq!(payload.version, 0);
    }

    #[tokio::test]
    async fn test_create_file_duplicate_conflicts() {
        let owner = ObjectId::new();
        let pid = ObjectId::new();
        let fid = ObjectId::new();
        let service = service_with(project_with_file(pid, owner, fid));

        let res = service.create_file(pid, owner, "main.typ".to_string()).await;
        assert!(matches!(res, Err(ProjectServiceError::PathConflict(_))));
    }

    #[tokio::test]
    async fn test_create_file_invalid_path() {
        let owner = ObjectId::new();
        let pid = ObjectId::new();
        let fid = ObjectId::new();
        let service = service_with(project_with_file(pid, owner, fid));

        let res = service
            .create_file(pid, owner, "../escape.typ".to_string())
            .await;
        assert!(matches!(res, Err(ProjectServiceError::InvalidPath(_))));
    }

    #[tokio::test]
    async fn test_create_file_access_denied() {
        let owner = ObjectId::new();
        let other = ObjectId::new();
        let pid = ObjectId::new();
        let fid = ObjectId::new();
        let service = service_with(project_with_file(pid, owner, fid));

        let res = service.create_file(pid, other, "a.typ".to_string()).await;
        assert!(matches!(res, Err(ProjectServiceError::AccessDenied)));
    }

    #[tokio::test]
    async fn test_create_file_where_directory_exists_conflicts() {
        let owner = ObjectId::new();
        let pid = ObjectId::new();
        let fid = ObjectId::new();
        let service = service_with(project_with_file(pid, owner, fid));

        // Create a file under `assets/`, making `assets` an implied directory.
        service
            .create_file(pid, owner, "assets/logo.txt".to_string())
            .await
            .unwrap();
        // Now a *file* named `assets` collides with that directory.
        let res = service.create_file(pid, owner, "assets".to_string()).await;
        assert!(matches!(res, Err(ProjectServiceError::PathConflict(_))));
    }

    #[tokio::test]
    async fn test_create_directory_success_and_conflict() {
        let owner = ObjectId::new();
        let pid = ObjectId::new();
        let fid = ObjectId::new();
        let service = service_with(project_with_file(pid, owner, fid));

        let payload = service
            .create_directory(pid, owner, "assets".to_string())
            .await
            .unwrap();
        assert!(payload.directories.contains(&"assets".to_string()));

        // Creating it again conflicts.
        let res = service
            .create_directory(pid, owner, "assets".to_string())
            .await;
        assert!(matches!(res, Err(ProjectServiceError::PathConflict(_))));

        // A directory named after an existing file conflicts too.
        let res = service
            .create_directory(pid, owner, "main.typ".to_string())
            .await;
        assert!(matches!(res, Err(ProjectServiceError::PathConflict(_))));
    }

    #[tokio::test]
    async fn test_rename_file_success_and_noop() {
        let owner = ObjectId::new();
        let pid = ObjectId::new();
        let fid = ObjectId::new();
        let service = service_with(project_with_file(pid, owner, fid));

        let payload = service
            .rename_file(pid, owner, fid, "renamed.typ".to_string())
            .await
            .unwrap();
        assert_eq!(payload.path, "renamed.typ");

        // Renaming to its own (new) path is a no-op that still succeeds.
        let payload = service
            .rename_file(pid, owner, fid, "renamed.typ".to_string())
            .await
            .unwrap();
        assert_eq!(payload.path, "renamed.typ");
    }

    #[tokio::test]
    async fn test_rename_file_conflict_and_missing() {
        let owner = ObjectId::new();
        let pid = ObjectId::new();
        let fid = ObjectId::new();
        let service = service_with(project_with_file(pid, owner, fid));
        service
            .create_file(pid, owner, "other.typ".to_string())
            .await
            .unwrap();

        // Rename main.typ onto the existing other.typ.
        let res = service
            .rename_file(pid, owner, fid, "other.typ".to_string())
            .await;
        assert!(matches!(res, Err(ProjectServiceError::PathConflict(_))));

        // Rename a nonexistent file.
        let res = service
            .rename_file(pid, owner, ObjectId::new(), "x.typ".to_string())
            .await;
        assert!(matches!(res, Err(ProjectServiceError::FileNotFound)));
    }

    #[tokio::test]
    async fn test_delete_file_clears_entry() {
        let owner = ObjectId::new();
        let pid = ObjectId::new();
        let fid = ObjectId::new();
        let service = service_with(project_with_file(pid, owner, fid));

        let removed = service.delete_file(pid, owner, fid).await.unwrap();
        assert_eq!(removed.path, "main.typ");

        // The deleted file was the entry, so the entry is cleared.
        let project = service.project_repo.find_by_id(pid).await.unwrap().unwrap();
        assert!(project.entry.is_none());
        assert!(project.files.is_empty());
    }

    #[tokio::test]
    async fn test_delete_directory_returns_binary_keys_and_clears_entry() {
        let owner = ObjectId::new();
        let pid = ObjectId::new();
        let text_id = ObjectId::new();
        let bin_id = ObjectId::new();
        let storage_key = ObjectId::new();

        let mut project = project_with_file(pid, owner, text_id);
        // Add a binary asset under assets/ and make it the entry.
        project
            .files
            .push(binary_file(bin_id, "assets/logo.png", storage_key));
        project.entry = Some(bin_id);
        project.directories.push("assets".to_string());
        let service = service_with(project);

        let keys = service
            .delete_directory(pid, owner, "assets".to_string())
            .await
            .unwrap();
        assert_eq!(keys, vec![storage_key]);

        let project = service.project_repo.find_by_id(pid).await.unwrap().unwrap();
        assert!(project.entry.is_none());
        assert!(!project.directories.contains(&"assets".to_string()));
        // The text file outside assets/ survives.
        assert!(project.files.iter().any(|f| f.id == text_id));
        assert!(!project.files.iter().any(|f| f.id == bin_id));
    }

    #[tokio::test]
    async fn test_set_entry_success_and_missing() {
        let owner = ObjectId::new();
        let pid = ObjectId::new();
        let fid = ObjectId::new();
        let service = service_with(project_with_file(pid, owner, fid));

        let payload = service.set_entry(pid, owner, fid).await.unwrap();
        assert_eq!(payload.entry, Some(fid.to_hex()));

        let res = service.set_entry(pid, owner, ObjectId::new()).await;
        assert!(matches!(res, Err(ProjectServiceError::FileNotFound)));
    }

    #[tokio::test]
    async fn test_add_uploaded_files_success_and_conflict() {
        let owner = ObjectId::new();
        let pid = ObjectId::new();
        let fid = ObjectId::new();
        let service = service_with(project_with_file(pid, owner, fid));

        let uploads = vec![
            UploadedFile {
                path: "assets/a.png".to_string(),
                content: FileContent::Binary {
                    storage_key: ObjectId::new(),
                },
                size: 5,
            },
            UploadedFile {
                path: "notes/readme.md".to_string(),
                content: FileContent::Text {
                    text: "# hi".to_string(),
                },
                size: 4,
            },
        ];
        let payloads = service
            .add_uploaded_files(pid, owner, uploads)
            .await
            .unwrap();
        assert_eq!(payloads.len(), 2);
        // Binary stays binary; an uploaded text file is stored as editable text.
        assert!(payloads.iter().any(|p| p.kind == FileKind::Binary));
        assert!(payloads.iter().any(|p| p.kind == FileKind::Text));

        // Uploading onto an existing path conflicts.
        let conflict = vec![UploadedFile {
            path: "main.typ".to_string(),
            content: FileContent::Binary {
                storage_key: ObjectId::new(),
            },
            size: 1,
        }];
        let res = service.add_uploaded_files(pid, owner, conflict).await;
        assert!(matches!(res, Err(ProjectServiceError::PathConflict(_))));
    }

    #[tokio::test]
    async fn test_check_can_add_files_detects_intra_batch_duplicate() {
        let owner = ObjectId::new();
        let pid = ObjectId::new();
        let fid = ObjectId::new();
        let service = service_with(project_with_file(pid, owner, fid));

        let dup = vec!["a.png".to_string(), "a.png".to_string()];
        let res = service.check_can_add_files(pid, owner, &dup).await;
        assert!(matches!(res, Err(ProjectServiceError::PathConflict(_))));
    }

    #[tokio::test]
    async fn test_file_for_download_binary_text_and_missing() {
        let owner = ObjectId::new();
        let pid = ObjectId::new();
        let text_id = ObjectId::new();
        let bin_id = ObjectId::new();
        let storage_key = ObjectId::new();

        let mut project = project_with_file(pid, owner, text_id);
        project
            .files
            .push(binary_file(bin_id, "assets/logo.png", storage_key));
        let service = service_with(project);

        let got = service
            .file_for_download(pid, owner, bin_id)
            .await
            .unwrap();
        assert_eq!(got, Some((storage_key, "assets/logo.png".to_string())));

        // A text file has no separate downloadable bytes.
        let got = service.file_for_download(pid, owner, text_id).await.unwrap();
        assert!(got.is_none());

        // A missing file id.
        let res = service.file_for_download(pid, owner, ObjectId::new()).await;
        assert!(matches!(res, Err(ProjectServiceError::FileNotFound)));
    }

    #[test]
    fn test_validate_new_paths_rejects_file_under_file() {
        let owner = ObjectId::new();
        let pid = ObjectId::new();
        let fid = ObjectId::new();
        // Existing file `main.typ`; adding `main.typ/child` would need `main.typ`
        // to be a directory.
        let project = project_with_file(pid, owner, fid);
        let res = validate_new_paths(&project, &["main.typ/child.typ".to_string()]);
        assert!(matches!(res, Err(ProjectServiceError::PathConflict(_))));
    }
}
