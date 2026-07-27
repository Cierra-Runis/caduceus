//! Object storage, laid out **per project**.
//!
//! Everything a project owns lives under one key prefix:
//!
//! ```text
//! projects/{project_id}/ydoc              # mutable Y.Doc snapshot
//! projects/{project_id}/blobs/{sha256}    # immutable, content-addressed bytes
//! ```
//!
//! The project is the storage boundary. Blobs stay content-addressed *within* a
//! project — so identical bytes in one project share an object, and the
//! write-blob-before-reference invariant still holds (upload the blob, then
//! record its hash on a file node) — but there is no cross-project sharing. In
//! exchange, deleting a project is a single prefix delete (no reachability
//! sweep), and browsing the bucket makes ownership obvious.
//!
//! Two layers:
//! - [`ObjectStore`] is a dumb key/value backend ([`MinioObjectStore`] for
//!   production, [`InMemoryObjectStore`] for tests). It knows nothing about
//!   projects, blobs, or snapshots.
//! - [`ProjectStore`] sits on top and owns the key layout above: content
//!   addressing, the snapshot object, and project deletion.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use derive_more::Display;
use sha2::{Digest, Sha256};

/// A stored blob: the lowercase-hex SHA-256 of its bytes plus their length.
/// This is the durable reference a file node keeps; the bytes live at
/// `projects/{project_id}/blobs/{sha256}`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Blob {
    pub sha256: String,
    pub size: u64,
}

/// Failure modes shared by every backend.
#[derive(Debug, Display)]
pub enum StorageError {
    /// A malformed content hash (not 64 lowercase hex characters). Rejected
    /// before it can reach a backend, since the hash is interpolated into an
    /// object key.
    #[display("invalid sha256: {_0}")]
    InvalidHash(String),
    /// The storage backend failed (network, auth, unexpected status, …).
    #[display("storage backend error: {_0}")]
    Backend(String),
}

impl std::error::Error for StorageError {}

