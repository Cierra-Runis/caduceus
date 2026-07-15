'use client';

import MonacoEditor from '@monaco-editor/react';
import { editor } from 'monaco-editor';
import { useTheme } from 'next-themes';
import { useEffect, useState } from 'react';
import { MonacoBinding } from 'y-monaco';
import { WebsocketProvider } from 'y-websocket';
import * as Y from 'yjs';

import { Spinner } from '@/components/ui/spinner';

export interface EditorProps {
  /// Virtual-FS path of the file currently open in the editor.
  path: string;
  provider: null | WebsocketProvider;
  ydoc: Y.Doc;
}

// Collaborative Monaco buffer. The editor's model is bound to the focused
// file's `Y.Text` via y-monaco, so edits flow through the CRDT (and remote
// cursors render from awareness). There is no controlled `value`: Yjs owns the
// content. Typst syntax intelligence comes later with tinymist (M4).
export function Editor({ path, provider, ydoc }: EditorProps) {
  const { resolvedTheme } = useTheme();
  const [instance, setInstance] = useState<editor.IStandaloneCodeEditor | null>(
    null,
  );

  // Rebind whenever the focused file (or the editor/provider) changes. The
  // binding seeds the model from the shared text and keeps the two in sync;
  // destroying it on cleanup detaches before we bind the next file.
  useEffect(() => {
    const model = instance?.getModel();
    if (!instance || !model || !provider || !path) return;

    const binding = new MonacoBinding(
      ydoc.getText(path),
      model,
      new Set([instance]),
      provider.awareness,
    );
    return () => binding.destroy();
  }, [instance, provider, ydoc, path]);

  return (
    <MonacoEditor
      loading={<Spinner />}
      onMount={setInstance}
      options={{
        cursorBlinking: 'smooth',
        fontFamily: 'var(--font-mono)',
        fontLigatures: true,
        smoothScrolling: true,
      }}
      theme={resolvedTheme === 'dark' ? 'vs-dark' : 'light'}
    />
  );
}
