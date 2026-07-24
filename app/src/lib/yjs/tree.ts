import * as Y from 'yjs';

// Client-side reader for the project's file tree, which lives in the shared
// Y.Doc as a top-level `nodes` map — `Map<NodeId, Map>`, each entry a node with
// `kind` ('file' | 'folder'), `name`, an optional `parent`, and (for files) a
// `sha256` / `size` blob reference. The server is authoritative: it seeds and
// validates this map, and the client only *reads* structure from it here (CRUD
// is a later step). A node's **path is derived** from its parent chain, never
// stored — mirroring the server's `ProjectTree::path_of`.

/// Top-level map name; must match the server's `crdt::NODES`.
const NODES = 'nodes';

/// A file with its derived path, for the sidebar/editor. Keyed by `id` (stable
/// across renames); `path` is recomputed from the parent chain.
export interface FileEntry {
  id: string;
  path: string;
}

/// A decoded node — the plain-object view of one `nodes` entry.
export interface TreeNode {
  id: string;
  kind: 'file' | 'folder';
  name: string;
  parent: null | string;
}

/// Derive each file's full path from its parent chain and return the files
/// sorted by path. Folders contribute path segments but aren't listed. A node
/// whose chain is broken (a missing or cyclic parent) is dropped — the server
/// keeps the tree valid, so this only guards against a transient mid-sync view.
export function fileEntries(nodes: TreeNode[]): FileEntry[] {
  const byId = new Map(nodes.map((node) => [node.id, node]));
  return nodes
    .filter((node) => node.kind === 'file')
    .map((node) => ({ id: node.id, path: pathOf(node, byId) }))
    .filter((entry): entry is FileEntry => entry.path !== null)
    .sort((a, b) => a.path.localeCompare(b.path));
}

/// Decode the `nodes` map of a project Y.Doc into plain nodes. Entries missing
/// the required fields (e.g. mid-sync, before every field has arrived) are
/// skipped rather than throwing — the map is eventually consistent.
export function readNodes(ydoc: Y.Doc): TreeNode[] {
  const map = ydoc.getMap<Y.Map<unknown>>(NODES);
  const out: TreeNode[] = [];
  map.forEach((node, id) => {
    if (!(node instanceof Y.Map)) return;
    const kind = node.get('kind');
    const name = node.get('name');
    if ((kind !== 'file' && kind !== 'folder') || typeof name !== 'string') {
      return;
    }
    const parent = node.get('parent');
    out.push({
      id,
      kind,
      name,
      parent: typeof parent === 'string' ? parent : null,
    });
  });
  return out;
}

/// Walk `node`'s parent chain to its root, joining names into a `/`-path.
/// Returns `null` if an ancestor id is dangling or the chain loops.
function pathOf(node: TreeNode, byId: Map<string, TreeNode>): null | string {
  const segments: string[] = [];
  const seen = new Set<string>();
  let current: TreeNode | undefined = node;
  while (current) {
    if (seen.has(current.id)) return null; // cycle
    seen.add(current.id);
    segments.unshift(current.name);
    if (current.parent === null) return segments.join('/');
    current = byId.get(current.parent);
  }
  return null; // dangling parent
}
