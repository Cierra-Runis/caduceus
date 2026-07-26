'use client';

import { RefObject } from 'react';
import { Panel, PanelImperativeHandle } from 'react-resizable-panels';
import { WebsocketProvider } from 'y-websocket';
import * as Y from 'yjs';

import { BinaryFileView } from './BinaryFileView';
import { Editor } from './Editor';

export interface EditorPanelProps {
  /// When set, the focused file is binary: render its preview instead of the
  /// code editor, so Monaco never opens it (which would create a text overlay
  /// the server would flush empty over the blob).
  binary: BinaryFile | null;
  editorPanelRef: RefObject<null | PanelImperativeHandle>;
  provider: null | WebsocketProvider;
  textId: string;
  ydoc: Y.Doc;
}

interface BinaryFile {
  path: string;
  projectId: string;
  sha256: string;
  size: number;
}

export function EditorPanel({
  binary,
  editorPanelRef,
  provider,
  textId,
  ydoc,
}: EditorPanelProps) {
  return (
    <Panel
      collapsible
      defaultSize={50}
      id='editor'
      minSize={20}
      panelRef={editorPanelRef}
      style={{ overflow: 'auto' }}
    >
      {binary ? (
        <BinaryFileView
          path={binary.path}
          projectId={binary.projectId}
          sha256={binary.sha256}
          size={binary.size}
        />
      ) : (
        <Editor provider={provider} textId={textId} ydoc={ydoc} />
      )}
    </Panel>
  );
}
