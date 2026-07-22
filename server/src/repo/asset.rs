use bson::oid::ObjectId;
use derive_more::Display;

/// Error surfaced by an [`AssetStore`]. Backend-agnostic on purpose: GridFS
/// (driver) errors and S3/MinIO (HTTP) errors both collapse into a single
/// message here, so the asset service and its callers depend on one error type
/// no matter which backend is configured.
#[derive(Debug, Display)]
#[display("asset storage error: {message}")]
pub struct AssetStoreError {
    pub message: String,
}

impl AssetStoreError {
    fn new(err: impl std::fmt::Display) -> Self {
        Self {
            message: err.to_string(),
        }
    }
}

pub type Result<T> = std::result::Result<T, AssetStoreError>;

/// A binary asset retrieved from storage: its bytes plus the metadata needed to
/// serve it back over HTTP with the right headers.
#[derive(Debug, Clone)]
pub struct StoredAsset {
    pub bytes: Vec<u8>,
    /// MIME type recorded at upload time, if the client provided one.
    pub content_type: Option<String>,
    /// Original filename recorded at upload time (best effort — some backends
    /// only retain the storage key).
    pub filename: String,
}

/// Storage backend for [`FileContent::Binary`](crate::models::project::FileContent)
/// payloads. Text lives inline in the project document; binaries (images,
/// fonts, …) live here and are referenced from a file by their storage key.
///
/// The key is an opaque `String`, not a backend-specific type: a GridFS
/// `ObjectId`, an S3/MinIO object path, and a Git blob SHA are all just strings.
/// Keeping the seam abstract lets the concrete backend be a deployment choice
/// (config-selected, see [`AssetStoreKind`]) and lets a future Git-backed
/// implementation drop in without reshaping stored documents.
#[async_trait::async_trait]
pub trait AssetStore: Send + Sync {
    /// Store `bytes` and return the opaque key that addresses them. `filename`
    /// and `content_type` are retained so the asset can be served back with the
    /// right headers.
    async fn upload(
        &self,
        filename: &str,
        content_type: Option<&str>,
        bytes: Vec<u8>,
    ) -> Result<String>;

    /// Fetch a previously stored asset by its key, or `None` if no asset has
    /// that key.
    async fn download(&self, key: &str) -> Result<Option<StoredAsset>>;
}

/// The configured asset backend, resolved once at startup from
/// [`StorageConfig`](crate::config::StorageConfig). A single concrete type
/// (rather than a `dyn AssetStore`) keeps [`AppState`](crate::AppState)
/// monomorphic and dispatches to the right backend per call.
pub enum AssetStoreKind {
    GridFs(GridFsAssetStore),
    S3(S3AssetStore),
}

#[async_trait::async_trait]
impl AssetStore for AssetStoreKind {
    async fn upload(
        &self,
        filename: &str,
        content_type: Option<&str>,
        bytes: Vec<u8>,
    ) -> Result<String> {
        match self {
            AssetStoreKind::GridFs(store) => store.upload(filename, content_type, bytes).await,
            AssetStoreKind::S3(store) => store.upload(filename, content_type, bytes).await,
        }
    }

    async fn download(&self, key: &str) -> Result<Option<StoredAsset>> {
        match self {
            AssetStoreKind::GridFs(store) => store.download(key).await,
            AssetStoreKind::S3(store) => store.download(key).await,
        }
    }
}

/// GridFS-backed [`AssetStore`] — the zero-dependency default, so a self-hosted
/// deployment needs nothing beyond the MongoDB it already runs. GridFS chunks
/// large binaries across a dedicated collection, so an uploaded image is not
/// bounded by the 16 MB BSON document limit and never bloats the project
/// document itself.
///
/// The storage key is the GridFS file's `ObjectId` rendered as a hex string;
/// this backend parses it back on the way in.
#[derive(Clone)]
pub struct GridFsAssetStore {
    pub bucket: mongodb::gridfs::GridFsBucket,
}

/// Key under which the upload MIME type is stashed in the GridFS file's
/// `metadata` document, to be read back when serving the asset.
const CONTENT_TYPE_KEY: &str = "content_type";

#[async_trait::async_trait]
impl AssetStore for GridFsAssetStore {
    async fn upload(
        &self,
        filename: &str,
        content_type: Option<&str>,
        bytes: Vec<u8>,
    ) -> Result<String> {
        use futures_util::io::AsyncWriteExt;

        // Assign the id up front so the caller gets the key without a round-trip
        // to read `stream.id()` back.
        let id = ObjectId::new();
        let mut metadata = bson::Document::new();
        if let Some(ct) = content_type {
            metadata.insert(CONTENT_TYPE_KEY, ct);
        }

        let mut stream = self
            .bucket
            .open_upload_stream(filename)
            .id(bson::Bson::ObjectId(id))
            .metadata(metadata)
            .await
            .map_err(AssetStoreError::new)?;
        // The GridFS stream surfaces driver failures as `io::Error`.
        stream
            .write_all(&bytes)
            .await
            .map_err(AssetStoreError::new)?;
        stream.close().await.map_err(AssetStoreError::new)?;

        Ok(id.to_hex())
    }

