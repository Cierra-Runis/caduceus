//! Persisting a project's Y.Doc as a snapshot in object storage.
//!
//! The whole CRDT state is encoded as a single yrs update and written to the
//! project's snapshot object (`projects/{project_id}/ydoc`, see
//! [`ProjectStore`]), overwritten on each save. Loading rebuilds a `Doc` by
//! applying that update onto an empty one.
//!
//! This is the durable source of truth for a room's CRDT state: a room rehydrates
//! from its snapshot on cold start (rather than re-seeding from text, which would
//! duplicate content), and the Mongo projection is a rebuildable cache derived
//! from it. This module owns only the yrs encode/decode; where the bytes live is
//! [`ProjectStore`]'s concern.

use yrs::updates::decoder::Decode;
use yrs::{Doc, ReadTxn, StateVector, Transact, Update};

use crate::storage::{ProjectStore, StorageError};

/// A snapshot couldn't be persisted or restored.
#[derive(Debug, derive_more::Display)]
pub enum SnapshotError {
    #[display("snapshot storage error: {_0}")]
    Storage(StorageError),
    /// Stored bytes weren't a decodable/applicable yrs update (corruption).
    #[display("snapshot decode error: {_0}")]
    Decode(String),
}

impl std::error::Error for SnapshotError {}

impl From<StorageError> for SnapshotError {
    fn from(e: StorageError) -> Self {
        SnapshotError::Storage(e)
    }
}

/// Encode the full CRDT state of `doc` as a single v1 update. Callers that hold
/// a `!Send` `Doc` encode it themselves (e.g. a room persist loop) and hand the
/// bytes to [`ProjectStore::put_snapshot`].
pub fn encode_doc(doc: &Doc) -> Vec<u8> {
    doc.transact()
        .encode_state_as_update_v1(&StateVector::default())
}

/// Decode raw snapshot bytes (a yrs v1 update) into a fresh `Doc`. Errors only
/// if the bytes are corrupt.
pub fn decode_doc(bytes: &[u8]) -> Result<Doc, SnapshotError> {
    let update = Update::decode_v1(bytes).map_err(|e| SnapshotError::Decode(e.to_string()))?;
    let doc = Doc::new();
    doc.transact_mut()
        .apply_update(update)
        .map_err(|e| SnapshotError::Decode(e.to_string()))?;
    Ok(doc)
}

/// Save `doc`'s full state as `project_id`'s snapshot, replacing any prior one.
pub async fn save_snapshot(
    store: &ProjectStore,
    project_id: &str,
    doc: &Doc,
) -> Result<(), SnapshotError> {
    store.put_snapshot(project_id, &encode_doc(doc)).await?;
    Ok(())
}

/// Load a project's `Doc` from its snapshot, or `None` if it has none yet (a
/// brand-new project). Errors only if a snapshot exists but is corrupt.
pub async fn load_snapshot(
    store: &ProjectStore,
    project_id: &str,
) -> Result<Option<Doc>, SnapshotError> {
    let Some(bytes) = store.get_snapshot(project_id).await? else {
        return Ok(None);
    };
    Ok(Some(decode_doc(&bytes)?))
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::sync::Arc;

    use super::super::{nodes_map, read_tree, write_tree};
    use super::*;
    use crate::models::tree::{Node, NodeContent, ProjectTree};
    use crate::storage::{Blob, InMemoryObjectStore};

    fn store() -> ProjectStore {
        ProjectStore::new(Arc::new(InMemoryObjectStore::new()))
    }

    fn sample_tree() -> ProjectTree {
        ProjectTree::from_nodes([
            Node {
                id: "d".to_string(),
                parent: None,
                name: "chapters".to_string(),
                content: NodeContent::Folder,
            },
            Node {
                id: "f".to_string(),
                parent: Some("d".to_string()),
                name: "intro.typ".to_string(),
                content: NodeContent::File {
                    blob: Blob {
                        sha256: "a".repeat(64),
                        size: 7,
                    },
                },
            },
        ])
    }

    #[tokio::test]
    async fn test_save_then_load_reconstructs_the_tree() {
        let store = store();
        let tree = sample_tree();

        // Build a doc holding the tree, snapshot it.
        let doc = Doc::new();
        let nodes = nodes_map(&doc);
        {
            let mut txn = doc.transact_mut();
            write_tree(&mut txn, &nodes, &tree);
        }
        save_snapshot(&store, "proj1", &doc).await.unwrap();

        // Load into a fresh doc and decode the tree back. Take the map handle
        // *before* opening the read txn — yrs allows only one live transaction
        // per doc, so creating the map inside the same expression would deadlock.
        let loaded = load_snapshot(&store, "proj1").await.unwrap().unwrap();
        let nodes = nodes_map(&loaded);
        let read = read_tree(&loaded.transact(), &nodes).unwrap();
        assert_eq!(read, tree);
    }

    #[tokio::test]
    async fn test_load_missing_snapshot_is_none() {
        let store = store();
        assert!(load_snapshot(&store, "never-saved").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_corrupt_snapshot_is_a_decode_error() {
        let store = store();
        store.put_snapshot("proj1", b"not a yrs update").await.unwrap();
        assert!(matches!(
            load_snapshot(&store, "proj1").await,
            Err(SnapshotError::Decode(_))
        ));
    }
}
