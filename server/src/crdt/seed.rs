//! Building an id-keyed [`Node`] tree from a flat path-keyed file list.
//!
//! **Test-only.** This once cold-started a room from the old flat Mongo `files`
//! list (full paths like `chapters/intro.typ`) before that source was retired
//! by the `files → tree` migration; production now cold-starts from
//! `Project.tree` via [`ProjectTree::from_nodes`](crate::models::tree::ProjectTree::from_nodes).
//! It survives as a convenience for building a populated tree from paths in
//! tests: a file node per file, plus the folder nodes its path implies.
//!
//! Every node id is opaque (files keep their real id; folders get a freshly
//! generated one) — a folder's *path* is derived like everything else, never its
//! identity. Opaque hex ids are also safe as downstream map keys, where a path
//! (which can contain `.`) would not be.

use std::collections::HashMap;

use bson::oid::ObjectId;

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
    // Folder path -> its generated node id, so files sharing a folder link to
    // the same node.
    let mut folder_ids: HashMap<String, NodeId> = HashMap::new();

    for (id, path, blob) in files {
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let Some((name, ancestors)) = segments.split_last() else {
            continue; // empty path — skip
        };

        // Ensure a folder node for each ancestor directory, linking each to its
        // own parent. `acc` is the running path used to dedupe folders.
        let mut acc = String::new();
        let mut parent: Option<NodeId> = None;
        for segment in ancestors {
            if !acc.is_empty() {
                acc.push('/');
            }
            acc.push_str(segment);
            let folder_id = folder_ids
                .entry(acc.clone())
                .or_insert_with(|| ObjectId::new().to_hex())
                .clone();
            nodes.entry(folder_id.clone()).or_insert_with(|| Node {
                id: folder_id.clone(),
                parent: parent.clone(),
                name: segment.to_string(),
                content: NodeContent::Folder,
            });
            parent = Some(folder_id);
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
    use crate::models::tree::ProjectTree;

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

        // The file plus two derived folders, and the path derives back.
        assert_eq!(tree.len(), 3);
        assert_eq!(tree.path_of("f").unwrap(), "chapters/part1/intro.typ");

        // Folders exist as opaque nodes (their ids are not their paths).
        let chapters = tree.children(None).find(|n| n.is_folder()).unwrap();
        assert_eq!(chapters.name, "chapters");
        assert_ne!(chapters.id, "chapters"); // opaque id, not the path

        let part1_id = tree.get("f").unwrap().parent.clone().unwrap();
        let part1 = tree.get(&part1_id).unwrap();
        assert_eq!(part1.name, "part1");
        assert_eq!(part1.parent.as_deref(), Some(chapters.id.as_str()));
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
        assert_eq!(tree.len(), 3); // one shared "chapters" + a + b

        let chapters = tree.children(None).find(|n| n.name == "chapters").unwrap();
        let mut names: Vec<&str> = tree
            .children(Some(&chapters.id))
            .map(|n| n.name.as_str())
            .collect();
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
