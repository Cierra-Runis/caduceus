//! The unified project file tree — **id is identity, path is derived**.
//!
//! Today a project keeps `files: Vec<ProjectFile>` keyed by full `path`, plus a
//! separate text room also keyed by path. That makes `path` the primary key, so
//! a rename changes a file's identity and a concurrent rename+edit cannot merge
//! (see `docs/Architecture - Compilation and Project Model.md`).
//!
//! This module is the replacement domain model: every node — file *or* folder —
//! has a stable [`NodeId`] and stores only its own `name` segment plus a
//! `parent` link, so the full path is *derived* by walking the parent chain and
//! a rename is one field write that never touches a key.
//!
//! [`Node`] is a sum type: a file and a folder are different shapes, so the data
//! only one of them has (a file's blob) lives in that variant and the illegal
//! "folder with content" state cannot be represented at all. The shared fields
//! (`id`/`parent`/`name`) are repeated per variant on purpose — one `match`
//! then yields both the kind and its data.
//!
//! It is deliberately CRDT-agnostic: a [`ProjectTree`] is a plain in-memory
//! structure the server authority builds from a Y.Doc (later) to **validate** an
//! incoming change and **derive** paths + the REST/Mongo projection. Keeping the
//! rules here — pure and heavily tested — is what lets the eventual yrs binding
//! stay a thin serialization layer.

use std::collections::{HashMap, HashSet};

use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::storage::Blob;

/// A node's stable identity. A string (rather than `ObjectId`) so a client can
/// mint ids for offline-created nodes; the tree rules never inspect its shape,
/// only its uniqueness (guaranteed by the CRDT map that keys nodes by id).
pub type NodeId = String;

/// Longest allowed single path segment (one `name`), in bytes.
pub const MAX_NAME_LEN: usize = 255;
/// Deepest allowed nesting (root-level node = depth 1). Also bounds the
/// parent-chain walk so a cycle can't loop forever.
pub const MAX_DEPTH: usize = 64;

/// One node in the tree — a file or a folder. Each variant carries the shared
/// fields plus its own kind-specific data; there is no separate "kind" tag and
/// no optional blob that only applies to files, so a folder-with-content is
/// unrepresentable rather than merely invalid.
///
/// Serialized internally-tagged as `{ "kind": "file"|"folder", … }`, which keeps
/// the wire shape flat for the CRDT/clients.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Node {
    File {
        id: NodeId,
        /// Parent folder's id, or `None` for a node directly under the root.
        #[serde(default)]
        parent: Option<NodeId>,
        /// A single path segment (e.g. `intro.typ`), never a full path.
        name: String,
        /// The file's content-addressed bytes, or `None` for a freshly-created
        /// file whose bytes haven't been flushed to a blob yet (its text lives
        /// only in the CRDT overlay until then).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        blob: Option<Blob>,
    },
    Folder {
        id: NodeId,
        #[serde(default)]
        parent: Option<NodeId>,
        name: String,
    },
}

impl Node {
    pub fn id(&self) -> &str {
        match self {
            Node::File { id, .. } | Node::Folder { id, .. } => id,
        }
    }

    pub fn parent(&self) -> Option<&str> {
        match self {
            Node::File { parent, .. } | Node::Folder { parent, .. } => parent.as_deref(),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Node::File { name, .. } | Node::Folder { name, .. } => name,
        }
    }

    /// This node's blob reference — always `None` for a folder, and `None` for a
    /// file whose content hasn't been flushed yet.
    pub fn blob(&self) -> Option<&Blob> {
        match self {
            Node::File { blob, .. } => blob.as_ref(),
            Node::Folder { .. } => None,
        }
    }

    pub fn is_folder(&self) -> bool {
        matches!(self, Node::Folder { .. })
    }

    pub fn is_file(&self) -> bool {
        matches!(self, Node::File { .. })
    }
}

