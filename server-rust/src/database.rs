use anyhow::Result;
use mongodb::{Client, Database as MongoDatabase};
use tracing::info;

#[derive(Clone)]
pub struct Database {
    pub db: MongoDatabase,
}

impl Database {
    pub async fn new(uri: &str, db_name: &str) -> Result<Self> {
        let client_options = mongodb::options::ClientOptions::parse(uri).await?;

        let client = Client::with_options(client_options)?;

        // Ping the server to ensure connection
        client
            .database("admin")
            .run_command(bson::doc! {"ping": 1})
            .await?;

        info!("Successfully pinged MongoDB deployment");

        let db = client.database(db_name);

        Ok(Database { db })
    }
}
