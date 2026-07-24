//! Content-addressed object storage.
//!
//! A blob is identified purely by the SHA-256 of its bytes and lives at the key
//! `blobs/{sha256}`. Content addressing is what the persistence redesign leans
//! on (see `docs/Architecture - Compilation and Project Model.md`):
//!
//! - **dedup** — identical bytes across files or projects share one object;
//! - **write-blob-before-reference safety** — a writer uploads the blob *first*
//!   and only then records the hash on a file node, so a crash in between
//!   leaves an unreferenced (GC-able) blob, never a dangling reference to bytes
//!   that were never written;
//! - **immutability** — an object's bytes never change, so two writers of the
//!   same content converge instead of racing.
//!
//! [`ObjectStore`] is the seam: [`MinioObjectStore`] talks to MinIO/S3 in
//! production, [`InMemoryObjectStore`] backs tests. Nothing above this module
//! knows which backend is in play.

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use derive_more::Display;
use sha2::{Digest, Sha256};

/// A stored blob: the lowercase-hex SHA-256 of its bytes plus their length.
/// This is the durable reference a file node keeps; the bytes themselves live
/// at `blobs/{sha256}`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Blob {
    pub sha256: String,
    pub size: u64,
}

/// Failure modes shared by every backend.
#[derive(Debug, Display)]
pub enum StorageError {
    /// The requested blob does not exist.
    #[display("blob not found: {_0}")]
    NotFound(String),
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

/// Object key for a blob's bytes.
fn blob_key(sha256: &str) -> String {
    format!("blobs/{sha256}")
}

/// A content-addressed byte store. Keys are SHA-256 hashes; values are
/// immutable blobs.
#[async_trait]
pub trait ObjectStore: Send + Sync {
    /// Store `bytes` and return the resulting [`Blob`]. Idempotent: content that
    /// already exists is not re-uploaded (a re-PUT of identical bytes would be a
    /// no-op regardless). This is the *write-before-reference* primitive —
    /// callers persist the returned hash only after this resolves.
    async fn put(&self, bytes: &[u8]) -> Result<Blob, StorageError>;

    /// Fetch a blob's bytes by content hash.
    async fn get(&self, sha256: &str) -> Result<Vec<u8>, StorageError>;

    /// Whether a blob exists, without downloading it — used by GC and to skip
    /// redundant uploads.
    async fn exists(&self, sha256: &str) -> Result<bool, StorageError>;

    /// Remove a blob by content hash. Idempotent: deleting an absent blob
    /// succeeds. Only the GC sweep should ever call this — a file deletion must
    /// not, because other nodes may reference the same content.
    async fn delete(&self, sha256: &str) -> Result<(), StorageError>;

    /// Store `bytes` at a caller-chosen `key`, overwriting any existing object.
    /// Unlike [`put`](Self::put) this is a *mutable, named* object rather than
    /// content-addressed — for state that changes in place, like a project's
    /// Y.Doc snapshot at `ydoc/{project_id}`.
    async fn put_object(&self, key: &str, bytes: &[u8]) -> Result<(), StorageError>;

    /// Fetch a named object's bytes, or `None` if it doesn't exist.
    async fn get_object(&self, key: &str) -> Result<Option<Vec<u8>>, StorageError>;
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
    async fn put(&self, bytes: &[u8]) -> Result<Blob, StorageError> {
        let sha256 = sha256_hex(bytes);
        // Dedup: identical content is already durable — don't pay to re-upload.
        if !self.exists(&sha256).await? {
            let resp = self
                .bucket
                .put_object(blob_key(&sha256), bytes)
                .await
                .map_err(|e| StorageError::Backend(e.to_string()))?;
            let code = resp.status_code();
            if !(200..300).contains(&code) {
                return Err(StorageError::Backend(format!("put returned status {code}")));
            }
        }
        Ok(Blob {
            sha256,
            size: bytes.len() as u64,
        })
    }

