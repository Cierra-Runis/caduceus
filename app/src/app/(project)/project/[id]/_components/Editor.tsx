'use client';

import MonacoEditor from '@monaco-editor/react';
import { editor, MarkerSeverity } from 'monaco-editor';
import { useTheme } from 'next-themes';
import { useEffect, useState } from 'react';
import { MonacoBinding } from 'y-monaco';
import { WebsocketProvider } from 'y-websocket';
import * as Y from 'yjs';

import { Spinner } from '@/components/ui/spinner';
import { LspDiagnostic } from '@/lib/lsp/client';

export interface EditorProps {
  /// Diagnostics (from tinymist) for the focused file, rendered as squiggles.
  diagnostics?: LspDiagnostic[];
  provider: null | WebsocketProvider;
  /// The focused file's **id** — the key of its `Y.Text` in the shared doc.
  /// Keyed by id (not path) so a rename never detaches the buffer.
  textId: string;
  ydoc: Y.Doc;
}

// Collaborative Monaco buffer. The editor's model is bound to the focused
// file's `Y.Text` via y-monaco, so edits flow through the CRDT (and remote
// cursors render from awareness). There is no controlled `value`: Yjs owns the
// content. Typst syntax intelligence comes later with tinymist (M4).
export function Editor({ diagnostics, provider, textId, ydoc }: EditorProps) {
  const { resolvedTheme } = useTheme();
  const [instance, setInstance] = useState<editor.IStandaloneCodeEditor | null>(
    null,
  );

  // Render the focused file's diagnostics as Monaco markers (squiggles),
  // replacing the previous set. Owner `tinymist` scopes them so they don't
  // collide with any other marker source.
  useEffect(() => {
    const model = instance?.getModel();
    if (!model) return;
    editor.setModelMarkers(model, 'tinymist', (diagnostics ?? []).map(toMarker));
  }, [instance, diagnostics]);

  // Rebind whenever the focused file (or the editor/provider) changes. The
  // binding seeds the model from the shared text and keeps the two in sync;
  // destroying it on cleanup detaches before we bind the next file.
  useEffect(() => {
    const model = instance?.getModel();
    if (!instance || !model || !provider || !textId) return;

    const binding = new MonacoBinding(
      ydoc.getText(textId),
      model,
      new Set([instance]),
      provider.awareness,
    );
    return () => binding.destroy();
  }, [instance, provider, ydoc, textId]);

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

// LSP severity (1 error … 4 hint) → Monaco's marker severity.
function markerSeverity(severity?: number): MarkerSeverity {
  switch (severity) {
    case 2:
      return MarkerSeverity.Warning;
    case 3:
      return MarkerSeverity.Info;
    case 4:
      return MarkerSeverity.Hint;
    default:
      return MarkerSeverity.Error;
  }
}

// LSP diagnostic → Monaco marker. LSP positions are zero-based; Monaco's are
// one-based, so every line/character is offset by one.
function toMarker(d: LspDiagnostic): editor.IMarkerData {
  return {
    endColumn: d.range.end.character + 1,
    endLineNumber: d.range.end.line + 1,
    message: d.message,
    severity: markerSeverity(d.severity),
    source: d.source,
    startColumn: d.range.start.character + 1,
    startLineNumber: d.range.start.line + 1,
  };
}
