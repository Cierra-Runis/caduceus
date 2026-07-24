import { describe, expect, it } from 'vitest';
import * as Y from 'yjs';

import {
  createFile,
  deleteFile,
  fileEntries,
  isValidSegment,
  readNodes,
  renameNode,
  TreeNode,
} from './tree';

// Build a project Y.Doc's `nodes` map the way the server's codec does: one
// entry per node, each a nested Y.Map with `kind` / `name` (+ `parent`, +
// `sha256` / `size` for files). Mutating helpers keep the tests close to the
// real wire shape rather than hand-rolling `TreeNode`s.
function docWith(
  nodes: {
    id: string;
    kind: 'file' | 'folder';
    name: string;
    parent?: string;
  }[],
): Y.Doc {
  const doc = new Y.Doc();
  const map = doc.getMap<Y.Map<unknown>>('nodes');
  doc.transact(() => {
    for (const node of nodes) {
      const entry = new Y.Map<unknown>();
      entry.set('kind', node.kind);
      entry.set('name', node.name);
      if (node.parent !== undefined) entry.set('parent', node.parent);
      if (node.kind === 'file') {
        entry.set('sha256', 'a'.repeat(64));
        entry.set('size', 0);
      }
      map.set(node.id, entry);
    }
  });
  return doc;
}

describe('readNodes', () => {
  it('decodes each node with its optional parent', () => {
    const doc = docWith([
      { id: 'd', kind: 'folder', name: 'chapters' },
      { id: 'f', kind: 'file', name: 'intro.typ', parent: 'd' },
    ]);
    const nodes = readNodes(doc);
    expect(nodes).toContainEqual<TreeNode>({
      id: 'd',
      kind: 'folder',
      name: 'chapters',
      parent: null,
    });
    expect(nodes).toContainEqual<TreeNode>({
      id: 'f',
      kind: 'file',
      name: 'intro.typ',
      parent: 'd',
    });
  });

  it('skips a half-written node missing its required fields', () => {
    const doc = new Y.Doc();
    const map = doc.getMap<Y.Map<unknown>>('nodes');
    const partial = new Y.Map<unknown>();
    partial.set('kind', 'file'); // no `name` yet
    map.set('x', partial);
    expect(readNodes(doc)).toEqual([]);
  });

  it('is empty for a fresh doc', () => {
    expect(readNodes(new Y.Doc())).toEqual([]);
  });
});

describe('fileEntries', () => {
  it('derives a nested path from the parent chain', () => {
    const doc = docWith([
      { id: 'd', kind: 'folder', name: 'chapters' },
      { id: 'p', kind: 'folder', name: 'part1', parent: 'd' },
      { id: 'f', kind: 'file', name: 'intro.typ', parent: 'p' },
    ]);
    expect(fileEntries(readNodes(doc))).toEqual([
      { id: 'f', path: 'chapters/part1/intro.typ' },
    ]);
  });

  it('lists only files, sorted by path, never folders', () => {
    const doc = docWith([
      { id: 'd', kind: 'folder', name: 'chapters' },
      { id: 'b', kind: 'file', name: 'b.typ', parent: 'd' },
      { id: 'a', kind: 'file', name: 'a.typ', parent: 'd' },
      { id: 'm', kind: 'file', name: 'main.typ' },
    ]);
    expect(fileEntries(readNodes(doc))).toEqual([
      { id: 'a', path: 'chapters/a.typ' },
      { id: 'b', path: 'chapters/b.typ' },
      { id: 'm', path: 'main.typ' },
    ]);
  });

  it('drops a file whose parent is dangling', () => {
    // `f` points at a folder that isn't in the map (a transient mid-sync view).
    const doc = docWith([{ id: 'f', kind: 'file', name: 'x.typ', parent: 'gone' }]);
    expect(fileEntries(readNodes(doc))).toEqual([]);
  });
});

describe('isValidSegment', () => {
  it('accepts ordinary file names', () => {
    expect(isValidSegment('main.typ')).toBe(true);
    expect(isValidSegment('a nice name.bib')).toBe(true);
  });

  it('rejects empty, dot, separators, and edge whitespace', () => {
    for (const bad of ['', '.', '..', 'a/b', 'a\\b', ' lead', 'trail ', 'a\tb']) {
      expect(isValidSegment(bad)).toBe(false);
    }
  });
});

describe('createFile', () => {
  it('adds a file node with a 24-hex id and an empty text root', () => {
    const doc = new Y.Doc();
    const id = createFile(doc, 'main.typ', null);
    expect(id).toMatch(/^[0-9a-f]{24}$/);
    expect(fileEntries(readNodes(doc))).toEqual([{ id, path: 'main.typ' }]);
    expect(doc.getText(id).toString()).toBe('');
  });

  it('nests under a parent folder, deriving the path', () => {
    const doc = docWith([{ id: 'd', kind: 'folder', name: 'chapters' }]);
    const id = createFile(doc, 'intro.typ', 'd');
    expect(fileEntries(readNodes(doc))).toContainEqual({
      id,
      path: 'chapters/intro.typ',
    });
  });

  it('rejects an invalid name and a duplicate sibling', () => {
    const doc = new Y.Doc();
    createFile(doc, 'main.typ', null);
    expect(() => createFile(doc, 'a/b.typ', null)).toThrow();
    expect(() => createFile(doc, 'main.typ', null)).toThrow();
    // The rejected attempts left the tree untouched.
    expect(readNodes(doc)).toHaveLength(1);
  });
});

describe('renameNode', () => {
  it('changes the name, re-deriving the path', () => {
    const doc = new Y.Doc();
    const id = createFile(doc, 'main.typ', null);
    renameNode(doc, id, 'index.typ');
    expect(fileEntries(readNodes(doc))).toEqual([{ id, path: 'index.typ' }]);
  });

  it('rejects a name already taken by a sibling', () => {
    const doc = new Y.Doc();
    createFile(doc, 'a.typ', null);
    const b = createFile(doc, 'b.typ', null);
    expect(() => renameNode(doc, b, 'a.typ')).toThrow();
  });

  it('allows renaming a node to its own current name', () => {
    const doc = new Y.Doc();
    const id = createFile(doc, 'a.typ', null);
    expect(() => renameNode(doc, id, 'a.typ')).not.toThrow();
  });
});

describe('deleteFile', () => {
  it('removes the node and clears its text', () => {
    const doc = new Y.Doc();
    const id = createFile(doc, 'a.typ', null);
    doc.getText(id).insert(0, 'hello');
    deleteFile(doc, id);
    expect(readNodes(doc)).toEqual([]);
    expect(doc.getText(id).toString()).toBe('');
  });

  it('refuses to delete a folder', () => {
    const doc = docWith([{ id: 'd', kind: 'folder', name: 'chapters' }]);
    expect(() => deleteFile(doc, 'd')).toThrow();
    expect(readNodes(doc)).toHaveLength(1);
  });
});
