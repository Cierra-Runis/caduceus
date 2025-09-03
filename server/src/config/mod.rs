use serde::Deserialize;

#[derive(Debug)]
pub enum Error {
    MissingField(String),
    InvalidField(String),
    Io(std::io::Error),
    Parse(config::ConfigError),
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub allow_origins: Vec<String>,
    pub mongo_uri: String,
    pub db_name: String,
    pub address: String,
    pub jwt_secret: String,
}

impl Config {
    pub fn load(file: &str) -> Result<Self, Error> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name(file))
            .add_source(config::Environment::with_prefix("APP"))
            .build();

        let result = match settings {
            Err(e) => return Err(Error::Parse(e)),
            Ok(s) => s.try_deserialize(),
        };

        let config: Config = match result {
            Err(e) => return Err(Error::Parse(e)),
            Ok(c) => c,
        };

        config.validate()?;

        Ok(config)
    }

    fn validate(&self) -> Result<(), Error> {
        if self.allow_origins.is_empty() {
            return Err(Error::MissingField("allow_origins".to_string()));
        }
        if self.mongo_uri.is_empty() {
            return Err(Error::MissingField("mongo_uri".to_string()));
        }
        if self.db_name.is_empty() {
            return Err(Error::MissingField("db_name".to_string()));
        }
        if self.address.is_empty() {
            return Err(Error::MissingField("address".to_string()));
        }
        if self.jwt_secret.is_empty() {
            return Err(Error::MissingField("jwt_secret".to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_config_load_test() {
        let result = Config::load("config/test.yaml");
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.db_name, "caduceus_test");
        assert!(!config.jwt_secret.is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn test_config_load_nonsexists() {
        let result = Config::load("config/nonsexists.yaml");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_config_load_invalid() {
        let invalid_config_path = "config/invalid.yaml";
        let invalid_content = r#"allow_origins: []"#;
        std::fs::write(invalid_config_path, invalid_content).unwrap();
        let result = Config::load(invalid_config_path);
        assert!(result.is_err());
        std::fs::remove_file(invalid_config_path).unwrap();
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

    #[test]
    fn test_config_validation_empty_origins() {
        let invalid_config = Config {
            allow_origins: vec![],
            mongo_uri: "mongodb://localhost:27017".to_string(),
            db_name: "test_db".to_string(),
            address: "127.0.0.1:8080".to_string(),
            jwt_secret: "test_secret".to_string(),
        };

        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_config_validation_empty_mongo_uri() {
        let invalid_config = Config {
            allow_origins: vec!["http://localhost:3000".to_string()],
            mongo_uri: "".to_string(),
            db_name: "test_db".to_string(),
            address: "127.0.0.1:8080".to_string(),
            jwt_secret: "test_secret".to_string(),
        };

        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_config_validation_empty_db_name() {
        let invalid_config = Config {
            allow_origins: vec!["http://localhost:3000".to_string()],
            mongo_uri: "mongodb://localhost:27017".to_string(),
            db_name: "".to_string(),
            address: "127.0.0.1:8080".to_string(),
            jwt_secret: "test_secret".to_string(),
        };

        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_config_validation_empty_address() {
        let invalid_config = Config {
            allow_origins: vec!["http://localhost:3000".to_string()],
            mongo_uri: "mongodb://localhost:27017".to_string(),
            db_name: "test_db".to_string(),
            address: "".to_string(),
            jwt_secret: "test_secret".to_string(),
        };

        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_config_validation_empty_jwt_secret() {
        let invalid_config = Config {
            allow_origins: vec!["http://localhost:3000".to_string()],
            mongo_uri: "mongodb://localhost:27017".to_string(),
            db_name: "test_db".to_string(),
            address: "127.0.0.1:8080".to_string(),
            jwt_secret: "".to_string(),
        };

        assert!(invalid_config.validate().is_err());
    }
}
