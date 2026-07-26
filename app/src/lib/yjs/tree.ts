import * as Y from 'yjs';

// Client-side reader *and writer* for the project's file tree, which lives in
// the shared Y.Doc as a top-level `nodes` map — `Map<NodeId, Map>`, each entry a
// node with `kind` ('file' | 'folder'), `name`, an optional `parent`, and (for
// files) a `sha256` / `size` blob reference. A node's **path is derived** from
// its parent chain, never stored — mirroring the server's
// `ProjectTree::path_of`.
//
// The mutations here write structure straight into the CRDT; the server accepts
// and persists whatever lands in the map (it does not yet validate — that is a
// later, server-authoritative step). So these helpers enforce the same rules
// the server's `ProjectTree::validate` does (segment shape, sibling-name
// uniqueness) up front, to avoid producing a tree the authority would reject.

/// Top-level map name; must match the server's `crdt::NODES`.
const NODES = 'nodes';

/// sha256 of empty content. A freshly created file has no bytes yet, so its node
/// references the empty blob honestly; keeping `node.blob` in step with edited
/// text (and uploading the bytes) is a later concern (the text overlay flush).
const EMPTY_SHA256 =
  'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855';

/// Max length of a single path segment, in bytes — matches the server's
/// `MAX_NAME_LEN`.
export const MAX_NAME_LEN = 255;

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

/// Create a binary file node (image, font, …) referencing an already-uploaded
/// blob, under `parent` (null = root). Unlike [`createFile`] it declares **no**
/// text root: a binary file has no text overlay, so the server never flushes
/// text over its blob. Throws on an invalid or duplicate name.
export function createBinaryFile(
  ydoc: Y.Doc,
  name: string,
  parent: null | string,
  sha256: string,
  size: number,
): string {
  assertValidName(readNodes(ydoc), name, parent);
  const id = newObjectId();
  ydoc.transact(() => {
    const node = new Y.Map<unknown>();
    node.set('kind', 'file');
    node.set('name', name);
    if (parent !== null) node.set('parent', parent);
    node.set('sha256', sha256);
    node.set('size', size);
    ydoc.getMap<Y.Map<unknown>>(NODES).set(id, node);
  });
  return id;
}

/// Create a new file node (and its empty text root) under `parent` (null = root)
/// and return its id. Throws if `name` is not a valid segment or collides with a
/// sibling.
export function createFile(
  ydoc: Y.Doc,
  name: string,
  parent: null | string,
): string {
  assertValidName(readNodes(ydoc), name, parent);
  const id = newObjectId();
  ydoc.transact(() => {
    const node = new Y.Map<unknown>();
    node.set('kind', 'file');
    node.set('name', name);
    if (parent !== null) node.set('parent', parent);
    node.set('sha256', EMPTY_SHA256);
    node.set('size', 0);
    ydoc.getMap<Y.Map<unknown>>(NODES).set(id, node);
    // Declare the (empty) text root so the editor can bind to it immediately.
    ydoc.getText(id);
  });
  return id;
}

/// Create a new folder node under `parent` (null = root) and return its id.
/// Throws if `name` is not a valid segment or collides with a sibling.
export function createFolder(
  ydoc: Y.Doc,
  name: string,
  parent: null | string,
): string {
  assertValidName(readNodes(ydoc), name, parent);
  const id = newObjectId();
  ydoc.transact(() => {
    const node = new Y.Map<unknown>();
    node.set('kind', 'folder');
    node.set('name', name);
    if (parent !== null) node.set('parent', parent);
    ydoc.getMap<Y.Map<unknown>>(NODES).set(id, node);
  });
  return id;
}

/// Create a **text** file node seeded with `content`, under `parent` (null =
/// root). Unlike [`createBinaryFile`] it declares a Y.Text root (so the file is
/// editable) and, because the content already matches the uploaded blob
/// (`sha256` / `size`), the node references that blob immediately — the file is
/// "saved" from the start. Used for an uploaded file detected as text. Throws on
/// an invalid or duplicate name.
export function createTextFile(
  ydoc: Y.Doc,
  name: string,
  parent: null | string,
  content: string,
  sha256: string,
  size: number,
): string {
  assertValidName(readNodes(ydoc), name, parent);
  const id = newObjectId();
  ydoc.transact(() => {
    const node = new Y.Map<unknown>();
    node.set('kind', 'file');
    node.set('name', name);
    if (parent !== null) node.set('parent', parent);
    node.set('sha256', sha256);
    node.set('size', size);
    ydoc.getMap<Y.Map<unknown>>(NODES).set(id, node);
    // Declare the text root and seed it — this makes the node a *text* file
    // (it has an overlay), distinguishing it from a binary blob.
    const text = ydoc.getText(id);
    if (content) text.insert(0, content);
  });
  return id;
}