/// Why a tree (or a proposed change to it) is invalid. The authority rejects an
/// incoming CRDT update whose resulting tree fails [`ProjectTree::validate`].
#[derive(Debug, Display, PartialEq, Eq)]
pub enum TreeError {
    #[display("node {_0} not found")]
    MissingNode(NodeId),
    #[display("invalid name {name:?} for node {id}")]
    InvalidName { id: NodeId, name: String },
    #[display("duplicate name {name:?} under parent {parent:?}")]
    DuplicateName {
        parent: Option<NodeId>,
        name: String,
    },
    #[display("parent {_0} does not exist")]
    ParentNotFound(NodeId),
    #[display("parent {_0} is a file, not a folder")]
    ParentNotFolder(NodeId),
    #[display("cycle detected at node {_0}")]
    Cycle(NodeId),
    #[display("node {_0} exceeds the maximum tree depth")]
    TooDeep(NodeId),
}

impl std::error::Error for TreeError {}

/// True iff `name` is a legal single path segment: non-empty, within the length
/// cap, not `.`/`..`, free of `/`, `\`, control characters, and leading/trailing
/// whitespace. Segment-level because a node stores only its own segment; the
/// full path is derived by joining validated segments with `/`.
pub fn is_valid_segment(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= MAX_NAME_LEN
        && name != "."
        && name != ".."
        && !name.contains('/')
        && !name.contains('\\')
        && !name.chars().any(|c| c.is_control())
        && name.trim() == name
}

/// A REST/Mongo-facing view of one node: the node itself plus its derived
/// `path`, serialized flat (`{ kind, id, parent, name, blob?, path }`). This is
/// the *projection* the metadata store keeps so listings and access checks
/// don't need to load the Y.Doc; it can be rebuilt from the tree at any time.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NodeProjection {
    #[serde(flatten)]
    pub node: Node,
    pub path: String,
}

/// An in-memory project file tree: nodes keyed by id. Built by the authority
/// from a Y.Doc to validate and to derive paths / the projection.
#[derive(Debug, Clone, Default)]
pub struct ProjectTree {
    nodes: HashMap<NodeId, Node>,
}

impl ProjectTree {
    /// Build a tree from a set of nodes. Ids are assumed unique (the CRDT keys
    /// nodes by id); a duplicate id would collapse to the last one. Call
    /// [`validate`](Self::validate) before trusting the result.
    pub fn from_nodes(nodes: impl IntoIterator<Item = Node>) -> Self {
        Self {
            nodes: nodes
                .into_iter()
                .map(|n| (n.id().to_string(), n))
                .collect(),
        }
    }

