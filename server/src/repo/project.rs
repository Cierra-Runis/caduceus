use bson::oid::ObjectId;
use futures_util::TryStreamExt;
use mongodb::error::Result;
use mongodb::options::ReturnDocument;

use crate::models::project::{FileContent, OwnerType, Project, ProjectFile};

#[async_trait::async_trait]
pub trait ProjectRepo {
    async fn create(&self, project: Project) -> Result<Project>;
    async fn find_by_id(&self, id: ObjectId) -> Result<Option<Project>>;
    async fn find_by_owner(
        &self,
        owner_id: ObjectId,
        owner_type: OwnerType,
    ) -> Result<Vec<Project>>;
    /// Replace one file's content, bump its version and `updated_at` (and the
    /// project's), and return the updated project. `None` if the project or the
    /// file does not exist. The file is addressed by its stable id, not path,
    /// so a concurrent rename does not misroute the write.
    async fn update_file_content(
        &self,
        project_id: ObjectId,
        file_id: ObjectId,
        content: FileContent,
        size: i64,
    ) -> Result<Option<Project>>;
    /// Update a project's metadata (name + ownership), bump `updated_at`, and
    /// return the updated project. `None` if the project does not exist.
    async fn update_metadata(
        &self,
        project_id: ObjectId,
        name: String,
        owner_id: ObjectId,
        owner_type: OwnerType,
    ) -> Result<Option<Project>>;
    /// Append one or more files to the project's tree (`$push`, so a concurrent
    /// content flush to a different file is not clobbered), bump `updated_at`,
    /// and return the updated project. `None` if the project does not exist.
    async fn add_files(
        &self,
        project_id: ObjectId,
        files: Vec<ProjectFile>,
    ) -> Result<Option<Project>>;
    /// Record an explicit (possibly empty) directory. Idempotent via
    /// `$addToSet`. `None` if the project does not exist.
    async fn add_directory(
        &self,
        project_id: ObjectId,
        directory: String,
    ) -> Result<Option<Project>>;
    /// Change one file's `path` (rename or move), addressed by its stable id so
    /// a concurrent content write is not misrouted. `None` if the project or
    /// the file does not exist.
    async fn rename_file(
        &self,
        project_id: ObjectId,
        file_id: ObjectId,
        new_path: String,
    ) -> Result<Option<Project>>;
    /// Remove one file by id. Returns the updated project (`Some` whenever the
    /// project exists, even if the file was already gone). The caller is
    /// responsible for freeing any backing object-storage bytes first.
    async fn delete_file(
        &self,
        project_id: ObjectId,
        file_id: ObjectId,
    ) -> Result<Option<Project>>;
    /// Remove a directory and everything under it: every file whose path is
    /// inside `directory`, plus that directory and its explicit sub-directories.
    /// `None` if the project does not exist.
    async fn delete_directory(
        &self,
        project_id: ObjectId,
        directory: String,
    ) -> Result<Option<Project>>;
    /// Set (or clear) the compile entry. `None` if the project does not exist.
    async fn set_entry(
        &self,
        project_id: ObjectId,
        entry: Option<ObjectId>,
    ) -> Result<Option<Project>>;
}

#[derive(Clone)]
pub struct MongoProjectRepo {
    pub collection: mongodb::Collection<Project>,
}

#[async_trait::async_trait]
impl ProjectRepo for MongoProjectRepo {
    async fn create(&self, project: Project) -> Result<Project> {
        let result = self.collection.insert_one(&project).await;
        match result {
            Ok(_) => Ok(project),
            Err(e) => Err(e),
        }
    }

    async fn find_by_id(&self, id: ObjectId) -> Result<Option<Project>> {
        let filter = bson::doc! { "_id": id };
        self.collection.find_one(filter).await
    }

