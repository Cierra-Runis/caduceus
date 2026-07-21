use bson::oid::ObjectId;
use futures_util::TryStreamExt;
use mongodb::error::Result;
use mongodb::options::ReturnDocument;

use crate::models::project::{FileContent, OwnerType, Project};

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

        self.collection
            .find_one_and_update(bson::doc! { "_id": project_id }, update)
            .array_filters(vec![bson::doc! { "f._id": file_id }])
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
    async fn test_find_by_id_not_found() {
        let repo = test_repo().await;
        let found = repo.find_by_id(ObjectId::new()).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
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
    async fn test_find_by_owner_empty_when_no_match() {
        let repo = test_repo().await;
        let found = repo
            .find_by_owner(ObjectId::new(), OwnerType::Team)
            .await
            .unwrap();
        assert!(found.is_empty());
    }

    #[tokio::test]
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
}
