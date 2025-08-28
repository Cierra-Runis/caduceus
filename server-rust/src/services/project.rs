use bson::oid::ObjectId;
use chrono::Utc;
use mongodb::{Collection, Database};

use crate::{
    error::{AppError, Result},
    models::project::Project,
};

#[derive(Clone)]
pub struct ProjectService {
    collection: Collection<Project>,
}

impl ProjectService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("projects"),
        }
    }

    pub async fn create_project(
        &self,
        name: String,
        owner_id: ObjectId,
        owner_type: String,
    ) -> Result<Project> {
        let now = Utc::now();
        let project = Project {
            id: None,
            name,
            owner_id,
            owner_type,
            created_at: now,
            updated_at: now,
        };

        let result = self.collection.insert_one(&project).await?;
        let inserted_id = result
            .inserted_id
            .as_object_id()
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("Failed to get inserted ID")))?;

        let mut created_project = project;
        created_project.id = Some(inserted_id);
        Ok(created_project)
    }

    // pub async fn get_project_by_id(&self, project_id: &str) -> Result<Project> {
    //     let object_id = ObjectId::parse_str(project_id)?;
    //     let project = self
    //         .collection
    //         .find_one(doc! { "_id": object_id })
    //         .await?
    //         .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

    //     Ok(project)
    // }

    // pub async fn get_projects_by_owner(
    //     &self,
    //     owner_id: ObjectId,
    //     owner_type: &str,
    // ) -> Result<Vec<Project>> {
    //     let mut cursor = self
    //         .collection
    //         .find(doc! { "owner_id": owner_id, "owner_type": owner_type })
    //         .await?;

    //     let mut projects = Vec::new();
    //     while cursor.advance().await? {
    //         projects.push(cursor.deserialize_current()?);
    //     }

    //     Ok(projects)
    // }
}