    async fn find_by_owner(
        &self,
        owner_id: ObjectId,
        owner_type: OwnerType,
    ) -> Result<Vec<Project>> {
        let filter = bson::doc! {
            "owner_id": owner_id,
            "owner_type": match owner_type {
                OwnerType::User => "user",
                OwnerType::Team => "team",
            }
        };
        let cursor = self.collection.find(filter).await?;
        let projects: Vec<Project> = cursor.try_collect().await?;
        Ok(projects)
    }

    async fn update_file_content(
        &self,
        project_id: ObjectId,
        file_id: ObjectId,
        content: FileContent,
        size: i64,
    ) -> Result<Option<Project>> {
        let content_bson = bson::to_bson(&content)?;
        let now = bson::DateTime::now();
        let update = bson::doc! {
            "$set": {
                "files.$[f].content": content_bson,
                "files.$[f].size": size,
                "files.$[f].updated_at": now,
                "updated_at": now,
            },
            "$inc": { "files.$[f].version": 1 },
        };

        let updated = self
            .collection
            .find_one_and_update(bson::doc! { "_id": project_id }, update)
            .array_filters(vec![bson::doc! { "f._id": file_id }])
            .return_document(ReturnDocument::After)
            .await?;

        // `array_filters` matching zero elements is not an error: the update
        // still applies to the document (bumping the top-level `updated_at`
        // above) and `find_one_and_update` still returns `Some`, even though
        // nothing in `files` actually changed. Filter that case out here so a
        // nonexistent `file_id` in an existing project is reported as `None`,
        // same as a nonexistent project, per this method's contract.
        Ok(updated.filter(|project| project.files.iter().any(|f| f.id == file_id)))
    }

    async fn update_metadata(
        &self,
        project_id: ObjectId,
        name: String,
        owner_id: ObjectId,
        owner_type: OwnerType,
    ) -> Result<Option<Project>> {
        let update = bson::doc! {
            "$set": {
                "name": name,
                "owner_id": owner_id,
                "owner_type": bson::to_bson(&owner_type)?,
                "updated_at": bson::DateTime::now(),
            },
        };

        self.collection
            .find_one_and_update(bson::doc! { "_id": project_id }, update)
            .return_document(ReturnDocument::After)
            .await
    }

    async fn add_files(
        &self,
        project_id: ObjectId,
        files: Vec<ProjectFile>,
    ) -> Result<Option<Project>> {
        let files_bson = bson::to_bson(&files)?;
        let update = bson::doc! {
            "$push": { "files": { "$each": files_bson } },
            "$set": { "updated_at": bson::DateTime::now() },
        };
        self.collection
            .find_one_and_update(bson::doc! { "_id": project_id }, update)
            .return_document(ReturnDocument::After)
            .await
    }

    async fn add_directory(
        &self,
        project_id: ObjectId,
        directory: String,
    ) -> Result<Option<Project>> {
        let update = bson::doc! {
            "$addToSet": { "directories": directory },
            "$set": { "updated_at": bson::DateTime::now() },
        };
        self.collection
            .find_one_and_update(bson::doc! { "_id": project_id }, update)
            .return_document(ReturnDocument::After)
            .await
    }

    async fn rename_file(
        &self,
        project_id: ObjectId,
        file_id: ObjectId,
        new_path: String,
    ) -> Result<Option<Project>> {
        let now = bson::DateTime::now();
        let update = bson::doc! {
            "$set": {
                "files.$[f].path": new_path,
                "files.$[f].updated_at": now,
                "updated_at": now,
            },
        };
        let updated = self
            .collection
            .find_one_and_update(bson::doc! { "_id": project_id }, update)
            .array_filters(vec![bson::doc! { "f._id": file_id }])
            .return_document(ReturnDocument::After)
            .await?;

        // Same guard as `update_file_content`: a zero-match `array_filters`
        // still returns the (top-level bumped) document, so a nonexistent
        // `file_id` must be reported as `None`, not a silent success.
        Ok(updated.filter(|project| project.files.iter().any(|f| f.id == file_id)))
    }

