use chrono::Utc;
use mongodb::bson::oid::ObjectId;
use mongodb::{Collection, Database};

use crate::models::project::OwnerType;
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
        owner_type: OwnerType,
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
}