/// Compute the lowercase-hex SHA-256 of `bytes`.
pub fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// True iff `s` is exactly 64 lowercase hex characters — the shape of a hash we
/// produce. Guards backends against path traversal / injection, because the
/// value is placed directly into an object key.
pub fn is_valid_sha256(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// A key/value byte store. Keys are opaque strings; the caller ([`ProjectStore`])
/// owns the key layout. Deliberately *not* content-addressed and *not*
/// project-aware — that lives one layer up.
#[async_trait]
pub trait ObjectStore: Send + Sync {
    /// Store `bytes` at `key`, overwriting any existing object.
    async fn put_object(&self, key: &str, bytes: &[u8]) -> Result<(), StorageError>;

    /// Fetch an object's bytes, or `None` if it doesn't exist.
    async fn get_object(&self, key: &str) -> Result<Option<Vec<u8>>, StorageError>;

    /// Every object key under `prefix`.
    async fn list_prefix(&self, prefix: &str) -> Result<Vec<String>, StorageError>;

    /// Delete a single object by key. Idempotent: an absent key succeeds.
    async fn delete_object(&self, key: &str) -> Result<(), StorageError>;

    /// Delete every object whose key starts with `prefix`. Idempotent: a prefix
    /// that matches nothing succeeds. This is what makes project deletion cheap.
    async fn delete_prefix(&self, prefix: &str) -> Result<(), StorageError>;
}

/// MinIO / S3-compatible backend (path-style addressing).
pub struct MinioObjectStore {
    bucket: Box<s3::Bucket>,
}

impl MinioObjectStore {
    /// Connect to a bucket. `endpoint` is the full base URL (e.g.
    /// `http://localhost:9000`); `region` is arbitrary for MinIO but part of the
    /// S3 signature (`us-east-1` is a safe default). The bucket must already
    /// exist (provisioned by `docker-compose.yml`'s `createbuckets` step).
    pub fn new(
        endpoint: &str,
        region: &str,
        bucket: &str,
        access_key: &str,
        secret_key: &str,
    ) -> Result<Self, StorageError> {
        let region = s3::Region::Custom {
            region: region.to_string(),
            endpoint: endpoint.to_string(),
        };
        let credentials =
            s3::creds::Credentials::new(Some(access_key), Some(secret_key), None, None, None)
                .map_err(|e| StorageError::Backend(e.to_string()))?;
        // Path-style (`/{bucket}/{key}`) is required for MinIO and any endpoint
        // that isn't virtual-host-style S3.
        let bucket = s3::Bucket::new(bucket, region, credentials)
            .map_err(|e| StorageError::Backend(e.to_string()))?
            .with_path_style();
        Ok(Self { bucket })
    }
}

#[async_trait]
impl ObjectStore for MinioObjectStore {
    async fn put_object(&self, key: &str, bytes: &[u8]) -> Result<(), StorageError> {
        let resp = self
            .bucket
            .put_object(key, bytes)
            .await
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        let code = resp.status_code();
        if !(200..300).contains(&code) {
            return Err(StorageError::Backend(format!(
                "put_object returned status {code}"
            )));
        }
        Ok(())
    }

    async fn get_object(&self, key: &str) -> Result<Option<Vec<u8>>, StorageError> {
        let resp = self
            .bucket
            .get_object(key)
            .await
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        match resp.status_code() {
            200 => Ok(Some(resp.to_vec())),
            404 => Ok(None),
            code => Err(StorageError::Backend(format!(
                "get_object returned status {code}"
            ))),
        }
    }

    async fn list_prefix(&self, prefix: &str) -> Result<Vec<String>, StorageError> {
        // No delimiter = fully recursive. At project scale the count is small.
        let pages = self
            .bucket
            .list(prefix.to_string(), None)
            .await
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        Ok(pages
            .into_iter()
            .flat_map(|page| page.contents.into_iter().map(|object| object.key))
            .collect())
    }

    async fn delete_object(&self, key: &str) -> Result<(), StorageError> {
        let resp = self
            .bucket
            .delete_object(key)
            .await
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        match resp.status_code() {
            // 404 is fine: deleting an already-absent object is a no-op.
            200 | 204 | 404 => Ok(()),
            code => Err(StorageError::Backend(format!(
                "delete returned status {code}"
            ))),
        }
    }

    async fn delete_prefix(&self, prefix: &str) -> Result<(), StorageError> {
        for key in self.list_prefix(prefix).await? {
            self.delete_object(&key).await?;
        }
        Ok(())
    }
}

/// In-memory backend for tests. The lock is never held across an `.await`, so a
/// plain `Mutex` is fine.
#[derive(Default)]
pub struct InMemoryObjectStore {
    objects: Mutex<HashMap<String, Vec<u8>>>,
}

impl InMemoryObjectStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of objects held — handy for asserting dedup / isolation in tests.
    pub fn len(&self) -> usize {
        self.objects.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[async_trait]
impl ObjectStore for InMemoryObjectStore {
    async fn put_object(&self, key: &str, bytes: &[u8]) -> Result<(), StorageError> {
        self.objects
            .lock()
            .unwrap()
            .insert(key.to_string(), bytes.to_vec());
        Ok(())
    }

    async fn get_object(&self, key: &str) -> Result<Option<Vec<u8>>, StorageError> {
        Ok(self.objects.lock().unwrap().get(key).cloned())
    }

    async fn list_prefix(&self, prefix: &str) -> Result<Vec<String>, StorageError> {
        Ok(self
            .objects
            .lock()
            .unwrap()
            .keys()
            .filter(|key| key.starts_with(prefix))
            .cloned()
            .collect())
    }

    async fn delete_object(&self, key: &str) -> Result<(), StorageError> {
        self.objects.lock().unwrap().remove(key);
        Ok(())
    }

    async fn delete_prefix(&self, prefix: &str) -> Result<(), StorageError> {
        self.objects
            .lock()
            .unwrap()
            .retain(|key, _| !key.starts_with(prefix));
        Ok(())
    }
}

/// Object key for a project's content-addressed blob.
fn blob_key(project_id: &str, sha256: &str) -> String {
    format!("projects/{project_id}/blobs/{sha256}")
}

/// Object key for a project's Y.Doc snapshot.
fn snapshot_key(project_id: &str) -> String {
    format!("projects/{project_id}/ydoc")
}

/// Everything owned by a project lives under this prefix.
fn project_prefix(project_id: &str) -> String {
    format!("projects/{project_id}/")
}

/// The project-scoped view over an [`ObjectStore`]: it owns the key layout, so
/// nothing else needs to know where a project's bytes live. Cheap to clone (it
/// is just an `Arc`).
#[derive(Clone)]
pub struct ProjectStore {
    inner: Arc<dyn ObjectStore>,
}

impl ProjectStore {
    pub fn new(inner: Arc<dyn ObjectStore>) -> Self {
        Self { inner }
    }

    /// Store `bytes` as a content-addressed blob within `project_id` and return
    /// the resulting [`Blob`]. Content addressing makes this idempotent — the
    /// same bytes always land on the same key — and it is the
    /// *write-before-reference* primitive: persist the returned hash on a node
    /// only after this resolves.
    pub async fn put_blob(&self, project_id: &str, bytes: &[u8]) -> Result<Blob, StorageError> {
        let sha256 = sha256_hex(bytes);
        self.inner
            .put_object(&blob_key(project_id, &sha256), bytes)
            .await?;
        Ok(Blob {
            sha256,
            size: bytes.len() as u64,
        })
    }

    /// Fetch a blob's bytes by content hash within `project_id`, or `None` if it
    /// isn't there.
    pub async fn get_blob(
        &self,
        project_id: &str,
        sha256: &str,
    ) -> Result<Option<Vec<u8>>, StorageError> {
        if !is_valid_sha256(sha256) {
            return Err(StorageError::InvalidHash(sha256.to_string()));
        }
        self.inner.get_object(&blob_key(project_id, sha256)).await
    }

    /// Replace a project's Y.Doc snapshot.
    pub async fn put_snapshot(&self, project_id: &str, bytes: &[u8]) -> Result<(), StorageError> {
        self.inner.put_object(&snapshot_key(project_id), bytes).await
    }

    /// Fetch a project's Y.Doc snapshot, or `None` if it has none yet.
    pub async fn get_snapshot(&self, project_id: &str) -> Result<Option<Vec<u8>>, StorageError> {
        self.inner.get_object(&snapshot_key(project_id)).await
    }

    /// Every blob sha currently stored for `project_id` — the input to a
    /// project-scoped orphan sweep.
    pub async fn list_blobs(&self, project_id: &str) -> Result<Vec<String>, StorageError> {
        let prefix = format!("{}blobs/", project_prefix(project_id));
        let keys = self.inner.list_prefix(&prefix).await?;
        Ok(keys
            .into_iter()
            .filter_map(|key| key.rsplit('/').next().map(str::to_string))
            .collect())
    }

    /// Delete one blob by sha within `project_id` — used by the orphan sweep,
    /// never by a file deletion (other nodes in the project may share the bytes).
    pub async fn delete_blob(&self, project_id: &str, sha256: &str) -> Result<(), StorageError> {
        if !is_valid_sha256(sha256) {
            return Err(StorageError::InvalidHash(sha256.to_string()));
        }
        self.inner.delete_object(&blob_key(project_id, sha256)).await
    }

    /// Delete everything a project owns — snapshot and every blob — in one
    /// prefix sweep. This is the whole GC story for project deletion.
    pub async fn delete_project(&self, project_id: &str) -> Result<(), StorageError> {
        self.inner.delete_prefix(&project_prefix(project_id)).await
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    // SHA-256("hello") — a fixed vector to pin the hashing itself.
    const HELLO_SHA: &str = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";

    #[test]
    fn test_sha256_hex_matches_known_vector() {
        assert_eq!(sha256_hex(b"hello"), HELLO_SHA);
        // Empty input has a well-known digest too.
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_is_valid_sha256() {
        assert!(is_valid_sha256(HELLO_SHA));
        assert!(!is_valid_sha256(&HELLO_SHA.to_uppercase())); // uppercase rejected
        assert!(!is_valid_sha256("abc")); // too short
        assert!(!is_valid_sha256(&"a".repeat(63))); // off-by-one
        assert!(!is_valid_sha256(&"a".repeat(65))); // too long
        assert!(!is_valid_sha256(&format!("{}g", &HELLO_SHA[..63]))); // non-hex char
        assert!(!is_valid_sha256("../secret")); // path traversal shape
    }

    fn store() -> ProjectStore {
        ProjectStore::new(Arc::new(InMemoryObjectStore::new()))
    }

    #[tokio::test]
    async fn test_put_blob_returns_content_hash_and_size() {
        let blob = store().put_blob("p1", b"hello").await.unwrap();
        assert_eq!(blob.sha256, HELLO_SHA);
        assert_eq!(blob.size, 5);
    }

    #[tokio::test]
    async fn test_blob_roundtrips_and_reports_missing() {
        let store = store();
        let blob = store.put_blob("p1", b"payload").await.unwrap();
        assert_eq!(
            store.get_blob("p1", &blob.sha256).await.unwrap().as_deref(),
            Some(&b"payload"[..])
        );
        // A hash that was never stored is absent, not an error.
        let absent = sha256_hex(b"never stored");
        assert_eq!(store.get_blob("p1", &absent).await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_blobs_dedup_within_a_project_but_not_across_projects() {
        let backend = Arc::new(InMemoryObjectStore::new());
        let store = ProjectStore::new(backend.clone());

        // Same content twice in one project is one object (content-addressed).
        let a = store.put_blob("p1", b"same bytes").await.unwrap();
        let b = store.put_blob("p1", b"same bytes").await.unwrap();
        assert_eq!(a, b);
        assert_eq!(backend.len(), 1);

        // The identical content in a *different* project does not share storage.
        let c = store.put_blob("p2", b"same bytes").await.unwrap();
        assert_eq!(a.sha256, c.sha256); // same hash…
        assert_eq!(backend.len(), 2); // …but a distinct object per project
    }

    #[tokio::test]
    async fn test_get_blob_rejects_a_malformed_hash() {
        let store = store();
        for bad in ["../secret", "SHORT", &"Z".repeat(64)] {
            assert!(matches!(
                store.get_blob("p1", bad).await,
                Err(StorageError::InvalidHash(_))
            ));
        }
    }

    #[tokio::test]
    async fn test_snapshot_put_get_overwrite_and_missing() {
        let store = store();
        assert_eq!(store.get_snapshot("p1").await.unwrap(), None);

        store.put_snapshot("p1", b"first").await.unwrap();
        assert_eq!(store.get_snapshot("p1").await.unwrap().as_deref(), Some(&b"first"[..]));

        // The snapshot is mutable: a second put overwrites.
        store.put_snapshot("p1", b"second").await.unwrap();
        assert_eq!(store.get_snapshot("p1").await.unwrap().as_deref(), Some(&b"second"[..]));
    }

    #[tokio::test]
    async fn test_delete_project_removes_its_snapshot_and_blobs_only() {
        let backend = Arc::new(InMemoryObjectStore::new());
        let store = ProjectStore::new(backend.clone());

        let blob = store.put_blob("p1", b"content").await.unwrap();
        store.put_snapshot("p1", b"doc").await.unwrap();
        // A second project whose data must survive p1's deletion.
        store.put_blob("p2", b"content").await.unwrap();
        store.put_snapshot("p2", b"doc").await.unwrap();
        assert_eq!(backend.len(), 4);

        store.delete_project("p1").await.unwrap();

        // p1 is gone…
        assert_eq!(store.get_blob("p1", &blob.sha256).await.unwrap(), None);
        assert_eq!(store.get_snapshot("p1").await.unwrap(), None);
        // …and p2 is untouched.
        assert_eq!(store.get_snapshot("p2").await.unwrap().as_deref(), Some(&b"doc"[..]));
        assert_eq!(backend.len(), 2);
    }

    #[tokio::test]
    async fn test_list_and_delete_blobs_are_project_scoped() {
        let store = store();
        let a = store.put_blob("p1", b"one").await.unwrap();
        let b = store.put_blob("p1", b"two").await.unwrap();
        store.put_blob("p2", b"one").await.unwrap(); // same content, other project

        let mut listed = store.list_blobs("p1").await.unwrap();
        listed.sort();
        let mut want = vec![a.sha256.clone(), b.sha256.clone()];
        want.sort();
        assert_eq!(listed, want);

        store.delete_blob("p1", &a.sha256).await.unwrap();
        assert_eq!(store.get_blob("p1", &a.sha256).await.unwrap(), None);
        assert_eq!(store.list_blobs("p1").await.unwrap(), vec![b.sha256]);
        // p2's identical-content blob is a distinct object, untouched.
        assert!(store.get_blob("p2", &a.sha256).await.unwrap().is_some());
    }

    /// Round-trip against a real MinIO. Ignored by default (needs a running
    /// server + bucket); run with a local stack via:
    ///   `docker compose up -d minio createbuckets`
    ///   `cargo test -p server storage:: -- --ignored`
    #[tokio::test]
    #[ignore = "requires a running MinIO (see docker-compose.yml)"]
    async fn test_minio_roundtrip() {
        let store = ProjectStore::new(Arc::new(
            MinioObjectStore::new(
                "http://localhost:9000",
                "us-east-1",
                "caduceus",
                "minioadmin",
                "minioadmin",
            )
            .unwrap(),
        ));

        let blob = store.put_blob("itest", b"integration bytes").await.unwrap();
        assert_eq!(
            store.get_blob("itest", &blob.sha256).await.unwrap().as_deref(),
            Some(&b"integration bytes"[..])
        );
        store.put_snapshot("itest", b"snap").await.unwrap();

        store.delete_project("itest").await.unwrap();
        assert_eq!(store.get_blob("itest", &blob.sha256).await.unwrap(), None);
        assert_eq!(store.get_snapshot("itest").await.unwrap(), None);
    }
}