    async fn get(&self, sha256: &str) -> Result<Vec<u8>, StorageError> {
        if !is_valid_sha256(sha256) {
            return Err(StorageError::InvalidHash(sha256.to_string()));
        }
        let resp = self
            .bucket
            .get_object(blob_key(sha256))
            .await
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        match resp.status_code() {
            200 => Ok(resp.to_vec()),
            404 => Err(StorageError::NotFound(sha256.to_string())),
            code => Err(StorageError::Backend(format!("get returned status {code}"))),
        }
    }

    async fn exists(&self, sha256: &str) -> Result<bool, StorageError> {
        if !is_valid_sha256(sha256) {
            return Err(StorageError::InvalidHash(sha256.to_string()));
        }
        match self.bucket.head_object(blob_key(sha256)).await {
            Ok((_, 200)) => Ok(true),
            Ok((_, 404)) => Ok(false),
            Ok((_, code)) => Err(StorageError::Backend(format!("head returned status {code}"))),
            // Some backends surface a missing key as an error rather than a 404
            // status; treat an explicit not-found as "absent", everything else
            // as a real failure.
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("404") || msg.to_lowercase().contains("not found") {
                    Ok(false)
                } else {
                    Err(StorageError::Backend(msg))
                }
            }
        }
    }

    async fn delete(&self, sha256: &str) -> Result<(), StorageError> {
        if !is_valid_sha256(sha256) {
            return Err(StorageError::InvalidHash(sha256.to_string()));
        }
        let resp = self
            .bucket
            .delete_object(blob_key(sha256))
            .await
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        match resp.status_code() {
            // 404 is fine: deleting an already-absent blob is a no-op.
            200 | 204 | 404 => Ok(()),
            code => Err(StorageError::Backend(format!(
                "delete returned status {code}"
            ))),
        }
    }

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
}

/// In-memory backend for tests. Holds every blob in a map keyed by content
/// hash. The lock is never held across an `.await`, so a plain `Mutex` is fine.
#[derive(Default)]
pub struct InMemoryObjectStore {
    blobs: Mutex<HashMap<String, Vec<u8>>>,
    /// Named (mutable) objects, kept separate from content-addressed blobs so
    /// `len`/`is_empty` still reflect only blobs.
    objects: Mutex<HashMap<String, Vec<u8>>>,
}

