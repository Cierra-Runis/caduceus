use actix_cors::Cors;
use serde::Deserialize;
mod cors;
use cors::CorsConfig;

#[derive(Debug)]
pub enum Error {
    MissingField(String),
    InvalidField(String),
    Io(std::io::Error),
    Parse(config::ConfigError),
}

/// WebSocket collaboration tuning knobs.
#[derive(Debug, Clone, Deserialize)]
pub struct WsConfig {
    /// Seconds between heartbeat pings sent to clients.
    #[serde(default = "WsConfig::default_heartbeat_interval_secs")]
    pub heartbeat_interval_secs: u64,
    /// Seconds of silence before a client is considered timed out.
    #[serde(default = "WsConfig::default_client_timeout_secs")]
    pub client_timeout_secs: u64,
    /// Seconds between CRDT-to-MongoDB persistence flushes.
    #[serde(default = "WsConfig::default_persist_interval_secs")]
    pub persist_interval_secs: u64,
}

impl WsConfig {
    fn default_heartbeat_interval_secs() -> u64 {
        5
    }
    fn default_client_timeout_secs() -> u64 {
        10
    }
    fn default_persist_interval_secs() -> u64 {
        3
    }
}

impl Default for WsConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval_secs: Self::default_heartbeat_interval_secs(),
            client_timeout_secs: Self::default_client_timeout_secs(),
            persist_interval_secs: Self::default_persist_interval_secs(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    cors: Option<CorsConfig>,
    pub mongo_uri: String,
    pub db_name: String,
    pub address: Vec<String>,
    pub jwt_secret: String,
    #[serde(default)]
    pub ws: WsConfig,
}

impl Config {
    pub fn cors(&self) -> Cors {
        match &self.cors {
            Some(cfg) => cfg.clone().into(),
            None => Cors::permissive(),
        }
    }

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

        Ok(config)
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
}
