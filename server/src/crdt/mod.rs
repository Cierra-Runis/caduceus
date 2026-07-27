//! CRDT (yrs) encoding of the project file tree.
//!
//! A project's Y.Doc holds one top-level map, `nodes: Map<NodeId, Map>`. Each
//! entry is a node's own map with `kind` / `name` (+ `parent` unless it's at the
//! root, + `sha256` / `size` for a file). Every field is its **own** CRDT cell,
//! so concurrent edits to different fields of one node — the canonical
//! rename-vs-move race — merge instead of clobbering each other. (That is the
//! whole reason a node isn't stored as one opaque JSON blob.)
//!
//! This module is only the **codec** between that Y.Doc shape and the pure
//! [`ProjectTree`] domain model: [`read_tree`] decodes, [`write_node`] /
//! [`write_tree`] encode. It performs no validation — the caller runs
//! [`ProjectTree::validate`] on the decoded tree. Field-level *mutation* (the
//! authority applying a single incoming change) is a later, room-layer concern;
//! here `write_*` set whole nodes, which is what seeding and tests need.

use yrs::{Any, Doc, Map, MapPrelim, MapRef, Out, ReadTxn, TransactionMut};

use crate::models::tree::{Node, NodeContent, ProjectTree};
use crate::storage::Blob;

pub mod snapshot;

/// Name of the top-level nodes map in a project's Y.Doc.
pub const NODES: &str = "nodes";