impl InMemoryObjectStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of distinct blobs held — handy for asserting dedup in tests.
    pub fn len(&self) -> usize {
        self.blobs.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[async_trait]
impl ObjectStore for InMemoryObjectStore {
    async fn put(&self, bytes: &[u8]) -> Result<Blob, StorageError> {
        let sha256 = sha256_hex(bytes);
        self.blobs
            .lock()
            .unwrap()
            .entry(sha256.clone())
            .or_insert_with(|| bytes.to_vec());
        Ok(Blob {
            sha256,
            size: bytes.len() as u64,
        })
    }

    async fn get(&self, sha256: &str) -> Result<Vec<u8>, StorageError> {
        if !is_valid_sha256(sha256) {
            return Err(StorageError::InvalidHash(sha256.to_string()));
        }
        self.blobs
            .lock()
            .unwrap()
            .get(sha256)
            .cloned()
            .ok_or_else(|| StorageError::NotFound(sha256.to_string()))
    }

    async fn exists(&self, sha256: &str) -> Result<bool, StorageError> {
        if !is_valid_sha256(sha256) {
            return Err(StorageError::InvalidHash(sha256.to_string()));
        }
        Ok(self.blobs.lock().unwrap().contains_key(sha256))
    }

    async fn delete(&self, sha256: &str) -> Result<(), StorageError> {
        if !is_valid_sha256(sha256) {
            return Err(StorageError::InvalidHash(sha256.to_string()));
        }
        self.blobs.lock().unwrap().remove(sha256);
        Ok(())
    }

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

    #[tokio::test]
    async fn test_put_returns_content_hash_and_size() {
        let store = InMemoryObjectStore::new();
        let blob = store.put(b"hello").await.unwrap();
        assert_eq!(blob.sha256, HELLO_SHA);
        assert_eq!(blob.size, 5);
    }

    #[tokio::test]
    async fn test_put_is_content_addressed_and_dedups() {
        let store = InMemoryObjectStore::new();
        let a = store.put(b"same bytes").await.unwrap();
        let b = store.put(b"same bytes").await.unwrap();
        assert_eq!(a, b);
        // Identical content stored twice is one object.
        assert_eq!(store.len(), 1);

        // Different content is a different key.
        let c = store.put(b"other bytes").await.unwrap();
        assert_ne!(a.sha256, c.sha256);
        assert_eq!(store.len(), 2);
    }

    #[tokio::test]
    async fn test_get_roundtrips_and_reports_missing() {
        let store = InMemoryObjectStore::new();
        let blob = store.put(b"payload").await.unwrap();
        assert_eq!(store.get(&blob.sha256).await.unwrap(), b"payload");

        let absent = sha256_hex(b"never stored");
        assert!(matches!(
            store.get(&absent).await,
            Err(StorageError::NotFound(_))
        ));
    }

    #[tokio::test]
    async fn test_exists_reflects_presence() {
        let store = InMemoryObjectStore::new();
        let blob = store.put(b"payload").await.unwrap();
        assert!(store.exists(&blob.sha256).await.unwrap());
        assert!(!store.exists(&sha256_hex(b"absent")).await.unwrap());
    }

    #[tokio::test]
    async fn test_delete_is_idempotent() {
        let store = InMemoryObjectStore::new();
        let blob = store.put(b"payload").await.unwrap();
        store.delete(&blob.sha256).await.unwrap();
        assert!(!store.exists(&blob.sha256).await.unwrap());
        // Deleting again must still succeed.
        store.delete(&blob.sha256).await.unwrap();
    }

    #[tokio::test]
    async fn test_malformed_hash_is_rejected_before_backend() {
        let store = InMemoryObjectStore::new();
        for bad in ["../secret", "SHORT", &"Z".repeat(64)] {
            assert!(matches!(
                store.get(bad).await,
                Err(StorageError::InvalidHash(_))
            ));
            assert!(matches!(
                store.exists(bad).await,
                Err(StorageError::InvalidHash(_))
            ));
            assert!(matches!(
                store.delete(bad).await,
                Err(StorageError::InvalidHash(_))
            ));
        }
    }

    #[tokio::test]
    async fn test_named_object_put_get_overwrite_and_missing() {
        let store = InMemoryObjectStore::new();
        assert_eq!(store.get_object("ydoc/p1").await.unwrap(), None);

        store.put_object("ydoc/p1", b"first").await.unwrap();
        assert_eq!(store.get_object("ydoc/p1").await.unwrap().as_deref(), Some(&b"first"[..]));

        // Named objects are mutable: a second put overwrites.
        store.put_object("ydoc/p1", b"second").await.unwrap();
        assert_eq!(store.get_object("ydoc/p1").await.unwrap().as_deref(), Some(&b"second"[..]));

        // Named objects don't count as content-addressed blobs.
        assert!(store.is_empty());
    }

    /// Round-trip against a real MinIO. Ignored by default (needs a running
    /// server + bucket); run with a local stack via:
    ///   `docker compose up -d minio createbuckets`
    ///   `cargo test -p server storage:: -- --ignored`
    #[tokio::test]
    #[ignore = "requires a running MinIO (see docker-compose.yml)"]
    async fn test_minio_roundtrip() {
        let store = MinioObjectStore::new(
            "http://localhost:9000",
            "us-east-1",
            "caduceus",
            "minioadmin",
            "minioadmin",
        )
        .unwrap();

        let blob = store.put(b"integration bytes").await.unwrap();
        assert!(store.exists(&blob.sha256).await.unwrap());
        assert_eq!(store.get(&blob.sha256).await.unwrap(), b"integration bytes");
        store.delete(&blob.sha256).await.unwrap();
        assert!(!store.exists(&blob.sha256).await.unwrap());
    }
}
