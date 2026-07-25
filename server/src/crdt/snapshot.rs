//! Persisting a project's Y.Doc as a snapshot in object storage.
//!
//! The whole CRDT state is encoded as a single yrs update and written to the
//! **mutable, named** key `ydoc/{project_id}`, overwritten on each save — as
//! opposed to the immutable content-addressed `blobs/{sha}`. Loading rebuilds a
//! `Doc` by applying that update onto an empty one.
//!
//! This is the durable source of truth for a room's CRDT state: a room rehydrates
//! from its snapshot on cold start (rather than re-seeding from text, which would
//! duplicate content), and the Mongo projection is a rebuildable cache derived
//! from it.

use yrs::updates::decoder::Decode;
use yrs::{Doc, ReadTxn, StateVector, Transact, Update};

use crate::storage::{ObjectStore, StorageError};

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

/// Object key for a project's Y.Doc snapshot.
fn snapshot_key(project_id: &str) -> String {
    format!("ydoc/{project_id}")
}

/// Encode the full CRDT state of `doc` as a single v1 update.
pub fn encode_doc(doc: &Doc) -> Vec<u8> {
    doc.transact()
        .encode_state_as_update_v1(&StateVector::default())
}

/// Save `doc`'s full state to `ydoc/{project_id}`, replacing any prior snapshot.
pub async fn save_snapshot(
    store: &dyn ObjectStore,
    project_id: &str,
    doc: &Doc,
) -> Result<(), SnapshotError> {
    store
        .put_object(&snapshot_key(project_id), &encode_doc(doc))
        .await?;
    Ok(())
}

/// Load a project's `Doc` from its snapshot, or `None` if it has none yet (a
/// brand-new project). Errors only if a snapshot exists but is corrupt.
pub async fn load_snapshot(
    store: &dyn ObjectStore,
    project_id: &str,
) -> Result<Option<Doc>, SnapshotError> {
    let Some(bytes) = store.get_object(&snapshot_key(project_id)).await? else {
        return Ok(None);
    };
    let update =
        Update::decode_v1(&bytes).map_err(|e| SnapshotError::Decode(e.to_string()))?;
    let doc = Doc::new();
    doc.transact_mut()
        .apply_update(update)
        .map_err(|e| SnapshotError::Decode(e.to_string()))?;
    Ok(Some(doc))
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::super::{nodes_map, read_tree, write_tree};
    use super::*;
    use crate::models::tree::{Node, NodeContent, ProjectTree};
    use crate::storage::{Blob, InMemoryObjectStore};

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
        let store = InMemoryObjectStore::new();
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
        let store = InMemoryObjectStore::new();
        assert!(load_snapshot(&store, "never-saved").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_corrupt_snapshot_is_a_decode_error() {
        let store = InMemoryObjectStore::new();
        store.put_object("ydoc/proj1", b"not a yrs update").await.unwrap();
        assert!(matches!(
            load_snapshot(&store, "proj1").await,
            Err(SnapshotError::Decode(_))
        ));
    }
}