    pub fn get(&self, id: &str) -> Option<&Node> {
        self.nodes.get(id)
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Direct children of `parent` (`None` = project root), unordered.
    pub fn children(&self, parent: Option<&str>) -> impl Iterator<Item = &Node> {
        self.nodes.values().filter(move |n| n.parent() == parent)
    }

    /// Walk from `id` up to a root-level node, returning the chain
    /// `[self, …, root_level]`. Detects cycles and over-deep chains so callers
    /// (path derivation, validation) can't loop forever.
    fn ascend(&self, id: &str) -> Result<Vec<&Node>, TreeError> {
        let mut chain: Vec<&Node> = Vec::new();
        let mut seen: HashSet<&str> = HashSet::new();
        let mut cursor: Option<&str> = Some(id);
        while let Some(cid) = cursor {
            if !seen.insert(cid) {
                return Err(TreeError::Cycle(cid.to_string()));
            }
            if chain.len() >= MAX_DEPTH {
                return Err(TreeError::TooDeep(cid.to_string()));
            }
            let node = self
                .nodes
                .get(cid)
                .ok_or_else(|| TreeError::MissingNode(cid.to_string()))?;
            chain.push(node);
            cursor = node.parent();
        }
        Ok(chain)
    }

    /// The derived path of a node: its ancestors' names from the root joined
    /// with `/`, e.g. `chapters/intro.typ`. Errors if the id is unknown or its
    /// chain is broken/cyclic.
    pub fn path_of(&self, id: &str) -> Result<String, TreeError> {
        let chain = self.ascend(id)?;
        // `rev()` gives root → leaf, which is the path order.
        let path = chain
            .iter()
            .rev()
            .map(|n| n.name())
            .collect::<Vec<_>>()
            .join("/");
        Ok(path)
    }

    /// Validate the whole tree. Enforces, for every node:
    /// - a legal `name` segment;
    /// - `parent` exists and is a folder;
    /// - names are unique among siblings (which also forbids a file and a
    ///   folder sharing a name under one parent);
    /// - no cycles and no over-deep chains.
    ///
    /// (A folder carrying content needs no check — it's unrepresentable.)
    pub fn validate(&self) -> Result<(), TreeError> {
        // Per-node structural checks.
        for node in self.nodes.values() {
            if !is_valid_segment(node.name()) {
                return Err(TreeError::InvalidName {
                    id: node.id().to_string(),
                    name: node.name().to_string(),
                });
            }
            if let Some(pid) = node.parent() {
                match self.nodes.get(pid) {
                    None => return Err(TreeError::ParentNotFound(pid.to_string())),
                    Some(p) if !p.is_folder() => {
                        return Err(TreeError::ParentNotFolder(pid.to_string()));
                    }
                    _ => {}
                }
            }
        }

        // Sibling-name uniqueness: no two nodes share (parent, name).
        let mut siblings: HashSet<(Option<&str>, &str)> = HashSet::new();
        for node in self.nodes.values() {
            if !siblings.insert((node.parent(), node.name())) {
                return Err(TreeError::DuplicateName {
                    parent: node.parent().map(String::from),
                    name: node.name().to_string(),
                });
            }
        }

        // Acyclic + bounded depth: every node must reach a root.
        for id in self.nodes.keys() {
            self.ascend(id)?;
        }

        Ok(())
    }

    /// Derive every node's path. Assumes/validates a well-formed tree.
    pub fn paths(&self) -> Result<HashMap<NodeId, String>, TreeError> {
        self.nodes
            .keys()
            .map(|id| Ok((id.clone(), self.path_of(id)?)))
            .collect()
    }

    /// Build the flattened projection (each node plus its derived path) for the
    /// metadata store / REST. Errors if the tree is malformed.
    pub fn projection(&self) -> Result<Vec<NodeProjection>, TreeError> {
        let mut out = Vec::with_capacity(self.nodes.len());
        for node in self.nodes.values() {
            out.push(NodeProjection {
                path: self.path_of(node.id())?,
                node: node.clone(),
            });
        }
        Ok(out)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    fn folder(id: &str, parent: Option<&str>, name: &str) -> Node {
        Node::Folder {
            id: id.to_string(),
            parent: parent.map(String::from),
            name: name.to_string(),
        }
    }

    fn file(id: &str, parent: Option<&str>, name: &str) -> Node {
        Node::File {
            id: id.to_string(),
            parent: parent.map(String::from),
            name: name.to_string(),
            blob: None,
        }
    }

    #[test]
    fn test_is_valid_segment() {
        assert!(is_valid_segment("main.typ"));
        assert!(is_valid_segment("chapters"));
        assert!(is_valid_segment("a nice name.bib"));
        assert!(!is_valid_segment("")); // empty
        assert!(!is_valid_segment(".")); // dot
        assert!(!is_valid_segment("..")); // dotdot
        assert!(!is_valid_segment("a/b")); // separator
        assert!(!is_valid_segment("a\\b")); // backslash
        assert!(!is_valid_segment(" leading")); // leading ws
        assert!(!is_valid_segment("trailing ")); // trailing ws
        assert!(!is_valid_segment("with\tctrl")); // control char
        assert!(!is_valid_segment(&"x".repeat(MAX_NAME_LEN + 1))); // too long
    }

    #[test]
    fn test_empty_tree_is_valid() {
        let tree = ProjectTree::default();
        assert!(tree.is_empty());
        assert!(tree.validate().is_ok());
        assert_eq!(tree.projection().unwrap(), vec![]);
    }

    #[test]
    fn test_file_node_serializes_flat_with_kind_and_blob() {
        let f = Node::File {
            id: "1".to_string(),
            parent: Some("d".to_string()),
            name: "a.typ".to_string(),
            blob: Some(Blob {
                sha256: "a".repeat(64),
                size: 3,
            }),
        };
        let json = serde_json::to_value(&f).unwrap();
        // Internally tagged: kind + fields sit flat, no nesting.
        assert_eq!(json["kind"], "file");
        assert_eq!(json["blob"]["size"], 3);
        assert_eq!(json["parent"], "d");
        // Round-trips back to the same node.
        assert_eq!(serde_json::from_value::<Node>(json).unwrap(), f);
    }

    #[test]
    fn test_folder_node_serializes_without_a_blob_field() {
        let d = folder("d", None, "chapters");
        let json = serde_json::to_value(&d).unwrap();
        assert_eq!(json["kind"], "folder");
        // A folder has no blob field at all — the illegal state doesn't exist.
        assert!(json.get("blob").is_none());
        assert_eq!(serde_json::from_value::<Node>(json).unwrap(), d);
    }

    #[test]
    fn test_new_file_omits_blob_until_flushed() {
        // A freshly-created file (no bytes yet) serializes without `blob`.
        let f = file("1", None, "new.typ");
        let json = serde_json::to_value(&f).unwrap();
        assert_eq!(json["kind"], "file");
        assert!(json.get("blob").is_none());
        assert_eq!(serde_json::from_value::<Node>(json).unwrap(), f);
    }

    #[test]
    fn test_root_level_path_is_the_name() {
        let tree = ProjectTree::from_nodes([file("1", None, "main.typ")]);
        tree.validate().unwrap();
        assert_eq!(tree.path_of("1").unwrap(), "main.typ");
    }

    #[test]
    fn test_nested_path_is_derived_from_parent_chain() {
        let tree = ProjectTree::from_nodes([
            folder("root", None, "chapters"),
            folder("sub", Some("root"), "part1"),
            file("f", Some("sub"), "intro.typ"),
        ]);
        tree.validate().unwrap();
        assert_eq!(tree.path_of("f").unwrap(), "chapters/part1/intro.typ");
        assert_eq!(tree.path_of("sub").unwrap(), "chapters/part1");
        assert_eq!(tree.path_of("root").unwrap(), "chapters");
    }

    #[test]
    fn test_rename_is_one_field_and_repaths_descendants() {
        let mut dir = folder("d", None, "chapters");
        // Rename the folder: mutate its `name`, id untouched.
        if let Node::Folder { name, .. } = &mut dir {
            *name = "sections".to_string();
        }
        let tree = ProjectTree::from_nodes([dir, file("f", Some("d"), "intro.typ")]);
        tree.validate().unwrap();
        assert_eq!(tree.path_of("f").unwrap(), "sections/intro.typ");
    }

    #[test]
    fn test_duplicate_sibling_name_rejected() {
        let tree =
            ProjectTree::from_nodes([file("1", None, "a.typ"), file("2", None, "a.typ")]);
        assert_eq!(
            tree.validate(),
            Err(TreeError::DuplicateName {
                parent: None,
                name: "a.typ".to_string(),
            })
        );
    }

    #[test]
    fn test_file_and_folder_cannot_share_a_name_under_one_parent() {
        // Same (parent, name) for a file and a folder is a duplicate — this is
        // how "a can't be both a file and a directory" falls out for free.
        let tree =
            ProjectTree::from_nodes([file("1", None, "assets"), folder("2", None, "assets")]);
        assert!(matches!(
            tree.validate(),
            Err(TreeError::DuplicateName { .. })
        ));
    }

    #[test]
    fn test_same_name_under_different_parents_is_fine() {
        let tree = ProjectTree::from_nodes([
            folder("a", None, "a"),
            folder("b", None, "b"),
            file("1", Some("a"), "x.typ"),
            file("2", Some("b"), "x.typ"),
        ]);
        tree.validate().unwrap();
    }

    #[test]
    fn test_invalid_name_rejected() {
        let tree = ProjectTree::from_nodes([file("1", None, "../escape")]);
        assert_eq!(
            tree.validate(),
            Err(TreeError::InvalidName {
                id: "1".to_string(),
                name: "../escape".to_string(),
            })
        );
    }

    #[test]
    fn test_parent_must_exist() {
        let tree = ProjectTree::from_nodes([file("1", Some("ghost"), "a.typ")]);
        assert_eq!(
            tree.validate(),
            Err(TreeError::ParentNotFound("ghost".to_string()))
        );
    }

    #[test]
    fn test_parent_must_be_a_folder() {
        let tree = ProjectTree::from_nodes([
            file("p", None, "main.typ"),
            file("c", Some("p"), "child.typ"),
        ]);
        assert_eq!(
            tree.validate(),
            Err(TreeError::ParentNotFolder("p".to_string()))
        );
    }

    // A folder-with-blob is unrepresentable now (`Node::Folder` has no blob
    // field), so there is no runtime rule — and no test — for it.

    #[test]
    fn test_cycle_detected() {
        // a → b → a: a parent chain that never reaches a root.
        let tree =
            ProjectTree::from_nodes([folder("a", Some("b"), "a"), folder("b", Some("a"), "b")]);
        assert!(matches!(tree.validate(), Err(TreeError::Cycle(_))));
    }

    #[test]
    fn test_too_deep_rejected() {
        let mut nodes = Vec::new();
        // Build a chain of MAX_DEPTH + 1 nested folders.
        for i in 0..=MAX_DEPTH {
            let parent = if i == 0 {
                None
            } else {
                Some(format!("n{}", i - 1))
            };
            nodes.push(folder(&format!("n{i}"), parent.as_deref(), &format!("d{i}")));
        }
        let tree = ProjectTree::from_nodes(nodes);
        assert!(matches!(tree.validate(), Err(TreeError::TooDeep(_))));
    }

    #[test]
    fn test_path_of_unknown_id() {
        let tree = ProjectTree::from_nodes([file("1", None, "a.typ")]);
        assert_eq!(
            tree.path_of("nope"),
            Err(TreeError::MissingNode("nope".to_string()))
        );
    }

    #[test]
    fn test_projection_carries_derived_paths_and_blobs() {
        let blob = Blob {
            sha256: "a".repeat(64),
            size: 12,
        };
        let f = Node::File {
            id: "f".to_string(),
            parent: Some("d".to_string()),
            name: "intro.typ".to_string(),
            blob: Some(blob.clone()),
        };
        let tree = ProjectTree::from_nodes([folder("d", None, "chapters"), f]);
        tree.validate().unwrap();

        let mut proj = tree.projection().unwrap();
        proj.sort_by(|a, b| a.path.cmp(&b.path));
        assert_eq!(proj[0].path, "chapters");
        assert!(proj[0].node.is_folder());
        assert_eq!(proj[1].path, "chapters/intro.typ");
        assert!(proj[1].node.is_file());
        assert_eq!(proj[1].node.blob(), Some(&blob));
    }

    #[test]
    fn test_children_lists_direct_descendants() {
        let tree = ProjectTree::from_nodes([
            folder("d", None, "chapters"),
            file("1", Some("d"), "a.typ"),
            file("2", Some("d"), "b.typ"),
            file("3", None, "root.typ"),
        ]);
        let mut names: Vec<&str> = tree.children(Some("d")).map(|n| n.name()).collect();
        names.sort();
        assert_eq!(names, vec!["a.typ", "b.typ"]);

        // Root level holds both the top folder and the top-level file.
        let mut root: Vec<&str> = tree.children(None).map(|n| n.name()).collect();
        root.sort();
        assert_eq!(root, vec!["chapters", "root.typ"]);
    }
}
