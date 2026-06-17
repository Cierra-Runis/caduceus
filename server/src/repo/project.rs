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

    #[tokio::test]
    async fn test_mongo_project_repo() {
        let config = config::Config::load("config/test.yaml").unwrap();
        let repo = MongoProjectRepo {
            collection: mongodb::Client::with_uri_str(config.mongo_uri)
                .await
                .unwrap()
                .database("test_db")
                .collection::<Project>("projects"),
        };

        let project = Project {
            id: ObjectId::new(),
            name: "Test Project".to_string(),
            owner_id: ObjectId::new(),
            owner_type: OwnerType::User,
            creator_id: ObjectId::new(),
            files: vec![],
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            entry: None,
            pinned_version: None,
        };

        // Test create
        let created_project = repo.create(project.clone()).await.unwrap();
        assert_eq!(created_project.name, project.name);

        // Test find_by_id
        let found_project = repo.find_by_id(created_project.id).await.unwrap();
        assert!(found_project.is_some());
        assert_eq!(found_project.unwrap().name, project.name);

        // Test find_by_owner
        let found_projects = repo
            .find_by_owner(project.owner_id, OwnerType::User)
            .await
            .unwrap();
        assert!(!found_projects.is_empty());
        assert_eq!(found_projects[0].owner_id, project.owner_id);
    }
}
