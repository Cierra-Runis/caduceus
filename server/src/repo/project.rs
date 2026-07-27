use bson::oid::ObjectId;
use futures_util::TryStreamExt;
use mongodb::error::Result;
use mongodb::options::ReturnDocument;

use std::collections::HashMap;

use crate::models::project::{OwnerType, Project, ProjectSettings};
use crate::models::tree::{NodeId, ProjectionEntry};

#[async_trait::async_trait]
pub trait ProjectRepo {
    async fn create(&self, project: Project) -> Result<Project>;
    async fn find_by_id(&self, id: ObjectId) -> Result<Option<Project>>;
    async fn find_by_owner(
        &self,
        owner_id: ObjectId,
        owner_type: OwnerType,
    ) -> Result<Vec<Project>>;
    /// Update a project's metadata (name + ownership), bump `updated_at`, and
    /// return the updated project. `None` if the project does not exist.
    async fn update_metadata(
        &self,
        project_id: ObjectId,
        name: String,
        owner_id: ObjectId,
        owner_type: OwnerType,
    ) -> Result<Option<Project>>;
    /// Overwrite a project's editor settings (auto-save policy, …), bump
    /// `updated_at`, and return the updated project. `None` if it does not
    /// exist.
    async fn update_settings(
        &self,
        project_id: ObjectId,
        settings: ProjectSettings,
    ) -> Result<Option<Project>>;
    /// Overwrite a project's stored tree projection (`Project::tree`) — the
    /// id-keyed cache of the CRDT file tree, refreshed by the room on persist.
    /// The authoritative structure lives in the Y.Doc snapshot; this is only the
    /// listing cache, so it deliberately does not bump `updated_at`.
    async fn update_tree(
        &self,
        project_id: ObjectId,
        tree: HashMap<NodeId, ProjectionEntry>,
    ) -> Result<()>;
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

    async fn update_settings(
        &self,
        project_id: ObjectId,
        settings: ProjectSettings,
    ) -> Result<Option<Project>> {
        let update = bson::doc! {
            "$set": {
                "settings": bson::to_bson(&settings)?,
                "updated_at": bson::DateTime::now(),
            },
        };

        self.collection
            .find_one_and_update(bson::doc! { "_id": project_id }, update)
            .return_document(ReturnDocument::After)
            .await
    }

    async fn update_tree(
        &self,
        project_id: ObjectId,
        tree: HashMap<NodeId, ProjectionEntry>,
    ) -> Result<()> {
        let update = bson::doc! { "$set": { "tree": bson::to_bson(&tree)? } };
        self.collection
            .update_one(bson::doc! { "_id": project_id }, update)
            .await?;
        Ok(())
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

        async fn update_settings(
            &self,
            project_id: ObjectId,
            settings: ProjectSettings,
        ) -> Result<Option<Project>> {
            let mut projects = self.projects.lock().unwrap();
            let Some(project) = projects.iter_mut().find(|p| p.id == project_id) else {
                return Ok(None);
            };
            project.settings = settings;
            project.updated_at = OffsetDateTime::now_utc();
            Ok(Some(project.clone()))
        }

        async fn update_tree(
            &self,
            project_id: ObjectId,
            tree: HashMap<NodeId, ProjectionEntry>,
        ) -> Result<()> {
            let mut projects = self.projects.lock().unwrap();
            if let Some(project) = projects.iter_mut().find(|p| p.id == project_id) {
                project.tree = tree;
            }
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_update_tree_persists_the_projection_on_the_field() {
        use crate::models::tree::NodeContent;

        let project_id = ObjectId::new();
        let repo = MockProjectRepo {
            projects: Mutex::new(vec![new_project(ObjectId::new(), OwnerType::User)]),
        };
        // Point the seeded project's id at a known value.
        repo.projects.lock().unwrap()[0].id = project_id;

        let mut tree = HashMap::new();
        tree.insert(
            "chapters".to_string(),
            ProjectionEntry {
                parent: None,
                name: "chapters".to_string(),
                path: "chapters".to_string(),
                content: NodeContent::Folder,
            },
        );
        repo.update_tree(project_id, tree.clone()).await.unwrap();

        let stored = repo.find_by_id(project_id).await.unwrap().unwrap();
        assert_eq!(stored.tree, tree);
    }

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

    fn new_project(owner_id: ObjectId, owner_type: OwnerType) -> Project {
        Project {
            id: ObjectId::new(),
            name: format!("Test Project {}", ObjectId::new().to_hex()),
            owner_id,
            owner_type,
            creator_id: ObjectId::new(),
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            entry: None,
            pinned_version: None,
            settings: ProjectSettings::default(),
            tree: Default::default(),
        }
    }

    async fn cleanup(repo: &MongoProjectRepo, id: ObjectId) {
        let _ = repo.collection.delete_one(bson::doc! { "_id": id }).await;
    }

    #[tokio::test]
    #[ignore = "requires a live MongoDB (provisioned in CI; run locally with cargo test -- --ignored)"]
    async fn test_create_and_find_by_id() {
        let repo = test_repo().await;
        let project = new_project(ObjectId::new(), OwnerType::User);

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

        let matching = new_project(owner_id, OwnerType::User);
        let same_owner_different_type = new_project(owner_id, OwnerType::Team);
        let same_type_different_owner = new_project(other_owner_id, OwnerType::User);

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
    async fn test_update_metadata_updates_fields_and_timestamp() {
        let repo = test_repo().await;
        let project = new_project(ObjectId::new(), OwnerType::User);
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
}
