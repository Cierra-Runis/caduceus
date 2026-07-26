import { useEffect, useState } from 'react';
import * as Y from 'yjs';

import { readNodes, TreeNode } from '@/lib/yjs/tree';

// Subscribes to the project's CRDT file tree and returns all nodes (files and
// folders), re-reading whenever the shared `nodes` map changes — the initial
// server-seeded sync, and any later structural edit. `observeDeep` fires on
// nested node-map changes too (a rename, a re-parent), but not on text
// keystrokes, so this stays quiet while people type. Callers derive what they
// need: the sidebar builds the tree, the editor/preview take `fileEntries`.
export function useProjectNodes(ydoc: Y.Doc): TreeNode[] {
  const [nodes, setNodes] = useState<TreeNode[]>(() => readNodes(ydoc));

  useEffect(() => {
    const map = ydoc.getMap('nodes');
    const update = () => setNodes(readNodes(ydoc));
    update(); // catch changes between initial render and this effect
    map.observeDeep(update);
    return () => map.unobserveDeep(update);
  }, [ydoc]);

  return nodes;
}