/// Delete a node and its whole subtree: the node, every descendant, and each
/// removed file's text. A file deletes just itself; a folder takes everything
/// under it (recursively), so no child is ever left orphaned.
export function deleteNode(ydoc: Y.Doc, id: string): void {
  const nodes = readNodes(ydoc);
  const target = nodes.find((node) => node.id === id);
  if (!target) return;

  const childrenOf = new Map<string, TreeNode[]>();
  for (const node of nodes) {
    if (node.parent !== null) {
      const siblings = childrenOf.get(node.parent) ?? [];
      siblings.push(node);
      childrenOf.set(node.parent, siblings);
    }
  }

  // Depth-first collect the subtree rooted at `target`.
  const subtree: TreeNode[] = [];
  const stack = [target];
  for (let node = stack.pop(); node !== undefined; node = stack.pop()) {
    subtree.push(node);
    stack.push(...(childrenOf.get(node.id) ?? []));
  }

  const map = ydoc.getMap<Y.Map<unknown>>(NODES);
  ydoc.transact(() => {
    for (const node of subtree) {
      map.delete(node.id);
      if (node.kind === 'file') {
        const text = ydoc.getText(node.id);
        if (text.length > 0) text.delete(0, text.length);
      }
    }
  });
}

/// Ensure a chain of nested folders exists under the root, creating any that are
/// missing, and return the id of the deepest one (or `null` for an empty chain,
/// i.e. the root). Used to recreate a folder hierarchy on folder upload. Runs
/// synchronously, so concurrent uploads calling it never race.
export function ensureFolderPath(
  ydoc: Y.Doc,
  segments: string[],
): null | string {
  let parent: null | string = null;
  for (const segment of segments) {
    const existing = readNodes(ydoc).find(
      (node) =>
        node.parent === parent &&
        node.name === segment &&
        node.kind === 'folder',
    );
    parent = existing ? existing.id : createFolder(ydoc, segment, parent);
  }
  return parent;
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

/// Extensions the editor treats as binary — content that is a blob, not editable
/// text, so it is never opened in the code editor (which would create a text
/// overlay and let the server flush empty text over the blob).
const BINARY_EXTENSIONS = new Set([
  'avif', 'bmp', 'gif', 'ico', 'jpeg', 'jpg', 'otf', 'pdf', 'png',
  'ttf', 'webp', 'woff', 'woff2',
]);

/// Whether the file node `id` is binary — i.e. it has **no** Y.Text overlay.
/// A text file declares its text root at creation (see [`createFile`] /
/// [`createTextFile`]); a binary file does not. Mirrors the server's rule
/// ("`get_text` → `None` means binary"), so it never opens a blob in the editor
/// (which would create an empty overlay the server flushes over the bytes).
export function isBinaryFile(ydoc: Y.Doc, id: string): boolean {
  return !ydoc.share.has(id);
}

/// Whether `path` names a binary file, by extension. A hint used when *deciding*
/// how to store an upload; the authoritative "is this file binary" check for an
/// existing node is [`isBinaryFile`] (does it have a text overlay).
export function isBinaryPath(path: string): boolean {
  const dot = path.lastIndexOf('.');
  return dot !== -1 && BINARY_EXTENSIONS.has(path.slice(dot + 1).toLowerCase());
}

/// Whether `name` is a legal single path segment, matching the server's
/// `is_valid_segment`: non-empty, ≤ MAX_NAME_LEN bytes, not `.`/`..`, no `/` or
/// `\`, no control characters, and no leading/trailing whitespace.
export function isValidSegment(name: string): boolean {
  return (
    name.length > 0 &&
    new TextEncoder().encode(name).length <= MAX_NAME_LEN &&
    name !== '.' &&
    name !== '..' &&
    !name.includes('/') &&
    !name.includes('\\') &&
    !/\p{Cc}/u.test(name) &&
    name.trim() === name
  );
}

/// Move a node under `newParent` (null = root) by rewriting its `parent` field.
/// Throws if the destination isn't a folder, if it would put the node inside
/// itself or a descendant (a cycle), or if the destination already holds a
/// sibling with this node's name. A no-op when it's already there.
export function moveNode(
  ydoc: Y.Doc,
  id: string,
  newParent: null | string,
): void {
  const nodes = readNodes(ydoc);
  const node = nodes.find((n) => n.id === id);
  if (!node) throw new Error(`no such node: ${id}`);
  if (node.parent === newParent) return;
  if (newParent !== null) {
    const dest = nodes.find((n) => n.id === newParent);
    if (!dest || dest.kind !== 'folder') {
      throw new Error('destination is not a folder');
    }
    if (newParent === id || isDescendant(nodes, newParent, id)) {
      throw new Error('cannot move a folder into itself');
    }
  }
  if (
    nodes.some(
      (n) => n.parent === newParent && n.id !== id && n.name === node.name,
    )
  ) {
    throw new Error(`"${node.name}" already exists there`);
  }
  const map = ydoc.getMap<Y.Map<unknown>>(NODES);
  ydoc.transact(() => {
    const entry = map.get(id);
    if (entry instanceof Y.Map) {
      if (newParent === null) entry.delete('parent');
      else entry.set('parent', newParent);
    }
  });
}

/// The blob a file node references (its uploaded bytes), or `undefined` for a
/// folder or a not-yet-synced node. Used to fetch a binary file's content.
export function readFileBlob(
  ydoc: Y.Doc,
  id: string,
): { sha256: string; size: number } | undefined {
  const node = ydoc.getMap<Y.Map<unknown>>(NODES).get(id);
  if (!(node instanceof Y.Map)) return undefined;
  const sha256 = node.get('sha256');
  if (typeof sha256 !== 'string') return undefined;
  const size = node.get('size');
  return { sha256, size: typeof size === 'number' ? size : 0 };
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

/// Rename a node (a single `name` field write, so a concurrent edit to any other
/// field merges). Throws if the new name is invalid or collides with a sibling.
export function renameNode(ydoc: Y.Doc, id: string, name: string): void {
  const nodes = readNodes(ydoc);
  const node = nodes.find((n) => n.id === id);
  if (!node) throw new Error(`no such node: ${id}`);
  assertValidName(nodes, name, node.parent, id);
  const map = ydoc.getMap<Y.Map<unknown>>(NODES);
  ydoc.transact(() => {
    const entry = map.get(id);
    if (entry instanceof Y.Map) entry.set('name', name);
  });
}

/// Reject an invalid segment, or one already used by a sibling of `parent`
/// (excluding `exceptId`, so renaming a node to its own name is fine).
function assertValidName(
  nodes: TreeNode[],
  name: string,
  parent: null | string,
  exceptId?: string,
): void {
  if (!isValidSegment(name)) {
    throw new Error(`invalid file name: ${JSON.stringify(name)}`);
  }
  const taken = nodes.some(
    (node) =>
      node.parent === parent && node.id !== exceptId && node.name === name,
  );
  if (taken) {
    throw new Error(`"${name}" already exists here`);
  }
}

/// Whether `candidate` sits inside the subtree rooted at `ancestorId` (walking
/// up from `candidate` reaches it). Guards a move against dropping a folder into
/// its own descendant.
function isDescendant(
  nodes: TreeNode[],
  candidate: string,
  ancestorId: string,
): boolean {
  const byId = new Map(nodes.map((n) => [n.id, n]));
  const seen = new Set<string>();
  let current = byId.get(candidate);
  while (current && current.parent !== null) {
    if (seen.has(current.id)) return false; // cycle guard
    seen.add(current.id);
    if (current.parent === ancestorId) return true;
    current = byId.get(current.parent);
  }
  return false;
}

/// Generate a 24-hex-char id in MongoDB ObjectId layout (4-byte big-endian
/// timestamp + 8 random bytes). The server persists file text back to Mongo
/// keyed by this id via `ObjectId::parse_str`, so it must parse as an ObjectId.
function newObjectId(): string {
  const bytes = new Uint8Array(12);
  crypto.getRandomValues(bytes);
  const secs = Math.floor(Date.now() / 1000);
  bytes[0] = (secs >>> 24) & 0xff;
  bytes[1] = (secs >>> 16) & 0xff;
  bytes[2] = (secs >>> 8) & 0xff;
  bytes[3] = secs & 0xff;
  return [...bytes].map((b) => b.toString(16).padStart(2, '0')).join('');
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