    async fn download(&self, key: &str) -> Result<Option<StoredAsset>> {
        use futures_util::io::AsyncReadExt;

        // A key that is not a valid ObjectId cannot address a GridFS file — treat
        // it as absent rather than erroring, matching a missing-key lookup.
        let Ok(id) = ObjectId::parse_str(key) else {
            return Ok(None);
        };

        let Some(file) = self
            .bucket
            .find_one(bson::doc! { "_id": bson::Bson::ObjectId(id) })
            .await
            .map_err(AssetStoreError::new)?
        else {
            return Ok(None);
        };

        let mut stream = self
            .bucket
            .open_download_stream(bson::Bson::ObjectId(id))
            .await
            .map_err(AssetStoreError::new)?;
        let mut bytes = Vec::new();
        stream
            .read_to_end(&mut bytes)
            .await
            .map_err(AssetStoreError::new)?;

        let content_type = file
            .metadata
            .as_ref()
            .and_then(|m| m.get_str(CONTENT_TYPE_KEY).ok())
            .map(str::to_string);

        Ok(Some(StoredAsset {
            bytes,
            content_type,
            filename: file.filename.unwrap_or_default(),
        }))
    }
}

/// S3/MinIO-backed [`AssetStore`], selected via config for deployments that
/// prefer object storage: large binaries stay out of the database, and the
/// door is open to presigned direct-to-browser delivery and lifecycle policies.
/// The storage key is the object's key within the bucket.
pub struct S3AssetStore {
    bucket: Box<s3::Bucket>,
}

impl S3AssetStore {
    /// Build a bucket handle from config. MinIO and other S3-compatibles are
    /// reached with a custom endpoint and path-style addressing.
    pub fn new(cfg: &crate::config::S3Config) -> Result<Self> {
        let region = s3::Region::Custom {
            region: cfg.region.clone(),
            endpoint: cfg.endpoint.clone(),
        };
        let credentials = s3::creds::Credentials::new(
            Some(&cfg.access_key),
            Some(&cfg.secret_key),
            None,
            None,
            None,
        )
        .map_err(AssetStoreError::new)?;
        let mut bucket =
            s3::Bucket::new(&cfg.bucket, region, credentials).map_err(AssetStoreError::new)?;
        if cfg.path_style {
            bucket = bucket.with_path_style();
        }
        Ok(Self { bucket })
    }
}

#[async_trait::async_trait]
impl AssetStore for S3AssetStore {
    async fn upload(
        &self,
        filename: &str,
        content_type: Option<&str>,
        bytes: Vec<u8>,
    ) -> Result<String> {
        // Namespace the object under a fresh id so distinct uploads of the same
        // filename never collide, while keeping the original name in the key for
        // human-readable listings in the bucket.
        let key = format!("{}/{}", ObjectId::new().to_hex(), sanitize_key_segment(filename));
        let content_type = content_type.unwrap_or("application/octet-stream");
        let response = self
            .bucket
            .put_object_with_content_type(&key, &bytes, content_type)
            .await
            .map_err(AssetStoreError::new)?;
        let code = response.status_code();
        if !(200..300).contains(&code) {
            return Err(AssetStoreError::new(format!(
                "s3 put returned status {code}"
            )));
        }
        Ok(key)
    }

    async fn download(&self, key: &str) -> Result<Option<StoredAsset>> {
        let response = self
            .bucket
            .get_object(key)
            .await
            .map_err(AssetStoreError::new)?;
        match response.status_code() {
            200..=299 => {
                let content_type = response.headers().get("content-type").cloned();
                let filename = key.rsplit('/').next().unwrap_or(key).to_string();
                Ok(Some(StoredAsset {
                    bytes: response.bytes().to_vec(),
                    content_type,
                    filename,
                }))
            }
            404 => Ok(None),
            code => Err(AssetStoreError::new(format!(
                "s3 get returned status {code}"
            ))),
        }
    }
}

