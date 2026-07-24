//! Building an id-keyed [`Node`] tree from the legacy path-keyed file list.
//!
//! A room cold-starts from its snapshot; when there is none yet (a project that
//! predates the CRDT tree), it is seeded from the Mongo `files` — a flat list of
//! full paths like `chapters/intro.typ`. This module turns that into explicit
//! nodes: a file node per file, plus the folder nodes its path implies.
//!
//! Folder node ids are the folder's own path (`chapters`, `chapters/part1`) —
//! deterministic, so re-seeding the same files yields the same tree — while file
//! nodes keep their real id. The two id spaces don't collide (file ids are
//! 24-char hex; folder ids contain a segment name).

use std::collections::HashMap;

use crate::models::tree::{Node, NodeContent, NodeId};
use crate::storage::Blob;

/// Build the node set for `files`, where each entry is
/// `(file_id, full_path, blob)`. Returns file nodes plus every folder node their
/// paths imply. Order is unspecified; feed the result to
/// [`ProjectTree::from_nodes`](crate::models::tree::ProjectTree::from_nodes).
pub fn nodes_from_files(
    files: impl IntoIterator<Item = (NodeId, String, Blob)>,
) -> Vec<Node> {
    let mut nodes: HashMap<NodeId, Node> = HashMap::new();

    for (id, path, blob) in files {
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let Some((name, ancestors)) = segments.split_last() else {
            continue; // empty path — skip
        };

        // Ensure a folder node for each ancestor directory, linking each to its
        // own parent. `acc` is the running path, which doubles as the folder id.
        let mut acc = String::new();
        let mut parent: Option<NodeId> = None;
        for segment in ancestors {
            if !acc.is_empty() {
                acc.push('/');
            }
            acc.push_str(segment);
            nodes.entry(acc.clone()).or_insert_with(|| Node {
                id: acc.clone(),
                parent: parent.clone(),
                name: segment.to_string(),
                content: NodeContent::Folder,
            });
            parent = Some(acc.clone());
        }

        nodes.insert(
            id.clone(),
            Node {
                id,
                parent,
                name: name.to_string(),
                content: NodeContent::File { blob },
            },
        );
    }

    nodes.into_values().collect()
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::models::tree::{NodeContent, ProjectTree};

    fn blob() -> Blob {
        Blob {
            sha256: "a".repeat(64),
            size: 1,
        }
    }

    #[test]
    fn test_root_files_have_no_folders() {
        let nodes = nodes_from_files([
            ("f1".to_string(), "main.typ".to_string(), blob()),
            ("f2".to_string(), "refs.bib".to_string(), blob()),
        ]);
        let tree = ProjectTree::from_nodes(nodes);
        tree.validate().unwrap();
        assert_eq!(tree.len(), 2);
        assert_eq!(tree.path_of("f1").unwrap(), "main.typ");
        assert!(tree.get("f1").unwrap().parent.is_none());
    }

    #[test]
    fn test_nested_path_creates_folder_nodes() {
        let nodes =
            nodes_from_files([("f".to_string(), "chapters/part1/intro.typ".to_string(), blob())]);
        let tree = ProjectTree::from_nodes(nodes);
        tree.validate().unwrap();

        // The file plus two derived folders.
        assert_eq!(tree.len(), 3);
        assert_eq!(tree.path_of("f").unwrap(), "chapters/part1/intro.typ");
        // Folder ids are their paths, and they nest correctly.
        assert!(matches!(
            tree.get("chapters").unwrap().content,
            NodeContent::Folder
        ));
        assert_eq!(tree.get("chapters").unwrap().parent, None);
        assert_eq!(
            tree.get("chapters/part1").unwrap().parent.as_deref(),
            Some("chapters")
        );
        assert_eq!(tree.get("f").unwrap().parent.as_deref(), Some("chapters/part1"));
    }

    #[test]
    fn test_folders_are_shared_and_deduplicated() {
        // Two files in the same folder yield one shared folder node.
        let nodes = nodes_from_files([
            ("a".to_string(), "chapters/a.typ".to_string(), blob()),
            ("b".to_string(), "chapters/b.typ".to_string(), blob()),
        ]);
        let tree = ProjectTree::from_nodes(nodes);
        tree.validate().unwrap();
        assert_eq!(tree.len(), 3); // chapters + a + b
        let mut names: Vec<&str> =
            tree.children(Some("chapters")).map(|n| n.name.as_str()).collect();
        names.sort();
        assert_eq!(names, vec!["a.typ", "b.typ"]);
    }

    #[test]
    fn test_file_node_carries_its_blob() {
        let nodes = nodes_from_files([("f".to_string(), "main.typ".to_string(), blob())]);
        let tree = ProjectTree::from_nodes(nodes);
        assert_eq!(tree.get("f").unwrap().blob(), Some(&blob()));
    }
}
