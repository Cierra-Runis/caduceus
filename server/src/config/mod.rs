use std::path::{Path, PathBuf};

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
    /// Seconds between room persistence ticks (Y.Doc snapshot to MinIO + tree
    /// projection to Mongo). Blob materialization is separate and client-driven.
    #[serde(default = "WsConfig::default_persist_interval_secs")]
    pub persist_interval_secs: u64,
    /// Seconds between orphaned-blob garbage-collection sweeps. Much larger than
    /// the persist interval: GC only reclaims space, so it can run lazily, and a
    /// blob must be seen orphaned across two consecutive sweeps before it is
    /// deleted (a grace window against the upload-then-reference gap).
    #[serde(default = "WsConfig::default_gc_interval_secs")]
    pub gc_interval_secs: u64,
    /// Seconds a room may sit with no connections before it is evicted from
    /// memory (after a final persist). The next joiner rebuilds it verbatim from
    /// the snapshot, so eviction only reclaims RAM — a reconnecting client still
    /// syncs against a byte-identical document.
    #[serde(default = "WsConfig::default_room_idle_secs")]
    pub room_idle_secs: u64,
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
    fn default_gc_interval_secs() -> u64 {
        300
    }
    fn default_room_idle_secs() -> u64 {
        1800
    }
}

impl Default for WsConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval_secs: Self::default_heartbeat_interval_secs(),
            client_timeout_secs: Self::default_client_timeout_secs(),
            persist_interval_secs: Self::default_persist_interval_secs(),
            gc_interval_secs: Self::default_gc_interval_secs(),
            room_idle_secs: Self::default_room_idle_secs(),
        }
    }
}

/// Object-storage (MinIO / S3) connection settings. Optional so a checkout can
/// run without a storage backend configured; wired into an [`ObjectStore`] by
/// whichever component first needs blob storage.
///
/// [`ObjectStore`]: crate::storage::ObjectStore
#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    /// Full base URL of the endpoint, e.g. `http://localhost:9000`.
    pub endpoint: String,
    /// S3 region. Arbitrary for MinIO but part of the request signature.
    #[serde(default = "StorageConfig::default_region")]
    pub region: String,
    /// Bucket that holds `blobs/{sha256}` objects.
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
}

impl StorageConfig {
    fn default_region() -> String {
        "us-east-1".to_string()
    }
}

/// Server-side Typst compilation via tinymist workers (LSP + diagnostics).
/// Optional so a checkout runs without any tinymist binaries: absent disables
/// the feature entirely. tinymist is obtained as a **subprocess** binary — the
/// crate is not usable as a library dependency (it builds only against a
/// patched Typst fork), and crates.io ships no binary — so each supported Typst
/// version maps to a `tinymist` binary built and staged out of band (see
/// `scripts/build-tinymist.sh`).
#[derive(Debug, Clone, Deserialize)]
pub struct LspConfig {
    /// Supported Typst versions and the tinymist binary that compiles each. A
    /// project routes to the worker for its pinned version.
    pub versions: Vec<TinymistVersion>,
    /// Version used when a project pins nothing, or pins an unsupported one.
    pub default_version: String,
    /// Directory under which each room's worker gets a staging root (binary
    /// blobs materialized from MinIO, package cache) for `#image`/`#read`.
    pub workspace_root: PathBuf,
    /// Optional shared, read-only package cache (`@preview/*`) across workers.
    #[serde(default)]
    pub package_cache: Option<PathBuf>,
}

/// One supported Typst version and the tinymist binary that compiles it. Each
/// tinymist release compiles exactly one Typst version (reported at runtime),
/// so version routing is binary selection.
#[derive(Debug, Clone, Deserialize)]
pub struct TinymistVersion {
    /// The Typst version this binary compiles, e.g. `"0.13.1"`.
    pub typst_version: String,
    /// Absolute path to the tinymist binary for this version.
    pub binary: PathBuf,
}

impl LspConfig {
    /// Resolve the tinymist binary for a requested version, falling back to the
    /// configured default. `None` when neither the request nor the default is
    /// among the supported versions (a misconfiguration).
    pub fn binary_for(&self, version: Option<&str>) -> Option<&Path> {
        let requested = version.unwrap_or(&self.default_version);
        self.lookup(requested)
            .or_else(|| self.lookup(&self.default_version))
    }

    /// Whether a Typst version has a configured worker binary.
    pub fn is_supported(&self, version: &str) -> bool {
        self.versions.iter().any(|v| v.typst_version == version)
    }

    fn lookup(&self, version: &str) -> Option<&Path> {
        self.versions
            .iter()
            .find(|v| v.typst_version == version)
            .map(|v| v.binary.as_path())
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
    #[serde(default)]
    pub storage: Option<StorageConfig>,
    #[serde(default)]
    pub lsp: Option<LspConfig>,
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
            storage: None,
            lsp: None,
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

// A separate test module so it does not inherit `mod tests`'s
// `use actix_web::test`, which shadows the built-in `#[test]` attribute.
#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod lsp_tests {
    use super::*;

    fn lsp_config() -> LspConfig {
        LspConfig {
            versions: vec![
                TinymistVersion {
                    typst_version: "0.12.0".to_string(),
                    binary: PathBuf::from("/opt/tinymist/0.12.0/tinymist"),
                },
                TinymistVersion {
                    typst_version: "0.13.1".to_string(),
                    binary: PathBuf::from("/opt/tinymist/0.13.1/tinymist"),
                },
            ],
            default_version: "0.13.1".to_string(),
            workspace_root: PathBuf::from("/var/lib/caduceus/lsp"),
            package_cache: None,
        }
    }

    #[test]
    fn binary_for_resolves_the_requested_version() {
        let cfg = lsp_config();
        assert_eq!(
            cfg.binary_for(Some("0.12.0")),
            Some(Path::new("/opt/tinymist/0.12.0/tinymist"))
        );
    }

    #[test]
    fn binary_for_falls_back_to_default_when_unpinned_or_unsupported() {
        let cfg = lsp_config();
        // No pin → default.
        assert_eq!(
            cfg.binary_for(None),
            Some(Path::new("/opt/tinymist/0.13.1/tinymist"))
        );
        // Pinned to an unsupported version → default, not None.
        assert_eq!(
            cfg.binary_for(Some("9.9.9")),
            Some(Path::new("/opt/tinymist/0.13.1/tinymist"))
        );
    }

    #[test]
    fn binary_for_is_none_when_even_the_default_is_missing() {
        let mut cfg = lsp_config();
        cfg.default_version = "9.9.9".to_string();
        assert_eq!(cfg.binary_for(Some("8.8.8")), None);
    }

    #[test]
    fn is_supported_reflects_configured_versions() {
        let cfg = lsp_config();
        assert!(cfg.is_supported("0.12.0"));
        assert!(!cfg.is_supported("0.99.0"));
    }
}
