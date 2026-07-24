import { useEffect, useState } from 'react';
import * as Y from 'yjs';

import { fileEntries, FileEntry, readNodes } from '@/lib/yjs/tree';

// Subscribes to the project's CRDT file tree and returns the current files (id +
// derived path), re-reading whenever the shared `nodes` map changes — the
// initial server-seeded sync, and any later structural edit. `observeDeep`
// fires on nested node-map changes too (a rename, a re-parent), but not on text
// keystrokes, so this stays quiet while people type.
export function useFileTree(ydoc: Y.Doc): FileEntry[] {
  const [files, setFiles] = useState<FileEntry[]>(() =>
    fileEntries(readNodes(ydoc)),
  );

  useEffect(() => {
    const nodes = ydoc.getMap('nodes');
    const update = () => setFiles(fileEntries(readNodes(ydoc)));
    update(); // catch changes between initial render and this effect
    nodes.observeDeep(update);
    return () => nodes.unobserveDeep(update);
  }, [ydoc]);

  return files;
}