/// Make a filename safe to embed in an object key: drop path separators so a
/// crafted name can't escape its id prefix or nest unexpectedly.
fn sanitize_key_segment(filename: &str) -> String {
    let trimmed = filename.rsplit(['/', '\\']).next().unwrap_or(filename);
    if trimmed.is_empty() {
        "asset".to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// In-memory [`AssetStore`] for service unit tests — no GridFS or S3 needed.
    #[derive(Default)]
    pub struct MockAssetStore {
        pub assets: Mutex<HashMap<String, StoredAsset>>,
    }

    #[async_trait::async_trait]
    impl AssetStore for MockAssetStore {
        async fn upload(
            &self,
            filename: &str,
            content_type: Option<&str>,
            bytes: Vec<u8>,
        ) -> Result<String> {
            let key = ObjectId::new().to_hex();
            self.assets.lock().unwrap().insert(
                key.clone(),
                StoredAsset {
                    bytes,
                    content_type: content_type.map(str::to_string),
                    filename: filename.to_string(),
                },
            );
            Ok(key)
        }

        async fn download(&self, key: &str) -> Result<Option<StoredAsset>> {
            Ok(self.assets.lock().unwrap().get(key).cloned())
        }
    }

    #[tokio::test]
    async fn test_mock_store_roundtrip() {
        let store = MockAssetStore::default();
        let key = store
            .upload("logo.png", Some("image/png"), vec![1, 2, 3])
            .await
            .unwrap();

        let asset = store.download(&key).await.unwrap().unwrap();
        assert_eq!(asset.bytes, vec![1, 2, 3]);
        assert_eq!(asset.content_type.as_deref(), Some("image/png"));
        assert_eq!(asset.filename, "logo.png");
    }

    #[tokio::test]
    async fn test_mock_store_missing_key() {
        let store = MockAssetStore::default();
        assert!(
            store
                .download(&ObjectId::new().to_hex())
                .await
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn test_sanitize_key_segment_strips_paths() {
        assert_eq!(sanitize_key_segment("logo.png"), "logo.png");
        assert_eq!(sanitize_key_segment("../../etc/passwd"), "passwd");
        assert_eq!(sanitize_key_segment("a\\b\\c.png"), "c.png");
        assert_eq!(sanitize_key_segment(""), "asset");
    }

    /// Build an [`S3Config`](crate::config::S3Config) from `S3_TEST_*` env vars,
    /// falling back to a local MinIO with the conventional `minioadmin`
    /// credentials. The same defaults let the ignored tests below run unchanged
    /// in CI (which provisions MinIO) and against a locally started MinIO.
    fn s3_test_config() -> crate::config::S3Config {
        fn env_or(key: &str, default: &str) -> String {
            std::env::var(key).unwrap_or_else(|_| default.to_string())
        }
        crate::config::S3Config {
            endpoint: env_or("S3_TEST_ENDPOINT", "http://localhost:9000"),
            bucket: env_or("S3_TEST_BUCKET", "caduceus-test"),
            region: env_or("S3_TEST_REGION", "us-east-1"),
            access_key: env_or("S3_TEST_ACCESS_KEY", "minioadmin"),
            secret_key: env_or("S3_TEST_SECRET_KEY", "minioadmin"),
            // MinIO requires path-style addressing.
            path_style: true,
        }
    }

    /// Create the target bucket if it is not already there, so the tests are
    /// self-contained regardless of whether CI pre-created it. Creating a bucket
    /// that already exists returns a non-2xx status (or errors) on MinIO, which
    /// is deliberately ignored — the tests only require that the bucket exists
    /// afterwards, and the subsequent upload/download would surface any real
    /// connectivity problem.
    async fn ensure_test_bucket(cfg: &crate::config::S3Config) {
        let region = s3::Region::Custom {
            region: cfg.region.clone(),
            endpoint: cfg.endpoint.clone(),
        };
        let credentials = s3::creds::Credentials::new(
            Some(&cfg.access_key),
            Some(&cfg.secret_key),
            None,
            None,
            None,
        )
        .expect("valid MinIO credentials");
        let _ = s3::Bucket::create_with_path_style(
            &cfg.bucket,
            region,
            credentials,
            s3::BucketConfiguration::default(),
        )
        .await;
    }

    /// Construct an [`S3AssetStore`] pointed at the test MinIO, ensuring its
    /// bucket exists first.
    async fn s3_test_store() -> S3AssetStore {
        let cfg = s3_test_config();
        ensure_test_bucket(&cfg).await;
        S3AssetStore::new(&cfg).expect("build S3AssetStore from test config")
    }

    #[tokio::test]
    #[ignore = "requires a live MinIO / S3 (CI provisions one; set S3_TEST_* env vars to run locally)"]
    async fn test_s3_store_roundtrip() {
        let store = s3_test_store().await;
        let bytes = vec![0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x2A, 0xFF];

        let key = store
            .upload("diagram.png", Some("image/png"), bytes.clone())
            .await
            .expect("upload to MinIO");

        let asset = store
            .download(&key)
            .await
            .expect("download from MinIO")
            .expect("asset present after upload");

        assert_eq!(asset.bytes, bytes, "downloaded bytes match uploaded bytes");
        assert_eq!(asset.content_type.as_deref(), Some("image/png"));
        assert_eq!(asset.filename, "diagram.png");
    }

    #[tokio::test]
    #[ignore = "requires a live MinIO / S3 (CI provisions one; set S3_TEST_* env vars to run locally)"]
    async fn test_s3_store_missing_key() {
        let store = s3_test_store().await;
        // A well-formed but never-uploaded key must read back as absent, not error.
        let missing = format!("{}/never-uploaded.png", ObjectId::new().to_hex());
        assert!(
            store
                .download(&missing)
                .await
                .expect("download of missing key is Ok")
                .is_none()
        );
    }
}