    async fn delete_file(
        &self,
        project_id: ObjectId,
        file_id: ObjectId,
    ) -> Result<Option<Project>> {
        let update = bson::doc! {
            "$pull": { "files": { "_id": file_id } },
            "$set": { "updated_at": bson::DateTime::now() },
        };
        self.collection
            .find_one_and_update(bson::doc! { "_id": project_id }, update)
            .return_document(ReturnDocument::After)
            .await
    }

    async fn delete_directory(
        &self,
        project_id: ObjectId,
        directory: String,
    ) -> Result<Option<Project>> {
        let escaped = regex::escape(&directory);
        // Files strictly inside the directory (prefixed by `directory/`), and
        // the directory itself plus any explicit sub-directory (`directory` or
        // `directory/…`).
        let files_pattern = format!("^{escaped}/");
        let dirs_pattern = format!("^{escaped}(/|$)");
        let update = bson::doc! {
            "$pull": {
                "files": { "path": { "$regex": files_pattern } },
                "directories": { "$regex": dirs_pattern },
            },
            "$set": { "updated_at": bson::DateTime::now() },
        };
        self.collection
            .find_one_and_update(bson::doc! { "_id": project_id }, update)
            .return_document(ReturnDocument::After)
            .await
    }

    async fn set_entry(
        &self,
        project_id: ObjectId,
        entry: Option<ObjectId>,
    ) -> Result<Option<Project>> {
        let update = bson::doc! {
            "$set": {
                "entry": entry,
                "updated_at": bson::DateTime::now(),
            },
        };
        self.collection
            .find_one_and_update(bson::doc! { "_id": project_id }, update)
            .return_document(ReturnDocument::After)
            .await
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod tests {
    use super::*;
    use crate::config;
    use std::sync::Mutex;
    use time::OffsetDateTime;

    #[derive(Default)]
    pub struct MockProjectRepo {
        pub projects: Mutex<Vec<Project>>,
    }

    #[async_trait::async_trait]
    impl ProjectRepo for MockProjectRepo {
        async fn create(&self, project: Project) -> Result<Project> {
            let mut projects = self.projects.lock().unwrap();
            projects.push(project.clone());
            Ok(project)
        }

        async fn find_by_id(&self, id: ObjectId) -> Result<Option<Project>> {
            let projects = self.projects.lock().unwrap();
            Ok(projects.iter().find(|p| p.id == id).cloned())
        }

        async fn find_by_owner(
            &self,
            owner_id: ObjectId,
            owner_type: OwnerType,
        ) -> Result<Vec<Project>> {
            let projects = self.projects.lock().unwrap();
            let filtered_projects: Vec<Project> = projects
                .iter()
                .filter(|p| p.owner_id == owner_id && p.owner_type == owner_type)
                .cloned()
                .collect();
            Ok(filtered_projects)
        }

        async fn update_file_content(
            &self,
            project_id: ObjectId,
            file_id: ObjectId,
            content: FileContent,
            size: i64,
        ) -> Result<Option<Project>> {
            let mut projects = self.projects.lock().unwrap();
            let Some(project) = projects.iter_mut().find(|p| p.id == project_id) else {
                return Ok(None);
            };
            let Some(file) = project.files.iter_mut().find(|f| f.id == file_id) else {
                return Ok(None);
            };
            file.content = content;
            file.size = size;
            file.version += 1;
            file.updated_at = OffsetDateTime::now_utc();
            project.updated_at = OffsetDateTime::now_utc();
            Ok(Some(project.clone()))
        }

        async fn update_metadata(
            &self,
            project_id: ObjectId,
            name: String,
            owner_id: ObjectId,
            owner_type: OwnerType,
        ) -> Result<Option<Project>> {
            let mut projects = self.projects.lock().unwrap();
            let Some(project) = projects.iter_mut().find(|p| p.id == project_id) else {
                return Ok(None);
            };
            project.name = name;
            project.owner_id = owner_id;
            project.owner_type = owner_type;
            project.updated_at = OffsetDateTime::now_utc();
            Ok(Some(project.clone()))
        }

        async fn add_files(
            &self,
            project_id: ObjectId,
            files: Vec<ProjectFile>,
        ) -> Result<Option<Project>> {
            let mut projects = self.projects.lock().unwrap();
            let Some(project) = projects.iter_mut().find(|p| p.id == project_id) else {
                return Ok(None);
            };
            project.files.extend(files);
            project.updated_at = OffsetDateTime::now_utc();
            Ok(Some(project.clone()))
        }

        async fn add_directory(
            &self,
            project_id: ObjectId,
            directory: String,
        ) -> Result<Option<Project>> {
            let mut projects = self.projects.lock().unwrap();
            let Some(project) = projects.iter_mut().find(|p| p.id == project_id) else {
                return Ok(None);
            };
            if !project.directories.contains(&directory) {
                project.directories.push(directory);
            }
            project.updated_at = OffsetDateTime::now_utc();
            Ok(Some(project.clone()))
        }

        async fn rename_file(
            &self,
            project_id: ObjectId,
            file_id: ObjectId,
            new_path: String,
        ) -> Result<Option<Project>> {
            let mut projects = self.projects.lock().unwrap();
            let Some(project) = projects.iter_mut().find(|p| p.id == project_id) else {
                return Ok(None);
            };
            let Some(file) = project.files.iter_mut().find(|f| f.id == file_id) else {
                return Ok(None);
            };
            file.path = new_path;
            file.updated_at = OffsetDateTime::now_utc();
            project.updated_at = OffsetDateTime::now_utc();
            Ok(Some(project.clone()))
        }

        async fn delete_file(
            &self,
            project_id: ObjectId,
            file_id: ObjectId,
        ) -> Result<Option<Project>> {
            let mut projects = self.projects.lock().unwrap();
            let Some(project) = projects.iter_mut().find(|p| p.id == project_id) else {
                return Ok(None);
            };
            project.files.retain(|f| f.id != file_id);
            project.updated_at = OffsetDateTime::now_utc();
            Ok(Some(project.clone()))
        }

        async fn delete_directory(
            &self,
            project_id: ObjectId,
            directory: String,
        ) -> Result<Option<Project>> {
            let mut projects = self.projects.lock().unwrap();
            let Some(project) = projects.iter_mut().find(|p| p.id == project_id) else {
                return Ok(None);
            };
            let prefix = format!("{directory}/");
            project.files.retain(|f| !f.path.starts_with(&prefix));
            project
                .directories
                .retain(|d| *d != directory && !d.starts_with(&prefix));
            project.updated_at = OffsetDateTime::now_utc();
            Ok(Some(project.clone()))
        }

        async fn set_entry(
            &self,
            project_id: ObjectId,
            entry: Option<ObjectId>,
        ) -> Result<Option<Project>> {
            let mut projects = self.projects.lock().unwrap();
            let Some(project) = projects.iter_mut().find(|p| p.id == project_id) else {
                return Ok(None);
            };
            project.entry = entry;
            project.updated_at = OffsetDateTime::now_utc();
            Ok(Some(project.clone()))
        }
    }

    use crate::models::project::ProjectFile;

    async fn test_repo() -> MongoProjectRepo {
        let config = config::Config::load("config/test.yaml").unwrap();
        let client = mongodb::Client::with_uri_str(config.mongo_uri)
            .await
            .unwrap();
        MongoProjectRepo {
            collection: client
                .database(&config.db_name)
                .collection::<Project>("projects"),
        }
    }

    fn new_project(owner_id: ObjectId, owner_type: OwnerType, files: Vec<ProjectFile>) -> Project {
        Project {
            id: ObjectId::new(),
            name: format!("Test Project {}", ObjectId::new().to_hex()),
            owner_id,
            owner_type,
            creator_id: ObjectId::new(),
            files,
            directories: vec![],
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            entry: None,
            pinned_version: None,
        }
    }

    async fn cleanup(repo: &MongoProjectRepo, id: ObjectId) {
        let _ = repo.collection.delete_one(bson::doc! { "_id": id }).await;
    }

    #[tokio::test]
    #[ignore = "requires a live MongoDB (provisioned in CI; run locally with cargo test -- --ignored)"]
    async fn test_create_and_find_by_id() {
        let repo = test_repo().await;
        let project = new_project(ObjectId::new(), OwnerType::User, vec![]);

        let created = repo.create(project.clone()).await.unwrap();
        assert_eq!(created.id, project.id);
        assert_eq!(created.name, project.name);

        let found = repo.find_by_id(project.id).await.unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.id, project.id);
        assert_eq!(found.name, project.name);
        assert_eq!(found.owner_id, project.owner_id);
        assert_eq!(found.owner_type, project.owner_type);
        assert_eq!(found.creator_id, project.creator_id);

        cleanup(&repo, project.id).await;
    }

    #[tokio::test]
    #[ignore = "requires a live MongoDB (provisioned in CI; run locally with cargo test -- --ignored)"]
    async fn test_find_by_id_not_found() {
        let repo = test_repo().await;
        let found = repo.find_by_id(ObjectId::new()).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    #[ignore = "requires a live MongoDB (provisioned in CI; run locally with cargo test -- --ignored)"]
    async fn test_find_by_owner_matches_owner_id_and_owner_type() {
        let repo = test_repo().await;
        let owner_id = ObjectId::new();
        let other_owner_id = ObjectId::new();

        let matching = new_project(owner_id, OwnerType::User, vec![]);
        let same_owner_different_type = new_project(owner_id, OwnerType::Team, vec![]);
        let same_type_different_owner = new_project(other_owner_id, OwnerType::User, vec![]);

        repo.create(matching.clone()).await.unwrap();
        repo.create(same_owner_different_type.clone())
            .await
            .unwrap();
        repo.create(same_type_different_owner.clone())
            .await
            .unwrap();

        let found = repo.find_by_owner(owner_id, OwnerType::User).await.unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id, matching.id);

        cleanup(&repo, matching.id).await;
        cleanup(&repo, same_owner_different_type.id).await;
        cleanup(&repo, same_type_different_owner.id).await;
    }

    #[tokio::test]
    #[ignore = "requires a live MongoDB (provisioned in CI; run locally with cargo test -- --ignored)"]
    async fn test_find_by_owner_empty_when_no_match() {
        let repo = test_repo().await;
        let found = repo
            .find_by_owner(ObjectId::new(), OwnerType::Team)
            .await
            .unwrap();
        assert!(found.is_empty());
    }

    #[tokio::test]
    #[ignore = "requires a live MongoDB (provisioned in CI; run locally with cargo test -- --ignored)"]
    async fn test_update_file_content_bumps_version_and_timestamps() {
        let repo = test_repo().await;
        let file = ProjectFile {
            id: ObjectId::new(),
            path: "main.typ".to_string(),
            content: FileContent::Text {
                text: "original".to_string(),
            },
            size: 8,
            version: 0,
            updated_at: OffsetDateTime::now_utc(),
        };
        let project = new_project(ObjectId::new(), OwnerType::User, vec![file.clone()]);
        repo.create(project.clone()).await.unwrap();

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let new_content = FileContent::Text {
            text: "updated content".to_string(),
        };
        let updated = repo
            .update_file_content(project.id, file.id, new_content, 16)
            .await
            .unwrap();

        assert!(updated.is_some());
        let updated = updated.unwrap();
        assert!(updated.updated_at > project.updated_at);

        let updated_file = updated.files.iter().find(|f| f.id == file.id).unwrap();
        assert_eq!(updated_file.version, file.version + 1);
        assert!(updated_file.updated_at > file.updated_at);
        assert_eq!(updated_file.size, 16);
        match &updated_file.content {
            FileContent::Text { text } => assert_eq!(text, "updated content"),
            FileContent::Binary { .. } => panic!("expected text content"),
        }

        cleanup(&repo, project.id).await;
    }

    #[tokio::test]
    #[ignore = "requires a live MongoDB (provisioned in CI; run locally with cargo test -- --ignored)"]
    async fn test_update_file_content_returns_none_for_missing_project() {
        let repo = test_repo().await;
        let result = repo
            .update_file_content(
                ObjectId::new(),
                ObjectId::new(),
                FileContent::Text {
                    text: "x".to_string(),
                },
                1,
            )
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    #[ignore = "requires a live MongoDB (provisioned in CI; run locally with cargo test -- --ignored)"]
    async fn test_update_metadata_updates_fields_and_timestamp() {
        let repo = test_repo().await;
        let project = new_project(ObjectId::new(), OwnerType::User, vec![]);
        repo.create(project.clone()).await.unwrap();

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let new_owner_id = ObjectId::new();
        let updated = repo
            .update_metadata(
                project.id,
                "renamed".to_string(),
                new_owner_id,
                OwnerType::Team,
            )
            .await
            .unwrap();

        assert!(updated.is_some());
        let updated = updated.unwrap();
        assert_eq!(updated.name, "renamed");
        assert_eq!(updated.owner_id, new_owner_id);
        assert_eq!(updated.owner_type, OwnerType::Team);
        assert!(updated.updated_at > project.updated_at);

        cleanup(&repo, project.id).await;
    }

    #[tokio::test]
    #[ignore = "requires a live MongoDB (provisioned in CI; run locally with cargo test -- --ignored)"]
    async fn test_update_metadata_returns_none_for_missing_project() {
        let repo = test_repo().await;
        let result = repo
            .update_metadata(
                ObjectId::new(),
                "x".to_string(),
                ObjectId::new(),
                OwnerType::User,
            )
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    #[ignore = "requires a live MongoDB (provisioned in CI; run locally with cargo test -- --ignored)"]
    async fn test_update_file_content_returns_none_for_missing_file() {
        let repo = test_repo().await;
        let project = new_project(ObjectId::new(), OwnerType::User, vec![]);
        repo.create(project.clone()).await.unwrap();

        let result = repo
            .update_file_content(
                project.id,
                ObjectId::new(),
                FileContent::Text {
                    text: "x".to_string(),
                },
                1,
            )
            .await
            .unwrap();
        assert!(result.is_none());

        cleanup(&repo, project.id).await;
    }

    fn text_file(path: &str) -> ProjectFile {
        ProjectFile {
            id: ObjectId::new(),
            path: path.to_string(),
            content: FileContent::Text {
                text: String::new(),
            },
            size: 0,
            version: 0,
            updated_at: OffsetDateTime::now_utc(),
        }
    }

    #[tokio::test]
    #[ignore = "requires a live MongoDB (provisioned in CI; run locally with cargo test -- --ignored)"]
    async fn test_add_files_and_directory() {
        let repo = test_repo().await;
        let project = new_project(ObjectId::new(), OwnerType::User, vec![]);
        repo.create(project.clone()).await.unwrap();

        let a = text_file("a.typ");
        let b = text_file("chapters/b.typ");
        let updated = repo
            .add_files(project.id, vec![a.clone(), b.clone()])
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.files.len(), 2);
        assert!(updated.files.iter().any(|f| f.id == a.id));
        assert!(updated.files.iter().any(|f| f.id == b.id));

        // Adding a directory is idempotent.
        repo.add_directory(project.id, "assets".to_string())
            .await
            .unwrap();
        let updated = repo
            .add_directory(project.id, "assets".to_string())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            updated
                .directories
                .iter()
                .filter(|d| *d == "assets")
                .count(),
            1
        );

        cleanup(&repo, project.id).await;
    }

    #[tokio::test]
    #[ignore = "requires a live MongoDB (provisioned in CI; run locally with cargo test -- --ignored)"]
    async fn test_rename_file_changes_path_and_reports_missing() {
        let repo = test_repo().await;
        let file = text_file("main.typ");
        let project = new_project(ObjectId::new(), OwnerType::User, vec![file.clone()]);
        repo.create(project.clone()).await.unwrap();

        let updated = repo
            .rename_file(project.id, file.id, "renamed.typ".to_string())
            .await
            .unwrap()
            .unwrap();
        let renamed = updated.files.iter().find(|f| f.id == file.id).unwrap();
        assert_eq!(renamed.path, "renamed.typ");

        // Renaming a nonexistent file reports None (array filter matched zero).
        let missing = repo
            .rename_file(project.id, ObjectId::new(), "x.typ".to_string())
            .await
            .unwrap();
        assert!(missing.is_none());

        cleanup(&repo, project.id).await;
    }

    #[tokio::test]
    #[ignore = "requires a live MongoDB (provisioned in CI; run locally with cargo test -- --ignored)"]
    async fn test_delete_file_removes_one() {
        let repo = test_repo().await;
        let keep = text_file("keep.typ");
        let drop = text_file("drop.typ");
        let project =
            new_project(ObjectId::new(), OwnerType::User, vec![keep.clone(), drop.clone()]);
        repo.create(project.clone()).await.unwrap();

        let updated = repo
            .delete_file(project.id, drop.id)
            .await
            .unwrap()
            .unwrap();
        assert!(updated.files.iter().any(|f| f.id == keep.id));
        assert!(!updated.files.iter().any(|f| f.id == drop.id));

        cleanup(&repo, project.id).await;
    }

    #[tokio::test]
    #[ignore = "requires a live MongoDB (provisioned in CI; run locally with cargo test -- --ignored)"]
    async fn test_delete_directory_prunes_subtree_only() {
        let repo = test_repo().await;
        let root = text_file("root.typ");
        let inside = text_file("assets/logo.txt");
        let deeper = text_file("assets/img/pic.txt");
        let sibling = text_file("assets2/other.txt");
        let mut project = new_project(
            ObjectId::new(),
            OwnerType::User,
            vec![
                root.clone(),
                inside.clone(),
                deeper.clone(),
                sibling.clone(),
            ],
        );
        project.directories = vec![
            "assets".to_string(),
            "assets/img".to_string(),
            "assets2".to_string(),
        ];
        repo.create(project.clone()).await.unwrap();

        let updated = repo
            .delete_directory(project.id, "assets".to_string())
            .await
            .unwrap()
            .unwrap();

        // Files and directories under `assets/` are gone; the similarly-named
        // `assets2` and the root file survive (prefix must be `assets/`, not
        // just `assets`).
        assert!(updated.files.iter().any(|f| f.id == root.id));
        assert!(updated.files.iter().any(|f| f.id == sibling.id));
        assert!(!updated.files.iter().any(|f| f.id == inside.id));
        assert!(!updated.files.iter().any(|f| f.id == deeper.id));
        assert!(!updated.directories.contains(&"assets".to_string()));
        assert!(!updated.directories.contains(&"assets/img".to_string()));
        assert!(updated.directories.contains(&"assets2".to_string()));

        cleanup(&repo, project.id).await;
    }

    #[tokio::test]
    #[ignore = "requires a live MongoDB (provisioned in CI; run locally with cargo test -- --ignored)"]
    async fn test_set_entry_sets_and_clears() {
        let repo = test_repo().await;
        let file = text_file("main.typ");
        let project = new_project(ObjectId::new(), OwnerType::User, vec![file.clone()]);
        repo.create(project.clone()).await.unwrap();

        let updated = repo
            .set_entry(project.id, Some(file.id))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.entry, Some(file.id));

        let updated = repo.set_entry(project.id, None).await.unwrap().unwrap();
        assert!(updated.entry.is_none());

        cleanup(&repo, project.id).await;
    }
}
