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
    /// Builds the CORS middleware from the loaded configuration.
    ///
    /// When no `cors` section is configured, cross-origin requests are
    /// rejected (no CORS allow headers are sent). Deployments that need
    /// cross-origin access must explicitly configure `cors.allow_origins`.
    pub fn cors(&self) -> Cors {
        match &self.cors {
            Some(cfg) => cfg.clone().into(),
            None => Cors::default(),
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
    use actix_web::{
        http::{header, Method},
        test, web, App, HttpResponse,
    };
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

    #[actix_web::test]
    async fn test_cors_missing_config_rejects_cross_origin() {
        let config = Config {
            cors: None,
            mongo_uri: "mongodb://localhost:27017".to_string(),
            db_name: "caduceus_test".to_string(),
            address: vec!["localhost:8080".to_string()],
            jwt_secret: "secret".to_string(),
            ws: WsConfig::default(),
        };

        let app = test::init_service(
            App::new()
                .wrap(config.cors())
                .route("/", web::get().to(HttpResponse::Ok)),
        )
        .await;

        // A preflight request from an external origin must be rejected.
        let req = test::TestRequest::default()
            .method(Method::OPTIONS)
            .uri("/")
            .insert_header((header::ORIGIN, "https://attacker.example"))
            .insert_header((header::ACCESS_CONTROL_REQUEST_METHOD, "GET"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());
        assert!(resp
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .is_none());

        // A plain request from an external origin must not receive any CORS
        // allow headers, so browsers deny the cross-origin read.
        let req = test::TestRequest::get()
            .uri("/")
            .insert_header((header::ORIGIN, "https://attacker.example"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .is_none());
    }

    #[actix_web::test]
    #[serial]
    async fn test_cors_configured_allows_listed_origin() {
        let config = Config::load("config/test.yaml").unwrap();

        let app = test::init_service(
            App::new()
                .wrap(config.cors())
                .route("/", web::get().to(HttpResponse::Ok)),
        )
        .await;

        // Preflight from the configured origin succeeds.
        let req = test::TestRequest::default()
            .method(Method::OPTIONS)
            .uri("/")
            .insert_header((header::ORIGIN, "http://localhost:3000"))
            .insert_header((header::ACCESS_CONTROL_REQUEST_METHOD, "GET"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        assert_eq!(
            resp.headers()
                .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                .map(|v| v.to_str().unwrap()),
            Some("http://localhost:3000")
        );

        // A plain request from the configured origin gets the allow header.
        let req = test::TestRequest::get()
            .uri("/")
            .insert_header((header::ORIGIN, "http://localhost:3000"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        assert_eq!(
            resp.headers()
                .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                .map(|v| v.to_str().unwrap()),
            Some("http://localhost:3000")
        );
    }
}
