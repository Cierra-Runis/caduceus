use anyhow::Result;
use mongodb::{Client, Database as MongoDatabase};

#[derive(Clone)]
pub struct Database {
    pub db: MongoDatabase,
}

impl Database {
    pub async fn new(uri: &str, db_name: &str) -> Result<Self> {
        let client_options = mongodb::options::ClientOptions::parse(uri).await?;

        let client = Client::with_options(client_options)?;

        let db = client.database(db_name);

        db.run_command(mongodb::bson::doc! {"ping": 1}).await?;

        Ok(Database { db })
    }
}
