use bson::oid::ObjectId;
use derive_more::Display;
use time::OffsetDateTime;

use crate::models::project::{FileContent, ProjectFile, ProjectFileDetailPayload};
use crate::repo::asset::{AssetStore, StoredAsset};
use crate::repo::{project::ProjectRepo, team::TeamRepo};
use crate::services::project::{ProjectServiceError, load_accessible};

/// Errors from the asset service. Access errors are shared with the project
/// service (same access rule via [`load_accessible`]); the rest are specific to
/// binary assets.
#[derive(Debug, Display)]
pub enum AssetServiceError {
    #[display("Project not found")]
    ProjectNotFound,
    #[display("Access denied: You do not have permission to access this project")]
    AccessDenied,
    #[display("File not found")]
    FileNotFound,
    #[display("File is not a binary asset")]
    NotBinary,
    #[display("Storage error: {_0}")]
    Storage(String),
}

impl From<ProjectServiceError> for AssetServiceError {
    fn from(err: ProjectServiceError) -> Self {
        match err {
            ProjectServiceError::AccessDenied => AssetServiceError::AccessDenied,
            ProjectServiceError::Database(e) => AssetServiceError::Storage(e.to_string()),
            // ProjectNotFound, plus a dangling team ref (OwnerNotFound) which,
            // from the asset API's view, means the project can't be resolved.
            _ => AssetServiceError::ProjectNotFound,
        }
    }
}

pub struct AssetService<P: ProjectRepo, T: TeamRepo, S: AssetStore> {
    pub project_repo: P,
    pub team_repo: T,
    pub asset_store: S,
}

impl<P: ProjectRepo, T: TeamRepo, S: AssetStore> AssetService<P, T, S> {
    /// Store an uploaded binary and attach it to the project as a new
    /// [`FileContent::Binary`] file. The caller must have access. Returns the
    /// new file's metadata (id, path, storage key, size) so the client can
    /// reference it — e.g. from a Typst `#image(...)` — and fetch it back.
    pub async fn upload_asset(
        &self,
        project_id: ObjectId,
        user_id: ObjectId,
        path: String,
        content_type: Option<String>,
        bytes: Vec<u8>,
    ) -> Result<ProjectFileDetailPayload, AssetServiceError> {
        load_accessible(&self.project_repo, &self.team_repo, project_id, user_id).await?;

        let size = bytes.len() as i64;
        let storage_key = self
            .asset_store
            .upload(&path, content_type.as_deref(), bytes)
            .await
            .map_err(|e| AssetServiceError::Storage(e.to_string()))?;

        let file = ProjectFile {
            id: ObjectId::new(),
            path,
            content: FileContent::Binary { storage_key },
            size,
            version: 0,
            updated_at: OffsetDateTime::now_utc(),
        };

        match self.project_repo.add_file(project_id, file.clone()).await {
            Ok(Some(_)) => Ok(file.into()),
            // The project vanished between the access check and the write.
            Ok(None) => Err(AssetServiceError::ProjectNotFound),
            Err(e) => Err(AssetServiceError::Storage(e.to_string())),
        }
    }

