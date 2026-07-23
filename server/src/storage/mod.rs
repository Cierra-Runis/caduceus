//! Object storage for binary assets (images, fonts, …).
//!
//! Text files live inline in the Mongo document; binaries are far too big to
//! sit in a project document that gets loaded whole into the editor, so their
//! bytes live here and the [`ProjectFile`](crate::models::project::ProjectFile)
//! only keeps a `storage_key` pointer.
//!
//! Everything is expressed against the [`ObjectStore`] trait so the services /
//! handlers never depend on a concrete backend: production wires a MinIO
//! (S3-compatible) store, tests use [`InMemoryObjectStore`]. This is also the
//! seam a future GitHub-sync layer plugs into — the same keyed blobs can be
//! mirrored to a git repo's blobs without the rest of the app noticing.

use std::collections::HashMap;
use std::sync::Mutex;

use derive_more::Display;
use s3::creds::Credentials;
use s3::{Bucket, Region};
use serde::Deserialize;

#[derive(Debug, Display)]
pub enum StorageError {
    #[display("Object not found")]
    NotFound,
    #[display("Object storage error: {_0}")]
    Backend(String),
}

/// A minimal blob store keyed by opaque string keys. Keys are the full object
/// path (e.g. `projects/{project_id}/{storage_key}`); the store does not impose
/// any layout of its own.
#[async_trait::async_trait]
pub trait ObjectStore: Send + Sync {
    async fn put(&self, key: &str, bytes: &[u8], content_type: &str) -> Result<(), StorageError>;
    async fn get(&self, key: &str) -> Result<Vec<u8>, StorageError>;
    async fn delete(&self, key: &str) -> Result<(), StorageError>;
}

/// Connection settings for the S3-compatible (MinIO) backend. Defaults target a
/// local MinIO from `docker-compose`, so a fresh checkout runs without extra
/// configuration; production overrides these in `config/<env>.yaml` or via
/// `APP_STORAGE__*` environment variables.
#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    #[serde(default = "StorageConfig::default_endpoint")]
    pub endpoint: String,
    #[serde(default = "StorageConfig::default_region")]
    pub region: String,
    #[serde(default = "StorageConfig::default_bucket")]
    pub bucket: String,
    #[serde(default = "StorageConfig::default_access_key")]
    pub access_key: String,
    #[serde(default = "StorageConfig::default_secret_key")]
    pub secret_key: String,
    /// MinIO speaks path-style addressing (`endpoint/bucket/key`), not the
    /// virtual-host style AWS uses, so this defaults to `true`.
    #[serde(default = "StorageConfig::default_path_style")]
    pub path_style: bool,
}

impl StorageConfig {
    fn default_endpoint() -> String {
        "http://localhost:9000".to_string()
    }
    fn default_region() -> String {
        "us-east-1".to_string()
    }
    fn default_bucket() -> String {
        "caduceus".to_string()
    }
    fn default_access_key() -> String {
        "minioadmin".to_string()
    }
    fn default_secret_key() -> String {
        "minioadmin".to_string()
    }
    fn default_path_style() -> bool {
        true
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            endpoint: Self::default_endpoint(),
            region: Self::default_region(),
            bucket: Self::default_bucket(),
            access_key: Self::default_access_key(),
            secret_key: Self::default_secret_key(),
            path_style: Self::default_path_style(),
        }
    }
}

/// S3-compatible object store, used with MinIO in development and any S3 API in
/// production.
pub struct MinioObjectStore {
    bucket: Box<Bucket>,
}

impl MinioObjectStore {
    /// Build the store from config. This only constructs the client; it does
    /// not talk to the server, so an unreachable MinIO surfaces later, on the
    /// first upload, rather than at boot.
    pub fn new(config: &StorageConfig) -> Result<Self, StorageError> {
        let credentials = Credentials::new(
            Some(&config.access_key),
            Some(&config.secret_key),
            None,
            None,
            None,
        )
        .map_err(|e| StorageError::Backend(e.to_string()))?;

        let region = Region::Custom {
            region: config.region.clone(),
            endpoint: config.endpoint.clone(),
        };

        let mut bucket = Bucket::new(&config.bucket, region, credentials)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        if config.path_style {
            bucket.set_path_style();
        }
        Ok(Self { bucket })
    }
}

#[async_trait::async_trait]
impl ObjectStore for MinioObjectStore {
    async fn put(&self, key: &str, bytes: &[u8], content_type: &str) -> Result<(), StorageError> {
        let response = self
            .bucket
            .put_object_with_content_type(key, bytes, content_type)
            .await
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        match response.status_code() {
            200..=299 => Ok(()),
            code => Err(StorageError::Backend(format!("PUT returned status {code}"))),
        }
    }

    async fn get(&self, key: &str) -> Result<Vec<u8>, StorageError> {
        let response = self
            .bucket
            .get_object(key)
            .await
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        match response.status_code() {
            200..=299 => Ok(response.bytes().to_vec()),
            404 => Err(StorageError::NotFound),
            code => Err(StorageError::Backend(format!("GET returned status {code}"))),
        }
    }

    async fn delete(&self, key: &str) -> Result<(), StorageError> {
        let response = self
            .bucket
            .delete_object(key)
            .await
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        match response.status_code() {
            // 204 No Content is the normal delete result; 404 is idempotent.
            200..=299 | 404 => Ok(()),
            code => Err(StorageError::Backend(format!(
                "DELETE returned status {code}"
            ))),
        }
    }
}

/// In-memory object store for unit and integration tests — no network, no
/// MinIO. Mirrors the trait's contract (missing key on `get` → `NotFound`,
/// `delete` is idempotent).
#[derive(Default)]
pub struct InMemoryObjectStore {
    objects: Mutex<HashMap<String, Vec<u8>>>,
}

#[async_trait::async_trait]
impl ObjectStore for InMemoryObjectStore {
    async fn put(&self, key: &str, bytes: &[u8], _content_type: &str) -> Result<(), StorageError> {
        self.objects
            .lock()
            .unwrap()
            .insert(key.to_string(), bytes.to_vec());
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Vec<u8>, StorageError> {
        self.objects
            .lock()
            .unwrap()
            .get(key)
            .cloned()
            .ok_or(StorageError::NotFound)
    }

    async fn delete(&self, key: &str) -> Result<(), StorageError> {
        self.objects.lock().unwrap().remove(key);
        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn in_memory_put_get_delete_roundtrip() {
        let store = InMemoryObjectStore::default();
        assert!(matches!(store.get("k").await, Err(StorageError::NotFound)));

        store.put("k", b"hello", "text/plain").await.unwrap();
        assert_eq!(store.get("k").await.unwrap(), b"hello");

        store.delete("k").await.unwrap();
        assert!(matches!(store.get("k").await, Err(StorageError::NotFound)));
        // Delete is idempotent.
        store.delete("k").await.unwrap();
    }
}
