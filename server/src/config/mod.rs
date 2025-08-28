use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub allow_origins: Vec<String>,
    pub mongo_uri: String,
    pub db_name: String,
    pub address: String,
    pub jwt_secret: String,
}

impl Config {
    pub fn load(env: String, config_path: String) -> Result<Self> {
        let config_file = format!("{}/{}.yaml", config_path, env);

        let settings = config::Config::builder()
            .add_source(config::File::with_name(&config_file))
            .build()?;

        let config: Config = settings.try_deserialize()?;
        config.validate()?;

        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        if self.allow_origins.is_empty() {
            return Err(anyhow::anyhow!("allow_origins cannot be empty"));
        }
        if self.mongo_uri.is_empty() {
            return Err(anyhow::anyhow!("mongo_uri cannot be empty"));
        }
        if self.db_name.is_empty() {
            return Err(anyhow::anyhow!("db_name cannot be empty"));
        }
        if self.address.is_empty() {
            return Err(anyhow::anyhow!("address cannot be empty"));
        }
        if self.jwt_secret.is_empty() {
            return Err(anyhow::anyhow!("jwt_secret cannot be empty"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_config_load_test() -> Result<()> {
        let config = Config::load("test".to_string(), "config".to_string())?;

        assert_eq!(config.db_name, "caduceus_test");
        assert!(!config.jwt_secret.is_empty());

        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn test_config_load_nonsexists() -> Result<()> {
        let result = Config::load("nonsexists".to_string(), "config".to_string());
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_config_validation_valid() {
        let valid_config = Config {
            allow_origins: vec!["http://localhost:3000".to_string()],
            mongo_uri: "mongodb://localhost:27017".to_string(),
            db_name: "test_db".to_string(),
            address: "127.0.0.1:8080".to_string(),
            jwt_secret: "test_secret".to_string(),
        };

        assert!(valid_config.validate().is_ok());
    }
}