    /// Fetch a binary asset's bytes (plus content type / filename for serving).
    /// The caller must have access to the project the file belongs to; a text
    /// file id is rejected as [`AssetServiceError::NotBinary`].
    pub async fn read_asset(
        &self,
        project_id: ObjectId,
        user_id: ObjectId,
        file_id: ObjectId,
    ) -> Result<StoredAsset, AssetServiceError> {
        let project =
            load_accessible(&self.project_repo, &self.team_repo, project_id, user_id).await?;

        let file = project
            .files
            .into_iter()
            .find(|f| f.id == file_id)
            .ok_or(AssetServiceError::FileNotFound)?;

        let storage_key = match file.content {
            FileContent::Binary { storage_key } => storage_key,
            FileContent::Text { .. } => return Err(AssetServiceError::NotBinary),
        };

        match self.asset_store.download(&storage_key).await {
            Ok(Some(asset)) => Ok(asset),
            // The file row references a key the store no longer has.
            Ok(None) => Err(AssetServiceError::FileNotFound),
            Err(e) => Err(AssetServiceError::Storage(e.to_string())),
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::models::project::{FileContentPayload, OwnerType, Project};
    use crate::repo::asset::tests::MockAssetStore;
    use crate::repo::project::tests::MockProjectRepo;
    use crate::repo::team::tests::MockTeamRepo;
    use std::sync::Mutex;

    fn project_with_text_file(
        project_id: ObjectId,
        owner_id: ObjectId,
        file_id: ObjectId,
    ) -> Project {
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
                    text: "hello".to_string(),
                },
                size: 5,
                version: 1,
                updated_at: OffsetDateTime::now_utc(),
            }],
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            entry: Some(file_id),
            pinned_version: None,
        }
    }

    fn service(
        project: Project,
    ) -> AssetService<MockProjectRepo, MockTeamRepo, MockAssetStore> {
        AssetService {
            project_repo: MockProjectRepo {
                projects: Mutex::new(vec![project]),
            },
            team_repo: MockTeamRepo::default(),
            asset_store: MockAssetStore::default(),
        }
    }

    #[tokio::test]
    async fn test_upload_then_read_roundtrip() {
        let owner_id = ObjectId::new();
        let project_id = ObjectId::new();
        let file_id = ObjectId::new();
        let service = service(project_with_text_file(project_id, owner_id, file_id));

        // Upload a binary asset.
        let payload = service
            .upload_asset(
                project_id,
                owner_id,
                "logo.png".to_string(),
                Some("image/png".to_string()),
                vec![9, 8, 7],
            )
            .await
            .unwrap();

        assert_eq!(payload.path, "logo.png");
        assert_eq!(payload.size, 3);
        assert_eq!(payload.version, 0);
        let storage_key = match payload.content {
            FileContentPayload::Binary { storage_key } => storage_key,
            FileContentPayload::Text { .. } => panic!("expected binary content"),
        };
        assert!(!storage_key.is_empty());

        // Read it back by its new file id.
        let new_file_id = ObjectId::parse_str(&payload.id).unwrap();
        let asset = service
            .read_asset(project_id, owner_id, new_file_id)
            .await
            .unwrap();
        assert_eq!(asset.bytes, vec![9, 8, 7]);
        assert_eq!(asset.content_type.as_deref(), Some("image/png"));
        assert_eq!(asset.filename, "logo.png");
    }

    #[tokio::test]
    async fn test_upload_access_denied() {
        let owner_id = ObjectId::new();
        let stranger = ObjectId::new();
        let project_id = ObjectId::new();
        let file_id = ObjectId::new();
        let service = service(project_with_text_file(project_id, owner_id, file_id));

        let res = service
            .upload_asset(
                project_id,
                stranger,
                "logo.png".to_string(),
                None,
                vec![1],
            )
            .await;
        assert!(matches!(res, Err(AssetServiceError::AccessDenied)));
    }

    #[tokio::test]
    async fn test_upload_project_not_found() {
        let service = service(project_with_text_file(
            ObjectId::new(),
            ObjectId::new(),
            ObjectId::new(),
        ));
        let res = service
            .upload_asset(
                ObjectId::new(),
                ObjectId::new(),
                "logo.png".to_string(),
                None,
                vec![1],
            )
            .await;
        assert!(matches!(res, Err(AssetServiceError::ProjectNotFound)));
    }

    #[tokio::test]
    async fn test_read_asset_rejects_text_file() {
        let owner_id = ObjectId::new();
        let project_id = ObjectId::new();
        let file_id = ObjectId::new();
        let service = service(project_with_text_file(project_id, owner_id, file_id));

        // The seeded file is text, not a binary asset.
        let res = service.read_asset(project_id, owner_id, file_id).await;
        assert!(matches!(res, Err(AssetServiceError::NotBinary)));
    }

    #[tokio::test]
    async fn test_read_asset_missing_file() {
        let owner_id = ObjectId::new();
        let project_id = ObjectId::new();
        let file_id = ObjectId::new();
        let service = service(project_with_text_file(project_id, owner_id, file_id));

        let res = service
            .read_asset(project_id, owner_id, ObjectId::new())
            .await;
        assert!(matches!(res, Err(AssetServiceError::FileNotFound)));
    }

    #[tokio::test]
    async fn test_read_asset_access_denied() {
        let owner_id = ObjectId::new();
        let stranger = ObjectId::new();
        let project_id = ObjectId::new();
        let file_id = ObjectId::new();
        let service = service(project_with_text_file(project_id, owner_id, file_id));

        let res = service.read_asset(project_id, stranger, file_id).await;
        assert!(matches!(res, Err(AssetServiceError::AccessDenied)));
    }
}
