use anyhow::Result;
use mongodb::{options::ClientOptions, Client, Database as MongoDatabase};

#[derive(Clone)]
pub struct Database {
    pub db: MongoDatabase,
}

impl Database {
    pub async fn new(uri: &str, db_name: &str) -> Result<Self> {
        let client_options = ClientOptions::parse(uri).await?;

        let client = Client::with_options(client_options)?;

        let db = client.database(db_name);

        db.run_command(mongodb::bson::doc! {"ping": 1}).await?;

        Ok(Database { db })
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::config::Config;

    #[tokio::test]
    async fn test_database_connection() {
        let config = Config::load("test".to_string(), "config".to_string()).unwrap();
        let database = Database::new(&config.mongo_uri, &config.db_name).await;
        assert!(database.is_ok());
    }

    #[tokio::test]
    async fn test_invalid_database_connection() {
        let database = Database::new("invalid_uri", "test_db").await;
        assert!(database.is_err());
    }
}