/// A node's Y.Doc representation is malformed — a field is missing, has the
/// wrong CRDT type, or carries an unknown `kind`. Distinct from
/// [`crate::models::tree::TreeError`], which is about an otherwise well-formed
/// tree breaking a *rule*.
#[derive(Debug, derive_more::Display, PartialEq, Eq)]
pub enum CodecError {
    #[display("node {id}: a node entry is not a map")]
    NotAMap { id: String },
    #[display("node {id}: missing field {field}")]
    MissingField { id: String, field: &'static str },
    #[display("node {id}: field {field} has the wrong type")]
    BadType { id: String, field: &'static str },
    #[display("node {id}: unknown kind {kind:?}")]
    UnknownKind { id: String, kind: String },
}

impl std::error::Error for CodecError {}

/// The top-level nodes map of a project's Y.Doc, creating it if absent.
pub fn nodes_map(doc: &Doc) -> MapRef {
    doc.get_or_insert_map(NODES)
}

/// Decode the whole tree from the `nodes` map. Does not validate — call
/// [`ProjectTree::validate`] on the result.
pub fn read_tree<T: ReadTxn>(txn: &T, nodes: &MapRef) -> Result<ProjectTree, CodecError> {
    let mut out = Vec::with_capacity(nodes.len(txn) as usize);
    for (id, value) in nodes.iter(txn) {
        let node_map = match value {
            Out::YMap(m) => m,
            _ => return Err(CodecError::NotAMap { id: id.to_string() }),
        };
        out.push(read_node(txn, id, &node_map)?);
    }
    Ok(ProjectTree::from_nodes(out))
}

fn read_node<T: ReadTxn>(txn: &T, id: &str, m: &MapRef) -> Result<Node, CodecError> {
    let kind = req_str(txn, m, id, "kind")?;
    let name = req_str(txn, m, id, "name")?;
    let parent = opt_str(txn, m, "parent");
    let content = match kind.as_str() {
        "folder" => NodeContent::Folder,
        "file" => {
            let sha256 = req_str(txn, m, id, "sha256")?;
            let size = req_u64(txn, m, id, "size")?;
            NodeContent::File {
                blob: Blob { sha256, size },
            }
        }
        _ => {
            return Err(CodecError::UnknownKind {
                id: id.to_string(),
                kind,
            });
        }
    };
    Ok(Node {
        id: id.to_string(),
        parent,
        name,
        content,
    })
}

/// Encode a single node into the `nodes` map (overwriting any existing entry for
/// its id). Sets each field as its own CRDT cell.
pub fn write_node(txn: &mut TransactionMut, nodes: &MapRef, node: &Node) {
    let m = nodes.insert(txn, node.id.clone(), MapPrelim::default());
    match &node.content {
        NodeContent::Folder => {
            m.insert(txn, "kind", "folder");
        }
        NodeContent::File { blob } => {
            m.insert(txn, "kind", "file");
            m.insert(txn, "sha256", blob.sha256.clone());
            m.insert(txn, "size", blob.size as i64);
        }
    }
    m.insert(txn, "name", node.name.clone());
    if let Some(parent) = &node.parent {
        m.insert(txn, "parent", parent.clone());
    }
}

/// Encode a whole tree into the `nodes` map.
pub fn write_tree(txn: &mut TransactionMut, nodes: &MapRef, tree: &ProjectTree) {
    for node in tree.iter() {
        write_node(txn, nodes, node);
    }
}

fn req_str<T: ReadTxn>(
    txn: &T,
    m: &MapRef,
    id: &str,
    field: &'static str,
) -> Result<String, CodecError> {
    match m.get(txn, field) {
        Some(Out::Any(Any::String(s))) => Ok(s.to_string()),
        Some(_) => Err(CodecError::BadType {
            id: id.to_string(),
            field,
        }),
        None => Err(CodecError::MissingField {
            id: id.to_string(),
            field,
        }),
    }
}

fn opt_str<T: ReadTxn>(txn: &T, m: &MapRef, field: &str) -> Option<String> {
    match m.get(txn, field) {
        Some(Out::Any(Any::String(s))) => Some(s.to_string()),
        _ => None,
    }
}

fn req_u64<T: ReadTxn>(
    txn: &T,
    m: &MapRef,
    id: &str,
    field: &'static str,
) -> Result<u64, CodecError> {
    match m.get(txn, field) {
        // yrs normalizes a small integer to `Number` (f64) rather than `BigInt`
        // when it round-trips through lib0, so accept either. A file size is far
        // inside f64's exact-integer range (2^53), so no precision is lost.
        Some(Out::Any(Any::BigInt(n))) if n >= 0 => Ok(n as u64),
        Some(Out::Any(Any::Number(f))) if f >= 0.0 && f.fract() == 0.0 => Ok(f as u64),
        Some(_) => Err(CodecError::BadType {
            id: id.to_string(),
            field,
        }),
        None => Err(CodecError::MissingField {
            id: id.to_string(),
            field,
        }),
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use yrs::Transact;

    use super::*;

    fn folder(id: &str, parent: Option<&str>, name: &str) -> Node {
        Node {
            id: id.to_string(),
            parent: parent.map(String::from),
            name: name.to_string(),
            content: NodeContent::Folder,
        }
    }

    fn file(id: &str, parent: Option<&str>, name: &str, size: u64) -> Node {
        Node {
            id: id.to_string(),
            parent: parent.map(String::from),
            name: name.to_string(),
            content: NodeContent::File {
                blob: Blob {
                    sha256: "a".repeat(64),
                    size,
                },
            },
        }
    }

    /// Write a tree into a fresh Doc, read it back, return the decoded tree.
    fn roundtrip(tree: &ProjectTree) -> ProjectTree {
        let doc = Doc::new();
        let nodes = nodes_map(&doc);
        {
            let mut txn = doc.transact_mut();
            write_tree(&mut txn, &nodes, tree);
        }
        let txn = doc.transact();
        read_tree(&txn, &nodes).unwrap()
    }

    #[test]
    fn test_roundtrip_preserves_the_tree() {
        let tree = ProjectTree::from_nodes([
            folder("d", None, "chapters"),
            file("f", Some("d"), "intro.typ", 42),
            file("r", None, "main.typ", 0),
        ]);
        // Decodes back to an identical tree, and it's still valid.
        let read = roundtrip(&tree);
        assert_eq!(read, tree);
        read.validate().unwrap();
        assert_eq!(read.path_of("f").unwrap(), "chapters/intro.typ");
    }

    #[test]
    fn test_file_blob_survives_roundtrip() {
        let tree = ProjectTree::from_nodes([file("f", None, "a.typ", 123)]);
        let read = roundtrip(&tree);
        let blob = read.get("f").unwrap().blob().unwrap();
        assert_eq!(blob.sha256, "a".repeat(64));
        assert_eq!(blob.size, 123);
    }

    #[test]
    fn test_root_node_has_no_parent_field() {
        // A root-level node omits `parent` on the wire; it must decode as `None`,
        // not as some empty string.
        let doc = Doc::new();
        let nodes = nodes_map(&doc);
        {
            let mut txn = doc.transact_mut();
            write_node(&mut txn, &nodes, &file("r", None, "main.typ", 0));
        }
        let txn = doc.transact();
        let node_map = match nodes.get(&txn, "r").unwrap() {
            Out::YMap(m) => m,
            _ => panic!("expected a map"),
        };
        assert!(node_map.get(&txn, "parent").is_none());
        assert_eq!(read_tree(&txn, &nodes).unwrap().get("r").unwrap().parent, None);
    }

    #[test]
    fn test_missing_field_is_a_codec_error() {
        // A node map with a kind but no name.
        let doc = Doc::new();
        let nodes = nodes_map(&doc);
        {
            let mut txn = doc.transact_mut();
            let m = nodes.insert(&mut txn, "x", MapPrelim::default());
            m.insert(&mut txn, "kind", "folder");
        }
        let txn = doc.transact();
        assert_eq!(
            read_tree(&txn, &nodes),
            Err(CodecError::MissingField {
                id: "x".to_string(),
                field: "name",
            })
        );
    }

    #[test]
    fn test_unknown_kind_is_a_codec_error() {
        let doc = Doc::new();
        let nodes = nodes_map(&doc);
        {
            let mut txn = doc.transact_mut();
            let m = nodes.insert(&mut txn, "x", MapPrelim::default());
            m.insert(&mut txn, "kind", "symlink");
            m.insert(&mut txn, "name", "link");
        }
        let txn = doc.transact();
        assert_eq!(
            read_tree(&txn, &nodes),
            Err(CodecError::UnknownKind {
                id: "x".to_string(),
                kind: "symlink".to_string(),
            })
        );
    }

    #[test]
    fn test_wrong_type_is_a_codec_error() {
        // `name` present but as a number, not a string.
        let doc = Doc::new();
        let nodes = nodes_map(&doc);
        {
            let mut txn = doc.transact_mut();
            let m = nodes.insert(&mut txn, "x", MapPrelim::default());
            m.insert(&mut txn, "kind", "folder");
            m.insert(&mut txn, "name", 7i64);
        }
        let txn = doc.transact();
        assert_eq!(
            read_tree(&txn, &nodes),
            Err(CodecError::BadType {
                id: "x".to_string(),
                field: "name",
            })
        );
    }
}
