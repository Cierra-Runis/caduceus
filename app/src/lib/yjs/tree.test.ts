import { describe, expect, it } from 'vitest';
import * as Y from 'yjs';

import { fileEntries, readNodes, TreeNode } from './tree';

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
